import XCTest
@testable import FrameLyKit

final class FrameLyKitTests: XCTestCase {
    private func decode(_ ptr: UnsafeMutablePointer<CChar>) throws -> [String: Any] {
        defer { fl_free_string(ptr) }
        let data = Data(String(cString: ptr).utf8)
        return try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
    }

    func testDisplaysEnvelope() throws {
        let json = try decode(fl_list_displays())
        let displays = try XCTUnwrap(json["ok"] as? [[String: Any]])
        XCTAssertFalse(displays.isEmpty)
        XCTAssertEqual(displays.filter { $0["isPrimary"] as? Bool == true }.count, 1)
        XCTAssertNotNil(displays[0]["pixelWidth"])
    }

    func testPermissionsEnvelope() throws {
        let json = try decode(fl_permissions())
        let p = try XCTUnwrap(json["ok"] as? [String: String])
        XCTAssertNotNil(p["screen"])
        XCTAssertNotNil(p["microphone"])
    }

    func testAudioInputsEnvelope() throws {
        let json = try decode(fl_list_audio_inputs())
        XCTAssertNotNil(json["ok"] as? [[String: Any]])
    }

    func testErrorEnvelope() throws {
        let json = try decode(respond { () throws -> Int in throw FFIError.permissionDenied("screen") })
        let err = try XCTUnwrap(json["err"] as? [String: String])
        XCTAssertEqual(err["code"], "permission_denied")
    }
}

/// FrameLyKit's blocking APIs must run off the main thread (as they do when called from Rust).
private func offMain<T>(_ work: @escaping () -> T) -> T {
    var result: T?
    let sem = DispatchSemaphore(value: 0)
    DispatchQueue.global().async {
        result = work()
        sem.signal()
    }
    sem.wait()
    return result!
}

final class CaptureTests: XCTestCase {
    func testDisplayCaptureWhenPermitted() throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let (err, w, h): (String?, UInt32, UInt32) = offMain {
            var data: UnsafeMutablePointer<UInt8>?
            var w: UInt32 = 0, h: UInt32 = 0
            let err = fl_capture_display(CGMainDisplayID(), &data, &w, &h).map { String(cString: $0) }
            fl_free_buffer(data)
            return (err, w, h)
        }
        XCTAssertNil(err)
        XCTAssertGreaterThan(w, 0)
        XCTAssertGreaterThan(h, 0)
    }

    func testUnknownDisplayFails() throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let err: String? = offMain {
            var data: UnsafeMutablePointer<UInt8>?
            var w: UInt32 = 0, h: UInt32 = 0
            return fl_capture_display(999_999, &data, &w, &h).map { String(cString: $0) }
        }
        XCTAssertTrue(err?.contains("not found") ?? false)
    }

    func testWindowCaptureWhenPermitted() throws {
        try XCTSkipUnless(CGPreflightScreenCaptureAccess(), "needs Screen Recording permission")
        let result: (String?, UInt32, UInt32)? = offMain {
            guard let first = try? listWindows().first, let id = UInt32(first.id) else { return nil }
            var data: UnsafeMutablePointer<UInt8>?
            var w: UInt32 = 0, h: UInt32 = 0
            let err = fl_capture_window(id, &data, &w, &h).map { String(cString: $0) }
            fl_free_buffer(data)
            return (err, w, h)
        }
        let (err, w, h) = try XCTUnwrap(result, "no on-screen window to capture")
        XCTAssertNil(err)
        XCTAssertGreaterThan(w, 40)
        XCTAssertGreaterThan(h, 40)
    }
}
