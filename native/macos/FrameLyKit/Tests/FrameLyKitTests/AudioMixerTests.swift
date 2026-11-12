import CoreMedia
import XCTest
@testable import FrameLyKit

final class AudioMixerTests: XCTestCase {
    private let sr = Double(AudioMixer.sampleRate)
    private func t(_ s: Double) -> CMTime { CMTime(seconds: s, preferredTimescale: 48_000) }
    private func tone(_ frames: Int, _ v: Float) -> [Float] { [Float](repeating: v, count: frames * 2) }

    func testSingleSourcePassesThroughAtItsTimestamp() throws {
        let m = AudioMixer(origin: t(100), gains: [1])
        m.push(source: 0, pts: t(100.5), samples: tone(480, 0.25))
        let out = try XCTUnwrap(m.pull(flush: true))
        XCTAssertEqual(out.startFrame, 0)
        XCTAssertEqual(out.samples.count, (24_000 + 480) * 2)
        XCTAssertEqual(out.samples[0], 0, "silence before the first buffer")
        XCTAssertEqual(out.samples[24_000 * 2], 0.25, accuracy: 1e-6, "buffer lands at 0.5 s")
    }

    func testMixesSourcesWithGains() throws {
        let m = AudioMixer(origin: t(0), gains: [1, 0.5])
        m.push(source: 0, pts: t(0), samples: tone(100, 0.2))
        m.push(source: 1, pts: t(0), samples: tone(100, 0.4))
        let out = try XCTUnwrap(m.pull())
        XCTAssertEqual(out.samples.count, 200)
        XCTAssertEqual(out.samples[10], 0.4, accuracy: 1e-6) // 0.2 + 0.4 × 0.5
    }

    func testWaitsForSlowerSourceUpToTheLatencyLimit() {
        let m = AudioMixer(origin: t(0), gains: [1, 1], latency: 0.25)
        m.push(source: 0, pts: t(0), samples: tone(4_800, 0.1)) // 100 ms
        m.push(source: 1, pts: t(0), samples: tone(480, 0.1)) //  10 ms
        XCTAssertEqual(m.pull()?.samples.count, 480 * 2, "only up to the slower source")
        // Source 1 goes quiet; source 0 runs ahead: release lags at most 250 ms behind.
        m.push(source: 0, pts: t(0.1), samples: tone(48_000, 0.1)) // up to 1.1 s
        let out = m.pull()
        XCTAssertEqual(m.readIndex, Int64(1.1 * sr) - 12_000)
        XCTAssertNotNil(out)
    }

    func testDeactivatedSourceNoLongerHoldsOutput() {
        let m = AudioMixer(origin: t(0), gains: [1, 1])
        m.push(source: 0, pts: t(0), samples: tone(4_800, 0.1))
        m.push(source: 1, pts: t(0), samples: tone(480, 0.1))
        m.deactivate(source: 1)
        XCTAssertEqual(m.pull()?.samples.count, 4_800 * 2)
    }

    func testJitterSnapsButDriftResyncs() throws {
        let m = AudioMixer(origin: t(0), gains: [1], snap: 0.010)
        m.push(source: 0, pts: t(0), samples: tone(480, 0.1))
        // Next buffer's timestamp is 3 ms late (jitter): glued contiguously.
        m.push(source: 0, pts: t(0.013), samples: tone(480, 0.1))
        XCTAssertEqual(m.sources[0].writtenUntil, 960)
        // 50 ms jump (drift past the window / dropout): placed at its timestamp, gap is silence.
        m.push(source: 0, pts: t(0.07), samples: tone(480, 0.1))
        XCTAssertEqual(m.sources[0].writtenUntil, 3_360 + 480)
        let out = try XCTUnwrap(m.pull(flush: true))
        XCTAssertEqual(out.samples[1_000 * 2], 0, "gap is silent")
    }

    func testLateBuffersAreDroppedNotReordered() {
        let m = AudioMixer(origin: t(0), gains: [1])
        m.push(source: 0, pts: t(0), samples: tone(4_800, 0.1))
        _ = m.pull()
        m.push(source: 0, pts: t(0.05), samples: tone(480, 0.1)) // entirely in the past… but within snap? no: 50 ms back
        XCTAssertGreaterThan(m.droppedLateFrames, 0)
    }

    func testSoftClipNeverExceedsUnity() {
        XCTAssertEqual(AudioMixer.softClip(0.5), 0.5)
        XCTAssertLessThanOrEqual(AudioMixer.softClip(3), 1)
        XCTAssertGreaterThan(AudioMixer.softClip(0.95), 0.8)
        XCTAssertEqual(AudioMixer.softClip(-3), -AudioMixer.softClip(3))
    }

    func testResampleChangesLengthAndKeepsLevel() {
        let out = PCM.resample(tone(441, 0.3), from: 44_100, to: 48_000)
        XCTAssertEqual(out.count / 2, 480)
        XCTAssertEqual(out[100], 0.3, accuracy: 1e-6)
    }

    func testSampleBufferRoundTrip() throws {
        let samples = (0..<960).map { Float($0 % 7) / 10 }
        let sb = try XCTUnwrap(PCM.sampleBuffer(samples, pts: t(1)))
        XCTAssertEqual(sb.numSamples, 480)
        XCTAssertEqual(sb.presentationTimeStamp, t(1))
        XCTAssertEqual(try XCTUnwrap(PCM.interleavedStereo(sb)), samples)
    }
}
