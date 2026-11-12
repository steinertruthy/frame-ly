import AVFoundation
import CoreAudio
import XCTest

/// Full hardware output latency of the default output device, in seconds:
/// device latency + safety offset + I/O buffer + stream latency (what Core Audio adds between
/// rendering a sample and it reaching the speaker).
func defaultOutputLatency() -> Double {
    func prop<T>(_ obj: AudioObjectID, _ sel: AudioObjectPropertySelector, _ scope: AudioObjectPropertyScope, _ v: inout T) {
        var addr = AudioObjectPropertyAddress(mSelector: sel, mScope: scope, mElement: kAudioObjectPropertyElementMain)
        var size = UInt32(MemoryLayout<T>.size)
        AudioObjectGetPropertyData(obj, &addr, 0, nil, &size, &v)
    }
    var device = AudioObjectID(0)
    prop(AudioObjectID(kAudioObjectSystemObject), kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectPropertyScopeGlobal, &device)
    var latency: UInt32 = 0, safety: UInt32 = 0, bufferFrames: UInt32 = 0, rate: Float64 = 48_000
    prop(device, kAudioDevicePropertyLatency, kAudioObjectPropertyScopeOutput, &latency)
    prop(device, kAudioDevicePropertySafetyOffset, kAudioObjectPropertyScopeOutput, &safety)
    prop(device, kAudioDevicePropertyBufferFrameSize, kAudioObjectPropertyScopeGlobal, &bufferFrames)
    prop(device, kAudioDevicePropertyNominalSampleRate, kAudioObjectPropertyScopeGlobal, &rate)
    var streamLatency: UInt32 = 0
    var streams = [AudioStreamID](repeating: 0, count: 8)
    var addr = AudioObjectPropertyAddress(mSelector: kAudioDevicePropertyStreams, mScope: kAudioObjectPropertyScopeOutput, mElement: kAudioObjectPropertyElementMain)
    var size = UInt32(MemoryLayout<AudioStreamID>.size * streams.count)
    if AudioObjectGetPropertyData(device, &addr, 0, nil, &size, &streams) == noErr, size > 0 {
        prop(streams[0], kAudioStreamPropertyLatency, kAudioObjectPropertyScopeGlobal, &streamLatency)
    }
    return Double(latency + safety + bufferFrames + streamLatency) / rate
}
@testable import FrameLyKit

/// End-to-end A/V sync: a tone scheduled at an exact host time must appear in the recorded
/// audio track at that same moment of the video timeline.
final class AVSyncTests: XCTestCase {
    func testSystemAudioIsInSyncWithVideo() async throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("framely-sync-\(UUID())")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let out = dir.appendingPathComponent("sync.mp4").path
        let json = """
        {"target":{"kind":"display","displayId":"\(CGMainDisplayID())"},
         "audio":{"system":true,"microphone":false,"systemGain":1,"microphoneGain":1},
         "width":640,"height":400,"fps":30,"bitrate":2000000,"codec":"h264","showsCursor":false,
         "outputPath":"\(out)","thumbnailPath":"\(dir.appendingPathComponent("t.jpg").path)"}
        """
        let config = try JSONDecoder().decode(RecordingConfig.self, from: Data(json.utf8))

        // 1 kHz tone, 300 ms, scheduled through the engine at an exact host time.
        let engine = AVAudioEngine()
        let player = AVAudioPlayerNode()
        let format = AVAudioFormat(standardFormatWithSampleRate: 48_000, channels: 2)!
        engine.attach(player)
        engine.connect(player, to: engine.mainMixerNode, format: format)
        let tone = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: 14_400)!
        tone.frameLength = 14_400
        for c in 0..<2 {
            for i in 0..<14_400 { tone.floatChannelData![c][i] = 0.5 * sin(2 * .pi * 1000 * Float(i) / 48_000) }
        }
        try engine.start()

        let (origin, scheduledHost, stopResult): (CMTime?, CMTime, String) = await withCheckedContinuation { cont in
            DispatchQueue.global().async {
                do {
                    let rec = try Recorder(config: config, emit: { _ in })
                    rec.excludesOwnAudio = false
                    try rec.start()
                    Thread.sleep(forTimeInterval: 0.7)
                    let now = CMClockGetTime(CMClockGetHostTimeClock())
                    let at = now + CMTime(seconds: 0.3, preferredTimescale: 1_000_000)
                    let hostTicks = AVAudioTime.hostTime(forSeconds: at.seconds)
                    player.scheduleBuffer(tone, at: AVAudioTime(hostTime: hostTicks))
                    player.play()
                    Thread.sleep(forTimeInterval: 1.2)
                    let r = try rec.stop()
                    cont.resume(returning: (rec.originHostTime, at, "ok \(r.durationMs)"))
                } catch {
                    cont.resume(returning: (nil, .zero, "error \(error)"))
                }
            }
        }
        engine.stop()
        XCTAssertTrue(stopResult.hasPrefix("ok"), stopResult)
        let origin0 = try XCTUnwrap(origin)

        // Output latency between the scheduled time and the sound actually leaving the mixer.
        // ScreenCaptureKit timestamps system audio when it reaches the hardware, i.e. when the
        // user hears it — the right reference for sync with what they see.
        let latency = engine.outputNode.presentationLatency + defaultOutputLatency()
        let expected = (scheduledHost - origin0).seconds + latency
        let onset = try await firstLoudSample(in: out)
        let error = (onset - expected) * 1000
        print("latency term: \(String(format: "%.1f", latency * 1000)) ms, device: \(engine.outputNode.outputFormat(forBus: 0).sampleRate) Hz")
        print("A/V sync: expected \(String(format: "%.1f", expected * 1000)) ms, onset \(String(format: "%.1f", onset * 1000)) ms, error \(String(format: "%+.1f", error)) ms")
        XCTAssertLessThan(abs(error), 15, "audio must line up with video within 15 ms")
    }

    /// Seconds from the start of the file to the first sample above -20 dBFS.
    private func firstLoudSample(in path: String) async throws -> Double {
        let asset = AVURLAsset(url: URL(fileURLWithPath: path))
        let tracks = try await asset.loadTracks(withMediaType: .audio)
        let track = try XCTUnwrap(tracks.first, "no audio track")
        let reader = try AVAssetReader(asset: asset)
        let output = AVAssetReaderTrackOutput(track: track, outputSettings: [
            AVFormatIDKey: kAudioFormatLinearPCM, AVLinearPCMIsFloatKey: true, AVLinearPCMBitDepthKey: 32,
            AVLinearPCMIsNonInterleaved: false, AVSampleRateKey: 48_000, AVNumberOfChannelsKey: 2,
        ])
        reader.add(output)
        reader.startReading()
        while let sb = output.copyNextSampleBuffer() {
            guard let samples = PCM.interleavedStereo(sb) else { continue }
            if let i = samples.firstIndex(where: { abs($0) > 0.1 }) {
                return sb.presentationTimeStamp.seconds + Double(i / 2) / 48_000
            }
        }
        XCTFail("tone not found in the recording")
        return .nan
    }
}
