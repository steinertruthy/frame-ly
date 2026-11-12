import AppKit

/// Sounds being played, kept alive until they finish (NSSound stops when deallocated).
nonisolated(unsafe) private var playing: [NSSound] = []

/// Plays a WAV held in memory. Asynchronous and fire-and-forget. FrameLy's own audio is
/// excluded from its recordings (`excludesCurrentProcessAudio`), so cues never end up in a video.
@_cdecl("fl_play_sound")
public func fl_play_sound(_ bytes: UnsafePointer<UInt8>, _ len: Int) {
    let data = Data(bytes: bytes, count: len)
    DispatchQueue.main.async {
        guard let sound = NSSound(data: data) else { return }
        playing.removeAll { !$0.isPlaying }
        playing.append(sound)
        sound.play()
    }
}
