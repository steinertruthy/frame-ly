import AppKit
import CoreGraphics

struct DisplayInfo: Encodable {
    struct Bounds: Encodable { let x, y, width, height: Double }
    let id: String
    let name: String
    let bounds: Bounds
    let pixelWidth: Int
    let pixelHeight: Int
    let scaleFactor: Double
    let refreshRate: Double?
    let isPrimary: Bool
}

/// Active displays via CoreGraphics (no permission needed). `bounds` are global points with a
/// top-left origin — the same space ScreenCaptureKit and window frames use.
func listDisplays() throws -> [DisplayInfo] {
    var count: UInt32 = 0
    guard CGGetActiveDisplayList(0, nil, &count) == .success else {
        throw FFIError.platform("CGGetActiveDisplayList failed")
    }
    var ids = [CGDirectDisplayID](repeating: 0, count: Int(count))
    guard CGGetActiveDisplayList(count, &ids, &count) == .success else {
        throw FFIError.platform("CGGetActiveDisplayList failed")
    }

    // NSScreen gives localized names ("Built-in Retina Display", "LG UltraFine"); it lives on
    // the main thread. If that isn't reachable we fall back to "Display N".
    let names: [CGDirectDisplayID: String] = onMain {
        var map: [CGDirectDisplayID: String] = [:]
        for screen in NSScreen.screens {
            if let n = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber {
                map[n.uint32Value] = screen.localizedName
            }
        }
        return map
    } ?? [:]

    return ids.prefix(Int(count)).enumerated().map { index, id in
        let b = CGDisplayBounds(id)
        let mode = CGDisplayCopyDisplayMode(id)
        let pw = mode?.pixelWidth ?? Int(b.width)
        let ph = mode?.pixelHeight ?? Int(b.height)
        let hz = mode?.refreshRate ?? 0
        return DisplayInfo(
            id: String(id),
            name: names[id] ?? "Display \(index + 1)",
            bounds: .init(x: b.origin.x, y: b.origin.y, width: b.width, height: b.height),
            pixelWidth: pw,
            pixelHeight: ph,
            scaleFactor: b.width > 0 ? Double(pw) / b.width : 1,
            refreshRate: hz > 0 ? hz : nil,
            isPrimary: CGDisplayIsMain(id) != 0
        )
    }
}

@_cdecl("fl_list_displays")
public func fl_list_displays() -> UnsafeMutablePointer<CChar> {
    respond { try listDisplays() }
}
