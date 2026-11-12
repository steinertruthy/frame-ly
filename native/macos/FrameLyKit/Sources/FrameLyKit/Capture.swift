import AppKit
import ScreenCaptureKit

// Screenshots via ScreenCaptureKit. Pixels are returned to Rust as tightly packed BGRA8 in a
// malloc'd buffer that Rust copies and then releases with `fl_free_buffer`.
//
// Signature of the capture exports:
//   (id, outData, outWidth, outHeight) -> error JSON or NULL on success

private func shareableContent() throws -> SCShareableContent {
    guard CGPreflightScreenCaptureAccess() else { throw FFIError.permissionDenied("screen") }
    return try blocking { try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true) }
}

/// Renders a CGImage into an sRGB, little-endian premultiplied-first (= BGRA) buffer.
private func bgraBuffer(_ image: CGImage) throws -> (UnsafeMutablePointer<UInt8>, Int, Int) {
    let (w, h) = (image.width, image.height)
    guard w > 0, h > 0, let data = malloc(w * h * 4)?.assumingMemoryBound(to: UInt8.self) else {
        throw FFIError.platform("empty image")
    }
    let info = CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue
    guard let ctx = CGContext(
        data: data, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4,
        space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: info
    ) else {
        free(data)
        throw FFIError.platform("CGContext creation failed")
    }
    ctx.interpolationQuality = .none
    ctx.draw(image, in: CGRect(x: 0, y: 0, width: w, height: h))
    return (data, w, h)
}

private func export(
    _ body: () throws -> CGImage,
    _ outData: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>,
    _ outW: UnsafeMutablePointer<UInt32>,
    _ outH: UnsafeMutablePointer<UInt32>
) -> UnsafeMutablePointer<CChar>? {
    do {
        let (data, w, h) = try bgraBuffer(try body())
        outData.pointee = data
        outW.pointee = UInt32(w)
        outH.pointee = UInt32(h)
        return nil
    } catch {
        return respond { () throws -> Int in throw error }
    }
}

func captureDisplay(_ displayId: CGDirectDisplayID) throws -> CGImage {
    let content = try shareableContent()
    guard let display = content.displays.first(where: { $0.displayID == displayId }) else {
        throw FFIError.platform("display \(displayId) not found")
    }
    // Never include FrameLy's own windows (main window, overlays, preview).
    let ownPid = ProcessInfo.processInfo.processIdentifier
    let ownApps = content.applications.filter { $0.processID == ownPid }
    let filter = SCContentFilter(display: display, excludingApplications: ownApps, exceptingWindows: [])
    let config = SCStreamConfiguration()
    let scale = filter.pointPixelScale
    config.width = Int((filter.contentRect.width * CGFloat(scale)).rounded())
    config.height = Int((filter.contentRect.height * CGFloat(scale)).rounded())
    config.showsCursor = false
    config.captureResolution = .best
    return try blocking { try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config) }
}

func captureWindow(_ windowId: CGWindowID) throws -> CGImage {
    let content = try shareableContent()
    guard let window = content.windows.first(where: { $0.windowID == windowId }) else {
        throw FFIError.platform("window \(windowId) not found")
    }
    let filter = SCContentFilter(desktopIndependentWindow: window)
    let config = SCStreamConfiguration()
    let scale = filter.pointPixelScale
    config.width = Int((filter.contentRect.width * CGFloat(scale)).rounded())
    config.height = Int((filter.contentRect.height * CGFloat(scale)).rounded())
    config.showsCursor = false
    config.captureResolution = .best
    config.ignoreShadowsSingleWindow = true
    return try blocking { try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config) }
}

@_cdecl("fl_capture_display")
public func fl_capture_display(
    _ displayId: UInt32,
    _ outData: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>,
    _ outW: UnsafeMutablePointer<UInt32>,
    _ outH: UnsafeMutablePointer<UInt32>
) -> UnsafeMutablePointer<CChar>? {
    export({ try captureDisplay(displayId) }, outData, outW, outH)
}

@_cdecl("fl_capture_window")
public func fl_capture_window(
    _ windowId: UInt32,
    _ outData: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>,
    _ outW: UnsafeMutablePointer<UInt32>,
    _ outH: UnsafeMutablePointer<UInt32>
) -> UnsafeMutablePointer<CChar>? {
    export({ try captureWindow(windowId) }, outData, outW, outH)
}

@_cdecl("fl_free_buffer")
public func fl_free_buffer(_ ptr: UnsafeMutablePointer<UInt8>?) {
    free(ptr)
}

/// Display currently containing the mouse pointer (CoreGraphics global coordinates).
@_cdecl("fl_display_at_cursor")
public func fl_display_at_cursor() -> UInt32 {
    let location = CGEvent(source: nil)?.location ?? .zero
    var id: CGDirectDisplayID = 0
    var count: UInt32 = 0
    guard CGGetDisplaysWithPoint(location, 1, &id, &count) == .success, count > 0 else { return CGMainDisplayID() }
    return id
}

/// Turns a Tauri window into a selection overlay: covers the whole display including the
/// menu bar and Dock, floats above everything, follows the user across Spaces.
/// Must run on the main thread.
@_cdecl("fl_configure_overlay")
public func fl_configure_overlay(_ nsWindow: UnsafeMutableRawPointer, _ displayId: UInt32) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    window.level = .screenSaver
    window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary, .ignoresCycle]
    window.hasShadow = false
    window.isMovable = false
    if let screen = NSScreen.screens.first(where: {
        ($0.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value == displayId
    }) {
        window.setFrame(screen.frame, display: true)
    }
}

/// Floating panel style for the post-capture preview: above normal windows, on every Space.
@_cdecl("fl_configure_floating")
public func fl_configure_floating(_ nsWindow: UnsafeMutableRawPointer) {
    let window = Unmanaged<NSWindow>.fromOpaque(nsWindow).takeUnretainedValue()
    window.level = .floating
    window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle]
}

/// Puts already-encoded PNG bytes on the general pasteboard (no re-encoding).
@_cdecl("fl_copy_png")
public func fl_copy_png(_ bytes: UnsafePointer<UInt8>, _ len: Int) -> Bool {
    let data = Data(bytes: bytes, count: len)
    let pb = NSPasteboard.general
    pb.clearContents()
    return pb.setData(data, forType: .png)
}
