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
