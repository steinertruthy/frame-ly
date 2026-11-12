import AppKit

/// Pending "exit full screen, then hide" observers, keyed by window.
nonisolated(unsafe) private var exitObservers: [ObjectIdentifier: NSObjectProtocol] = [:]

/// Hides a window. Hiding a full-screen window in place leaves its Space empty and black, so
/// it has to leave full screen — but it does so invisibly (alpha 0, no window animation): the
/// user only sees the Space slide away, exactly like closing a full-screen window, and never
/// the window shrinking back to its normal size. Must be called on the main thread.
@_cdecl("fl_hide_window")
public func fl_hide_window(_ nsWindow: UnsafeMutableRawPointer) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    guard window.styleMask.contains(.fullScreen) else {
        window.orderOut(nil)
        return
    }
    let key = ObjectIdentifier(window)
    guard exitObservers[key] == nil else { return } // already on its way out
    let animation = window.animationBehavior
    window.alphaValue = 0
    window.animationBehavior = .none
    exitObservers[key] = NotificationCenter.default.addObserver(
        forName: NSWindow.didExitFullScreenNotification, object: window, queue: .main
    ) { _ in
        if let token = exitObservers.removeValue(forKey: key) {
            NotificationCenter.default.removeObserver(token)
        }
        window.orderOut(nil)
        // Restore once hidden, so the next show looks normal.
        window.animationBehavior = animation
        window.alphaValue = 1
    }
    window.toggleFullScreen(nil)
}
