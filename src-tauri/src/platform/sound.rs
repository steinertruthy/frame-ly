//! Plays a WAV from memory without blocking.
//! macOS: NSSound (FrameLyKit). Windows: `PlaySoundW` with `SND_MEMORY | SND_ASYNC`.

#[cfg(target_os = "macos")]
pub fn play_wav(wav: &'static [u8]) {
    extern "C" {
        fn fl_play_sound(bytes: *const u8, len: usize);
    }
    // SAFETY: pointer/len describe a live slice; Swift copies it before returning.
    unsafe { fl_play_sound(wav.as_ptr(), wav.len()) }
}

#[cfg(windows)]
pub fn play_wav(wav: &'static [u8]) {
    use windows::{
        core::PCWSTR,
        Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT},
    };
    // SAFETY: with SND_MEMORY the "name" is the WAV image; it is 'static, so it outlives the
    // asynchronous playback.
    let ok = unsafe { PlaySoundW(PCWSTR(wav.as_ptr().cast()), None, SND_MEMORY | SND_ASYNC | SND_NODEFAULT) };
    if !ok.as_bool() {
        tracing::debug!("PlaySoundW failed");
    }
}
