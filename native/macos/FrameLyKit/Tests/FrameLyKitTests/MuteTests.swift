import AVFoundation
import ScreenCaptureKit
import XCTest
@testable import FrameLyKit

/// Muting a source during a recording silences it from then on, without stopping or
/// shifting the recording; unmuting brings it back.
final class MuteTests: XCTestCase {
    func testMutingSystemAudioSilencesItUntilUnmuted() async throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("framely-mute-\(UUID())")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let out = dir.appendingPathComponent("mute.mp4").path
        let json = """
        {"target":{"kind":"display","displayId":"\(CGMainDisplayID())"},
         "audio":{"system":true,"microphone":false,"systemGain":1,"microphoneGain":1},
         "width":640,"height":400,"fps":30,"bitrate":2000000,"codec":"h264","showsCursor":false,
         "outputPath":"\(out)","thumbnailPath":"\(dir.appendingPathComponent("t.jpg").path)"}
        """
        let config = try JSONDecoder().decode(RecordingConfig.self, from: Data(json.utf8))

        // A continuous 1 kHz tone for the whole recording.
        let engine = AVAudioEngine()
        let player = AVAudioPlayerNode()
        let format = AVAudioFormat(standardFormatWithSampleRate: 48_000, channels: 2)!
        engine.attach(player)
        engine.connect(player, to: engine.mainMixerNode, format: format)
        let tone = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: 48_000)!
        tone.frameLength = 48_000
        for c in 0..<2 {
            for i in 0..<48_000 { tone.floatChannelData![c][i] = 0.4 * sin(2 * .pi * 1000 * Float(i) / 48_000) }
        }
        try engine.start()
        player.scheduleBuffer(tone, at: nil, options: .loops, completionHandler: nil)
        player.play()

        let result: String = await withCheckedContinuation { cont in
            DispatchQueue.global().async {
                do {
                    let rec = try Recorder(config: config, emit: { _ in })
                    rec.excludesOwnAudio = false
                    try rec.start()
                    Thread.sleep(forTimeInterval: 1.2) // 0…1.2 s: audible
                    rec.setMuted(system: true, microphone: false)
                    Thread.sleep(forTimeInterval: 1.2) // 1.2…2.4 s: muted
                    rec.setMuted(system: false, microphone: false)
                    Thread.sleep(forTimeInterval: 1.0) // 2.4…3.4 s: audible again
                    _ = try rec.stop()
                    cont.resume(returning: "ok")
                } catch {
                    cont.resume(returning: "error \(error)")
                }
            }
        }
        player.stop()
        engine.stop()
        XCTAssertEqual(result, "ok")

        let rms = try await windowRMS(in: out)
        let level = { (from: Double, to: Double) in
            let w = rms.filter { $0.time >= from && $0.time < to }.map(\.rms)
            return w.isEmpty ? 0 : w.reduce(0, +) / Float(w.count)
        }
        let before = level(0.4, 1.0), muted = level(1.5, 2.2), after = level(2.7, 3.2)
        print(String(format: "RMS before %.3f, muted %.4f, after %.3f", before, muted, after))
        XCTAssertGreaterThan(before, 0.05, "tone audible before muting")
        XCTAssertLessThan(muted, 0.005, "silent while muted")
        XCTAssertGreaterThan(after, 0.05, "tone back after unmuting")
    }

    /// RMS of the audio track in 100 ms windows: (start time, rms).
    private func windowRMS(in path: String) async throws -> [(time: Double, rms: Float)] {
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
        var all: [Float] = []
        var start: Double?
        while let sb = output.copyNextSampleBuffer() {
            guard let samples = PCM.interleavedStereo(sb) else { continue }
            if start == nil { start = sb.presentationTimeStamp.seconds }
            all.append(contentsOf: samples)
        }
        let window = 4_800 * 2
        return stride(from: 0, to: all.count - window, by: window).map { i in
            let slice = all[i..<i + window]
            let rms = (slice.reduce(0) { $0 + $1 * $1 } / Float(window)).squareRoot()
            return ((start ?? 0) + Double(i / 2) / 48_000, rms)
        }
    }
}
