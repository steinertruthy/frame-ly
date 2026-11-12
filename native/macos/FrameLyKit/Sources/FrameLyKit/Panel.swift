import AppKit
import ObjectiveC

/// NSPanel subclass that can take keyboard focus without activating FrameLy, so opening the
/// menu-bar panel never brings the main window to the front (like a native status-item popover).
final class FrameLyPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}

/// Converts a Tauri NSWindow into a non-activating floating panel. Must run on the main thread.
@_cdecl("fl_configure_panel")
public func fl_configure_panel(_ nsWindow: UnsafeMutableRawPointer) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    object_setClass(window, FrameLyPanel.self)
    window.styleMask.insert(.nonactivatingPanel)
    if let panel = window as? NSPanel {
        panel.isFloatingPanel = true
        panel.becomesKeyOnlyIfNeeded = false
        panel.hidesOnDeactivate = false
    }
    window.level = .popUpMenu
    window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle, .transient]
    window.hasShadow = true
}

/// Shows the panel and gives it keyboard focus without activating the app.
@_cdecl("fl_show_panel")
public func fl_show_panel(_ nsWindow: UnsafeMutableRawPointer) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    window.orderFrontRegardless()
    window.makeKey()
}
