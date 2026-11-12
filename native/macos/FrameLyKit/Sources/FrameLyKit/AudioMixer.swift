import AVFoundation
import CoreMedia

/// Mixes timestamped PCM from several sources (system audio, microphone) into one continuous
/// 48 kHz stereo Float32 stream on a shared timeline.
///
/// Sync model: every buffer is placed by its host-clock timestamp. Buffers that land within
/// `snapFrames` (10 ms) of where the source's previous buffer ended are glued on contiguously
/// (no clicks from timestamp jitter); larger deviations — device clock drift accumulating past
/// 10 ms, or a gap — resynchronize to the timestamp. A/V offset therefore stays within ~10 ms
/// for any recording length.
///
/// Output is released up to the point every active source has delivered, but never more than
/// `latencyFrames` behind the most advanced source, so a silent or stalled source (e.g. a mic
/// that was unplugged) never blocks the others; its missing span is silence.
final class AudioMixer {
    static let sampleRate = 48_000
    static let channels = 2

    struct Source {
        var gain: Float
        var writtenUntil: Int64?
        var active = true
        /// Peak of the most recent buffer, after gain (0…1+).
        var peak: Float = 0
    }

    private(set) var sources: [Source]
    /// Interleaved stereo samples starting at frame `readIndex`.
    private var buffer: [Float] = []
    private(set) var readIndex: Int64 = 0
    private(set) var droppedLateFrames: Int64 = 0
    let origin: CMTime
    let latencyFrames: Int64
    let snapFrames: Int64

    init(origin: CMTime, gains: [Float], latency: Double = 0.25, snap: Double = 0.010) {
        self.origin = origin
        sources = gains.map { Source(gain: $0) }
        latencyFrames = Int64(latency * Double(Self.sampleRate))
        snapFrames = Int64(snap * Double(Self.sampleRate))
    }

    func frameIndex(of pts: CMTime) -> Int64 {
        Int64(((pts - origin).seconds * Double(Self.sampleRate)).rounded())
    }

    func time(ofFrame frame: Int64) -> CMTime {
        origin + CMTime(value: frame, timescale: CMTimeScale(Self.sampleRate))
    }

    /// Adds interleaved stereo `samples` captured at host time `pts` from source `index`.
    func push(source index: Int, pts: CMTime, samples: [Float]) {
        let frames = Int64(samples.count / Self.channels)
        guard frames > 0, sources.indices.contains(index), sources[index].active else { return }
        var start = frameIndex(of: pts)
        if let w = sources[index].writtenUntil, abs(start - w) <= snapFrames { start = w }
        let gain = sources[index].gain
        sources[index].peak = samples.reduce(0) { max($0, abs($1)) } * gain
        sources[index].writtenUntil = max(sources[index].writtenUntil ?? 0, start + frames)

        // Skip the part that falls before what was already released.
        var skip: Int64 = 0
        if start < readIndex {
            skip = min(frames, readIndex - start)
            droppedLateFrames += skip
        }
        guard skip < frames else { return }
        let offset = Int((start + skip - readIndex)) * Self.channels
        let needed = offset + Int(frames - skip) * Self.channels
        if buffer.count < needed { buffer.append(contentsOf: repeatElement(0, count: needed - buffer.count)) }
        let src = Int(skip) * Self.channels
        for i in 0..<(Int(frames - skip) * Self.channels) {
            buffer[offset + i] += samples[src + i] * gain
        }
    }

    /// A source that stopped for good (device unplugged): never wait for it again.
    func deactivate(source index: Int) {
        guard sources.indices.contains(index) else { return }
        sources[index].active = false
        sources[index].peak = 0
    }

    /// Releases mixed audio. `flush` releases everything buffered (end of recording).
    func pull(flush: Bool = false) -> (startFrame: Int64, samples: [Float])? {
        let written = sources.filter(\.active).compactMap(\.writtenUntil)
        let all = sources.compactMap(\.writtenUntil)
        guard let maxWritten = all.max() else { return nil }
        var target = flush ? maxWritten : max(written.min() ?? maxWritten, maxWritten - latencyFrames)
        target = min(target, maxWritten)
        guard target > readIndex else { return nil }
        let count = Int(target - readIndex) * Self.channels
        if buffer.count < count { buffer.append(contentsOf: repeatElement(0, count: count - buffer.count)) }
        var out = Array(buffer[0..<count])
        buffer.removeFirst(count)
        for i in out.indices { out[i] = Self.softClip(out[i]) }
        let start = readIndex
        readIndex = target
        return (start, out)
    }

    /// Transparent below 0.8, smoothly limits above, never exceeds ±1.
    static func softClip(_ x: Float) -> Float {
        let threshold: Float = 0.8
        let a = abs(x)
        guard a > threshold else { return x }
        let over = (a - threshold) / (1 - threshold)
        let limited = threshold + (1 - threshold) * tanh(over)
        return x < 0 ? -limited : limited
    }
}

// MARK: - PCM conversion

enum PCM {
    /// Extracts audio as interleaved stereo Float32 at 48 kHz from any linear PCM sample buffer
    /// (Float32 or Int16, interleaved or planar, mono or stereo, any sample rate).
    static func interleavedStereo(_ sb: CMSampleBuffer) -> [Float]? {
        guard let desc = sb.formatDescription,
              let asbd = CMAudioFormatDescriptionGetStreamBasicDescription(desc)?.pointee,
              asbd.mFormatID == kAudioFormatLinearPCM
        else { return nil }
        let frames = sb.numSamples
        guard frames > 0 else { return nil }
        let channels = Int(asbd.mChannelsPerFrame)
        let isFloat = asbd.mFormatFlags & kAudioFormatFlagIsFloat != 0
        let planar = asbd.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0
        let bits = Int(asbd.mBitsPerChannel)
        guard channels >= 1, (isFloat && bits == 32) || (!isFloat && bits == 16) else { return nil }

        var blockBuffer: CMBlockBuffer?
        var listSize = 0
        CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(sb, bufferListSizeNeededOut: &listSize, bufferListOut: nil, bufferListSize: 0,
                                                                blockBufferAllocator: nil, blockBufferMemoryAllocator: nil, flags: 0, blockBufferOut: nil)
        let raw = UnsafeMutableRawPointer.allocate(byteCount: listSize, alignment: MemoryLayout<AudioBufferList>.alignment)
        defer { raw.deallocate() }
        let list = raw.bindMemory(to: AudioBufferList.self, capacity: 1)
        guard CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
            sb, bufferListSizeNeededOut: nil, bufferListOut: list, bufferListSize: listSize,
            blockBufferAllocator: nil, blockBufferMemoryAllocator: nil,
            flags: kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment, blockBufferOut: &blockBuffer
        ) == noErr else { return nil }
        let buffers = UnsafeMutableAudioBufferListPointer(list)

        func sample(_ channel: Int, _ frame: Int) -> Float {
            let ch = min(channel, channels - 1)
            let (buf, index) = planar ? (buffers[ch], frame) : (buffers[0], frame * channels + ch)
            guard let data = buf.mData else { return 0 }
            return isFloat
                ? data.assumingMemoryBound(to: Float.self)[index]
                : Float(data.assumingMemoryBound(to: Int16.self)[index]) / 32768
        }
        var out = [Float](repeating: 0, count: frames * 2)
        for f in 0..<frames {
            out[f * 2] = sample(0, f)
            out[f * 2 + 1] = sample(1, f)
        }
        let rate = Int(asbd.mSampleRate.rounded())
        return rate == AudioMixer.sampleRate ? out : resample(out, from: rate, to: AudioMixer.sampleRate)
    }

    /// Linear-interpolation resampler for interleaved stereo (only used for unusual device rates).
    static func resample(_ input: [Float], from: Int, to: Int) -> [Float] {
        let inFrames = input.count / 2
        guard from > 0, to > 0, inFrames > 1 else { return input }
        let outFrames = Int((Double(inFrames) * Double(to) / Double(from)).rounded())
        var out = [Float](repeating: 0, count: outFrames * 2)
        let step = Double(from) / Double(to)
        for f in 0..<outFrames {
            let pos = Double(f) * step
            let i = min(Int(pos), inFrames - 2)
            let frac = Float(pos - Double(i))
            for c in 0..<2 {
                out[f * 2 + c] = input[i * 2 + c] * (1 - frac) + input[(i + 1) * 2 + c] * frac
            }
        }
        return out
    }

    /// Wraps interleaved stereo Float32 48 kHz samples in a CMSampleBuffer for AVAssetWriter.
    static func sampleBuffer(_ samples: [Float], pts: CMTime) -> CMSampleBuffer? {
        let frames = samples.count / 2
        guard frames > 0 else { return nil }
        var asbd = AudioStreamBasicDescription(
            mSampleRate: Double(AudioMixer.sampleRate), mFormatID: kAudioFormatLinearPCM,
            mFormatFlags: kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked,
            mBytesPerPacket: 8, mFramesPerPacket: 1, mBytesPerFrame: 8, mChannelsPerFrame: 2,
            mBitsPerChannel: 32, mReserved: 0
        )
        var format: CMAudioFormatDescription?
        guard CMAudioFormatDescriptionCreate(allocator: nil, asbd: &asbd, layoutSize: 0, layout: nil, magicCookieSize: 0,
                                             magicCookie: nil, extensions: nil, formatDescriptionOut: &format) == noErr,
              let format
        else { return nil }
        let bytes = samples.count * MemoryLayout<Float>.size
        var block: CMBlockBuffer?
        guard CMBlockBufferCreateWithMemoryBlock(allocator: nil, memoryBlock: nil, blockLength: bytes, blockAllocator: nil,
                                                 customBlockSource: nil, offsetToData: 0, dataLength: bytes,
                                                 flags: kCMBlockBufferAssureMemoryNowFlag, blockBufferOut: &block) == noErr,
              let block
        else { return nil }
        let copied = samples.withUnsafeBytes { CMBlockBufferReplaceDataBytes(with: $0.baseAddress!, blockBuffer: block, offsetIntoDestination: 0, dataLength: bytes) }
        guard copied == noErr else { return nil }
        var out: CMSampleBuffer?
        guard CMAudioSampleBufferCreateReadyWithPacketDescriptions(allocator: nil, dataBuffer: block, formatDescription: format,
                                                                   sampleCount: frames, presentationTimeStamp: pts,
                                                                   packetDescriptions: nil, sampleBufferOut: &out) == noErr
        else { return nil }
        return out
    }
}
