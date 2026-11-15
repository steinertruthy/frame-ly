import AVFoundation

// Audio for speech recognition: decodes any file AVFoundation can read (video or audio) to
// 16 kHz mono 32-bit float PCM (little-endian, no header), streamed into `dest` as it decodes,
// so long recordings never sit in memory. Apple's converter does the resampling and downmix.

struct DecodeResult: Encodable {
    let samples: Int
    let durationMs: Int
}

public typealias DecodeProgress = @convention(c) (UnsafeMutableRawPointer?, Double) -> Bool

func decodeAudio(_ path: String, dest: String, progress: DecodeProgress, ctx: UnsafeMutableRawPointer?) throws -> DecodeResult {
    let asset = AVURLAsset(url: URL(fileURLWithPath: path))
    let (tracks, duration) = try blocking {
        (try await asset.loadTracks(withMediaType: .audio), try await asset.load(.duration))
    }
    guard !tracks.isEmpty else { throw FFIError(code: "no_audio", message: "no audio track") }

    let reader: AVAssetReader
    do { reader = try AVAssetReader(asset: asset) } catch {
        throw FFIError(code: "unsupported", message: error.localizedDescription)
    }
    let settings: [String: Any] = [
        AVFormatIDKey: kAudioFormatLinearPCM,
        AVSampleRateKey: 16_000,
        AVNumberOfChannelsKey: 1,
        AVLinearPCMBitDepthKey: 32,
        AVLinearPCMIsFloatKey: true,
        AVLinearPCMIsBigEndianKey: false,
        AVLinearPCMIsNonInterleaved: false,
    ]
    // The mix output combines every audio track (e.g. separate system/mic tracks).
    let output = AVAssetReaderAudioMixOutput(audioTracks: tracks, audioSettings: settings)
    output.alwaysCopiesSampleData = false
    guard reader.canAdd(output) else { throw FFIError(code: "unsupported", message: "cannot read audio") }
    reader.add(output)
    guard reader.startReading() else {
        throw FFIError(code: "unsupported", message: reader.error?.localizedDescription ?? "cannot read audio")
    }

    FileManager.default.createFile(atPath: dest, contents: nil, attributes: [.posixPermissions: 0o600])
    guard let out = FileHandle(forWritingAtPath: dest) else {
        reader.cancelReading()
        throw FFIError.platform("cannot write \(dest)")
    }
    defer { try? out.close() }

    let total = duration.isNumeric ? duration.seconds : 0
    var samples = 0
    var lastReport = 0.0
    while let buffer = output.copyNextSampleBuffer() {
        guard let block = CMSampleBufferGetDataBuffer(buffer) else { continue }
        var length = 0
        var pointer: UnsafeMutablePointer<CChar>?
        guard CMBlockBufferGetDataPointer(block, atOffset: 0, lengthAtOffsetOut: nil, totalLengthOut: &length, dataPointerOut: &pointer) == kCMBlockBufferNoErr,
              let pointer
        else { continue }
        // Block buffers can be non-contiguous; copy the whole range when needed.
        let data: Data
        if CMBlockBufferIsRangeContiguous(block, atOffset: 0, length: length) {
            data = Data(bytes: pointer, count: length)
        } else {
            var copy = Data(count: length)
            copy.withUnsafeMutableBytes { raw in
                _ = CMBlockBufferCopyDataBytes(block, atOffset: 0, dataLength: length, destination: raw.baseAddress!)
            }
            data = copy
        }
        out.write(data)
        samples += length / 4
        let seconds = Double(samples) / 16_000
        if total > 0, seconds - lastReport >= 2 {
            lastReport = seconds
            if !progress(ctx, min(seconds / total, 1)) {
                reader.cancelReading()
                throw FFIError(code: "cancelled", message: "cancelled")
            }
        }
    }
    if reader.status == .failed {
        throw FFIError(code: "unsupported", message: reader.error?.localizedDescription ?? "decoding failed")
    }
    _ = progress(ctx, 1)
    return DecodeResult(samples: samples, durationMs: Int((Double(samples) / 16.0).rounded()))
}

@_cdecl("fl_decode_audio")
public func fl_decode_audio(
    _ path: UnsafePointer<CChar>,
    _ dest: UnsafePointer<CChar>,
    _ progress: DecodeProgress,
    _ ctx: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar> {
    let (p, d) = (String(cString: path), String(cString: dest))
    return respond { try decodeAudio(p, dest: d, progress: progress, ctx: ctx) }
}
