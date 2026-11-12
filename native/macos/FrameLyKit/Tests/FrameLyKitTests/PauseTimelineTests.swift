import AVFoundation
import CoreMedia
import XCTest
@testable import FrameLyKit

final class PauseTimelineTests: XCTestCase {
    private func t(_ s: Double) -> CMTime { CMTime(seconds: s, preferredTimescale: 600) }

    func testNoPausesIsIdentity() {
        XCTAssertEqual(PauseTimeline().adjusted(t(5)), t(5))
    }

    func testDropsWhilePausedAndShiftsAfterResume() {
        var tl = PauseTimeline()
        tl.pause(at: t(10))
        XCTAssertNil(tl.adjusted(t(11)))
        tl.resume(at: t(14))
        XCTAssertEqual(tl.adjusted(t(9)), t(9), "before the pause: unchanged")
        XCTAssertNil(tl.adjusted(t(12)), "late buffer from inside the pause: dropped")
        XCTAssertEqual(tl.adjusted(t(15)), t(11), "after: shifted by 4 s")
    }

    func testMultiplePausesAccumulate() {
        var tl = PauseTimeline()
        tl.pause(at: t(2)); tl.resume(at: t(3))
        tl.pause(at: t(5)); tl.resume(at: t(7))
        XCTAssertEqual(tl.adjusted(t(8)), t(5))
        XCTAssertEqual(tl.pausedDuration(until: t(8)), t(3))
    }

    func testPausedDurationIncludesOngoingPause() {
        var tl = PauseTimeline()
        tl.pause(at: t(4))
        XCTAssertEqual(tl.pausedDuration(until: t(6)), t(2))
        XCTAssertTrue(tl.isPaused)
    }

    func testRepeatedCallsAreIdempotent() {
        var tl = PauseTimeline()
        tl.resume(at: t(1)) // resume without pause: ignored
        tl.pause(at: t(2)); tl.pause(at: t(3)) // second pause ignored
        tl.resume(at: t(4))
        XCTAssertEqual(tl.intervals.count, 1)
        XCTAssertEqual(tl.adjusted(t(5)), t(3))
    }
}

final class RecorderTests: XCTestCase {
    /// Records the main display for ~1.5 s with a pause in the middle and checks the file.
    func testRecordsPlayableMP4WhenPermitted() async throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("framely-rec-test-\(UUID())")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let out = dir.appendingPathComponent("out.mp4").path
        let thumb = dir.appendingPathComponent("thumb.jpg").path
        let json = """
        {"target":{"kind":"display","displayId":"\(CGMainDisplayID())"},"width":1280,"height":800,"fps":30,
         "bitrate":4000000,"codec":"h264","showsCursor":true,"outputPath":"\(out)","thumbnailPath":"\(thumb)"}
        """
        let result: String = await withCheckedContinuation { cont in
            DispatchQueue.global().async {
                let start = String(cString: json.withCString { fl_rec_start($0) })
                guard start.contains("\"ok\"") else { return cont.resume(returning: start) }
                Thread.sleep(forTimeInterval: 0.6)
                fl_rec_pause()
                Thread.sleep(forTimeInterval: 0.5)
                fl_rec_resume()
                Thread.sleep(forTimeInterval: 0.6)
                cont.resume(returning: String(cString: fl_rec_stop()))
            }
        }
        XCTAssertTrue(result.contains("\"ok\""), result)
        let asset = AVURLAsset(url: URL(fileURLWithPath: out))
        let duration = try await asset.load(.duration).seconds
        let tracks = try await asset.loadTracks(withMediaType: .video)
        XCTAssertEqual(tracks.count, 1)
        // ~1.2 s of recording; the 0.5 s pause must be cut out.
        XCTAssertGreaterThan(duration, 0.9)
        XCTAssertLessThan(duration, 1.6)
        XCTAssertTrue(FileManager.default.fileExists(atPath: thumb))
    }

    /// Records `target` for ~0.8 s and returns (start/stop result, output path, dir to delete).
    private func record(target: String, width: Int, height: Int) async throws -> (String, String, URL) {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("framely-rec-test-\(UUID())")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let out = dir.appendingPathComponent("out.mp4").path
        let json = """
        {"target":\(target),"width":\(width),"height":\(height),"fps":30,"bitrate":3000000,"codec":"h264",
         "showsCursor":false,"outputPath":"\(out)","thumbnailPath":"\(dir.appendingPathComponent("t.jpg").path)"}
        """
        let result: String = await withCheckedContinuation { cont in
            DispatchQueue.global().async {
                let start = String(cString: json.withCString { fl_rec_start($0) })
                guard start.contains("\"ok\"") else { return cont.resume(returning: start) }
                Thread.sleep(forTimeInterval: 0.8)
                cont.resume(returning: String(cString: fl_rec_stop()))
            }
        }
        return (result, out, dir)
    }

    private func videoSize(_ path: String) async throws -> CGSize {
        let tracks = try await AVURLAsset(url: URL(fileURLWithPath: path)).loadTracks(withMediaType: .video)
        return try await XCTUnwrap(tracks.first).load(.naturalSize)
    }

    func testRecordsRegionAtRequestedSize() async throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let target = #"{"kind":"region","displayId":"\#(CGMainDisplayID())","x":100,"y":100,"width":400,"height":300}"#
        let (result, out, dir) = try await record(target: target, width: 800, height: 600)
        defer { try? FileManager.default.removeItem(at: dir) }
        XCTAssertTrue(result.contains("\"ok\""), result)
        let size = try await videoSize(out)
        XCTAssertEqual(size, CGSize(width: 800, height: 600))
    }

    func testRecordsSingleWindow() async throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let window: WindowEntry? = await withCheckedContinuation { cont in
            DispatchQueue.global().async { cont.resume(returning: try? listWindows().first) }
        }
        let w = try XCTUnwrap(window, "no on-screen window")
        let target = #"{"kind":"window","windowId":"\#(w.id)"}"#
        let (result, out, dir) = try await record(target: target, width: 640, height: 400)
        defer { try? FileManager.default.removeItem(at: dir) }
        XCTAssertTrue(result.contains("\"ok\""), result)
        let size = try await videoSize(out)
        XCTAssertEqual(size, CGSize(width: 640, height: 400))
    }
}
