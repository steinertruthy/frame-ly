import AppKit
import XCTest
@testable import FrameLyKit

final class MediaTests: XCTestCase {
    private func offMain<T>(_ work: @escaping () throws -> T) throws -> T {
        var result: Result<T, Error>!
        let sem = DispatchSemaphore(value: 0)
        DispatchQueue.global().async {
            result = Result { try work() }
            sem.signal()
        }
        sem.wait()
        return try result.get()
    }

    func testImageThumbnailIsBoundedAndJPEG() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("framely-media-\(UUID())")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }
        let src = dir.appendingPathComponent("big.png")
        let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 2000, pixelsHigh: 1000, bitsPerSample: 8, samplesPerPixel: 4,
                                   hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
        try rep.representation(using: .png, properties: [:])!.write(to: src)
        let dest = dir.appendingPathComponent("t.jpg").path
        try offMain { try makeThumbnail(src.path, dest: dest, maxPixels: 400) }
        let thumb = try XCTUnwrap(NSImage(contentsOfFile: dest)?.representations.first)
        XCTAssertEqual(thumb.pixelsWide, 400)
        XCTAssertEqual(thumb.pixelsHigh, 200)
    }

    func testMissingFileFails() {
        XCTAssertThrowsError(try offMain { try makeThumbnail("/nonexistent.png", dest: "/tmp/x.jpg", maxPixels: 100) })
    }
}
