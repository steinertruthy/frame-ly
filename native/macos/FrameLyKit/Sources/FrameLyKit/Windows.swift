import AppKit
import ScreenCaptureKit

struct WindowEntry: Encodable {
    struct Bounds: Encodable { let x, y, width, height: Double }
    let id: String
    let title: String
    let appName: String
    let appId: String?
    let pid: Int32
    let bounds: Bounds
}

/// System surfaces that are technically windows but never what a user means by "a window".
private let excludedBundles: Set<String> = [
    "com.apple.dock",
    "com.apple.controlcenter",
    "com.apple.WindowManager",
    "com.apple.notificationcenterui",
    "com.apple.Spotlight",
    "com.apple.systemuiserver",
    "com.apple.wallpaper.agent",
]

/// On-screen, normal-level application windows via ScreenCaptureKit, front-to-back.
/// Requires Screen Recording permission; checked first so this never triggers the OS prompt.
func listWindows() throws -> [WindowEntry] {
    guard CGPreflightScreenCaptureAccess() else { throw FFIError.permissionDenied("screen") }
    let content = try blocking {
        try await SCShareableContent.excludingDesktopWindows(true, onScreenWindowsOnly: true)
    }
    let ownPid = ProcessInfo.processInfo.processIdentifier
    return content.windows.compactMap { w -> WindowEntry? in
        guard let app = w.owningApplication,
              app.processID != ownPid,
              w.windowLayer == 0,
              w.isOnScreen,
              w.frame.width >= 40, w.frame.height >= 40,
              !excludedBundles.contains(app.bundleIdentifier)
        else { return nil }
        let title = w.title ?? ""
        return WindowEntry(
            id: String(w.windowID),
            title: title.isEmpty ? app.applicationName : title,
            appName: app.applicationName,
            appId: app.bundleIdentifier.isEmpty ? nil : app.bundleIdentifier,
            pid: app.processID,
            bounds: .init(x: w.frame.origin.x, y: w.frame.origin.y, width: w.frame.width, height: w.frame.height)
        )
    }
}

@_cdecl("fl_list_windows")
public func fl_list_windows() -> UnsafeMutablePointer<CChar> {
    respond { try listWindows() }
}
