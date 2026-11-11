import AVFoundation

struct AudioInput: Encodable {
    let id: String
    let name: String
    let isDefault: Bool
}

/// Audio capture devices (built-in, USB, Bluetooth, Continuity). Listing needs no permission.
func listAudioInputs() -> [AudioInput] {
    let session = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.microphone, .external],
        mediaType: .audio,
        position: .unspecified
    )
    let defaultId = AVCaptureDevice.default(for: .audio)?.uniqueID
    return session.devices.map {
        AudioInput(id: $0.uniqueID, name: $0.localizedName, isDefault: $0.uniqueID == defaultId)
    }
}

@_cdecl("fl_list_audio_inputs")
public func fl_list_audio_inputs() -> UnsafeMutablePointer<CChar> {
    respond { listAudioInputs() }
}
