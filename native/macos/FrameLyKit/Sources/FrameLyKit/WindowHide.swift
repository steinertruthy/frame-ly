import AppKit

/// Pending "exit full screen, then hide" observers, keyed by window.
nonisolated(unsafe) private var exitObservers: [ObjectIdentifier: NSObjectProtocol] = [:]

/// Hides a window. A full-screen window first leaves full screen and is hidden once the
/// animation ends; hiding it in place leaves its full-screen Space empty and black.
/// Must be called on the main thread.
@_cdecl("fl_hide_window")
public func fl_hide_window(_ nsWindow: UnsafeMutableRawPointer) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    guard window.styleMask.contains(.fullScreen) else {
        window.orderOut(nil)
        return
    }
    let key = ObjectIdentifier(window)
    guard exitObservers[key] == nil else { return } // already on its way out
    exitObservers[key] = NotificationCenter.default.addObserver(
        forName: NSWindow.didExitFullScreenNotification, object: window, queue: .main
    ) { _ in
        if let token = exitObservers.removeValue(forKey: key) {
            NotificationCenter.default.removeObserver(token)
        }
        window.orderOut(nil)
    }
    window.toggleFullScreen(nil)
}
