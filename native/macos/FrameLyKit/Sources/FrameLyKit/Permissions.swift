import AVFoundation
import CoreGraphics

enum PermissionState: String, Encodable {
    case granted, denied, notDetermined, restricted
}

struct PermissionsInfo: Encodable {
    let screen: PermissionState
    let microphone: PermissionState
}

/// macOS exposes no "not determined" state for Screen Recording: preflight is only true or
/// false. We remember whether we already asked, so "false + asked" reads as denied.
private let screenAskedKey = "FrameLy.screenCaptureRequested"

func screenState() -> PermissionState {
    if CGPreflightScreenCaptureAccess() { return .granted }
    return UserDefaults.standard.bool(forKey: screenAskedKey) ? .denied : .notDetermined
}

func microphoneState() -> PermissionState {
    switch AVCaptureDevice.authorizationStatus(for: .audio) {
    case .authorized: return .granted
    case .denied: return .denied
    case .restricted: return .restricted
    case .notDetermined: return .notDetermined
    @unknown default: return .notDetermined
    }
}

@_cdecl("fl_permissions")
public func fl_permissions() -> UnsafeMutablePointer<CChar> {
    respond { PermissionsInfo(screen: screenState(), microphone: microphoneState()) }
}

/// Shows the system prompt the first time; afterwards macOS only allows changing it in
/// System Settings. A newly granted permission may require relaunching the app.
@_cdecl("fl_request_screen")
public func fl_request_screen() -> UnsafeMutablePointer<CChar> {
    respond { () -> PermissionState in
        let granted = CGRequestScreenCaptureAccess()
        UserDefaults.standard.set(true, forKey: screenAskedKey)
        return granted ? .granted : screenState()
    }
}

@_cdecl("fl_request_microphone")
public func fl_request_microphone() -> UnsafeMutablePointer<CChar> {
    respond { () -> PermissionState in
        if microphoneState() != .notDetermined { return microphoneState() }
        _ = try blocking { await AVCaptureDevice.requestAccess(for: .audio) }
        return microphoneState()
    }
}
