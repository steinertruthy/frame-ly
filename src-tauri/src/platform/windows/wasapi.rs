//! WASAPI capture: system audio (loopback of the default output) and microphones, delivered
//! as interleaved stereo f32 at 48 kHz with QPC timestamps (100 ns), the clock
//! Windows.Graphics.Capture uses for video frames.
//!
//! Microphones ask Windows to convert to that format (`AUTOCONVERTPCM`), falling back to the
//! device's mix format; loopback always reads the output's own mix format (any channel count,
//! 16/24/32-bit, any rate) and converts it here.

use super::win_err;
use crate::core::{
    errors::{AppError, AppResult},
    media_timeline::{resample_stereo, SAMPLE_RATE},
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};
use windows::{
    core::HSTRING,
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Media::Audio::{
            eCapture, eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator,
            MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM, AUDCLNT_STREAMFLAGS_LOOPBACK,
            AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
        },
    },
};

const WAVE_FORMAT_PCM: u16 = 1;
const WAVE_FORMAT_IEEE_FLOAT: u16 = 3;
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;
/// 100 ms of buffering in the audio engine (we poll every ~10 ms).
const BUFFER_HNS: i64 = 1_000_000;

#[derive(Clone, Copy)]
enum Sample {
    F32,
    I16,
    /// Packed 3-byte samples.
    I24,
    /// 32-bit containers (also 24-bit samples left-justified in 32 bits).
    I32,
}

/// How to turn device buffers into 48 kHz stereo f32.
#[derive(Clone, Copy)]
struct Conversion {
    channels: usize,
    rate: u32,
    sample: Sample,
}

pub struct Capture {
    client: IAudioClient,
    capture: IAudioCaptureClient,
    conversion: Option<Conversion>,
    /// Endpoint id of the device being captured.
    pub device_id: String,
    /// Friendly name and native format, for the log.
    pub name: String,
    pub format: String,
}

// SAFETY: WASAPI clients are free-threaded in the MTA; each `Capture` is used by one thread
// at a time (moved into the audio thread).
unsafe impl Send for Capture {}

fn target_format() -> WAVEFORMATEX {
    WAVEFORMATEX {
        wFormatTag: WAVE_FORMAT_IEEE_FLOAT,
        nChannels: 2,
        nSamplesPerSec: SAMPLE_RATE,
        nAvgBytesPerSec: SAMPLE_RATE * 8,
        nBlockAlign: 8,
        wBitsPerSample: 32,
        cbSize: 0,
    }
}

impl Capture {
    /// Everything the system's default output device plays (and only that device).
    pub fn default_output() -> AppResult<Capture> {
        Self::open(None, true)
    }

    /// Everything one specific output device plays.
    pub fn loopback_of(output_id: &str) -> AppResult<Capture> {
        Self::open(Some(output_id), true)
    }

    pub fn microphone(id: Option<&str>) -> AppResult<Capture> {
        Self::open(id, false)
    }

    fn open(id: Option<&str>, loopback: bool) -> AppResult<Capture> {
        // SAFETY: COM/WASAPI calls with valid arguments on this thread.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| win_err("MMDeviceEnumerator", e))?;
            let device = match (id, loopback) {
                (Some(id), _) => {
                    enumerator.GetDevice(&HSTRING::from(id)).map_err(|e| win_err("GetDevice", e))?
                }
                (None, true) => enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .map_err(|e| win_err("default output", e))?,
                (None, false) => enumerator
                    .GetDefaultAudioEndpoint(eCapture, eConsole)
                    .map_err(|e| win_err("default input", e))?,
            };
            let device_id = endpoint_id(&device);
            let name = device_name(&device);
            // Loopback streams are never converted by Windows (AUTOCONVERTPCM is accepted but
            // ignored: an 8-channel HDMI mix would be read as stereo garbage), so they always
            // use the device's own format. Microphones ask Windows to convert first.
            if !loopback {
                let client: IAudioClient =
                    device.Activate(CLSCTX_ALL, None).map_err(|e| win_err("IAudioClient", e))?;
                let wanted = target_format();
                let converted = client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                    BUFFER_HNS,
                    0,
                    &wanted,
                    None,
                );
                match converted {
                    Ok(()) => {
                        let capture: IAudioCaptureClient =
                            client.GetService().map_err(|e| win_err("IAudioCaptureClient", e))?;
                        client.Start().map_err(|e| win_err("audio Start", e))?;
                        return Ok(Capture {
                            client,
                            capture,
                            conversion: None,
                            device_id,
                            name,
                            format: "48000 Hz, 2 ch, float 32-bit (converted by Windows)".into(),
                        });
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "WASAPI auto-conversion refused; using the mix format")
                    }
                }
            }
            // A failed Initialize leaves a client unusable: always start from a fresh one.
            let client: IAudioClient =
                device.Activate(CLSCTX_ALL, None).map_err(|e| win_err("IAudioClient", e))?;
            let mix = client.GetMixFormat().map_err(|e| win_err("GetMixFormat", e))?;
            let parsed = parse_format(mix);
            let format = parsed.as_ref().map(|(_, f)| f.clone()).unwrap_or_default();
            let result = parsed.and_then(|(conversion, _)| {
                let base = if loopback { AUDCLNT_STREAMFLAGS_LOOPBACK } else { 0 };
                client
                    .Initialize(AUDCLNT_SHAREMODE_SHARED, base, BUFFER_HNS, 0, mix, None)
                    .map_err(|e| win_err("IAudioClient::Initialize", e))
                    .map(|()| conversion)
            });
            CoTaskMemFree(Some(mix as *const _));
            let conversion = result?;
            let capture: IAudioCaptureClient =
                client.GetService().map_err(|e| win_err("IAudioCaptureClient", e))?;
            client.Start().map_err(|e| win_err("audio Start", e))?;
            Ok(Capture { client, capture, conversion: Some(conversion), device_id, name, format })
        }
    }

    /// Drains every packet available now: `on_data(qpc time in 100 ns, stereo f32 at 48 kHz)`.
    /// An error means the device is gone (unplugged, disabled).
    pub fn read(&mut self, mut on_data: impl FnMut(i64, &[f32])) -> AppResult<()> {
        // SAFETY: GetBuffer/ReleaseBuffer are paired; the data pointer is read only for
        // `frames` frames of the negotiated format before ReleaseBuffer.
        unsafe {
            loop {
                let packet = self.capture.GetNextPacketSize().map_err(|e| win_err("GetNextPacketSize", e))?;
                if packet == 0 {
                    return Ok(());
                }
                let (mut data, mut frames, mut flags, mut qpc) = (std::ptr::null_mut(), 0u32, 0u32, 0u64);
                self.capture
                    .GetBuffer(&mut data, &mut frames, &mut flags, None, Some(&mut qpc))
                    .map_err(|e| win_err("GetBuffer", e))?;
                let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                let samples = match self.conversion {
                    None => {
                        if silent || data.is_null() {
                            vec![0.0; frames as usize * 2]
                        } else {
                            std::slice::from_raw_parts(data as *const f32, frames as usize * 2).to_vec()
                        }
                    }
                    Some(c) => convert(data, frames as usize, c, silent),
                };
                self.capture.ReleaseBuffer(frames).map_err(|e| win_err("ReleaseBuffer", e))?;
                if frames > 0 {
                    on_data(qpc as i64, &samples);
                }
            }
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: stopping a started client.
        let _ = unsafe { self.client.Stop() };
    }
}

unsafe fn endpoint_id(device: &windows::Win32::Media::Audio::IMMDevice) -> String {
    // SAFETY: GetId returns a CoTaskMem string we own and free.
    unsafe {
        match device.GetId() {
            Ok(p) => {
                let s = p.to_string().unwrap_or_default();
                CoTaskMemFree(Some(p.0 as *const _));
                s
            }
            Err(_) => String::new(),
        }
    }
}

/// Endpoint id of the current default output (what the system plays sound through now).
pub fn default_output_id() -> Option<String> {
    // SAFETY: COM calls with valid arguments; the id string is owned and freed in endpoint_id.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        Some(endpoint_id(&device)).filter(|id| !id.is_empty())
    }
}

unsafe fn device_name(device: &windows::Win32::Media::Audio::IMMDevice) -> String {
    // SAFETY: property store read of a live device.
    unsafe {
        device
            .OpenPropertyStore(STGM_READ)
            .ok()
            .and_then(|store| store.GetValue(&PKEY_Device_FriendlyName).ok())
            .map(|v| v.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "audio device".into())
    }
}

/// Reads a mix format: how to convert it, and a description for the log.
unsafe fn parse_format(mix: *const WAVEFORMATEX) -> AppResult<(Conversion, String)> {
    // SAFETY: `mix` comes from GetMixFormat and is valid until freed by the caller.
    unsafe {
        let f = std::ptr::read_unaligned(mix);
        let tag = f.wFormatTag;
        let bits = f.wBitsPerSample;
        let channels = f.nChannels.max(1) as usize;
        let container = f.nBlockAlign as usize / channels;
        let float =
            tag == WAVE_FORMAT_IEEE_FLOAT || (tag == WAVE_FORMAT_EXTENSIBLE && is_float_extensible(mix));
        let pcm = tag == WAVE_FORMAT_PCM || tag == WAVE_FORMAT_EXTENSIBLE;
        let sample = match (float, pcm, container) {
            (true, _, 4) => Sample::F32,
            (false, true, 2) => Sample::I16,
            (false, true, 3) => Sample::I24,
            (false, true, 4) => Sample::I32,
            _ => {
                return Err(AppError::Platform(format!(
                    "unsupported audio format (tag {tag}, {bits} bits, {channels} ch)"
                )))
            }
        };
        let rate = f.nSamplesPerSec;
        let kind = if float { "float" } else { "PCM" };
        Ok((Conversion { channels, rate, sample }, format!("{rate} Hz, {channels} ch, {kind} {bits}-bit")))
    }
}

/// WAVEFORMATEXTENSIBLE whose SubFormat is IEEE float.
unsafe fn is_float_extensible(f: *const WAVEFORMATEX) -> bool {
    // SubFormat GUID sits 24 bytes into WAVEFORMATEXTENSIBLE (after WAVEFORMATEX + 8 bytes).
    const IEEE_FLOAT: [u8; 4] = [3, 0, 0, 0];
    // SAFETY: the caller passes an EXTENSIBLE format, which is at least 40 bytes long.
    unsafe {
        let p = (f as *const u8).add(24);
        std::slice::from_raw_parts(p, 4) == IEEE_FLOAT
    }
}

/// Device format → interleaved stereo f32 at 48 kHz. Multichannel layouts (5.1, 7.1: what
/// HDMI outputs usually run at) keep front left/right and fold the center channel into both.
unsafe fn convert(data: *const u8, frames: usize, c: Conversion, silent: bool) -> Vec<f32> {
    let mut stereo = vec![0.0f32; frames * 2];
    if !silent && !data.is_null() {
        // SAFETY: every index is `< frames * channels`, inside the packet returned by GetBuffer.
        let get = |i: usize| -> f32 {
            unsafe {
                match c.sample {
                    Sample::F32 => (data as *const f32).add(i).read_unaligned(),
                    Sample::I16 => (data as *const i16).add(i).read_unaligned() as f32 / 32_768.0,
                    Sample::I24 => {
                        let p = data.add(i * 3);
                        i32::from_le_bytes([0, *p, *p.add(1), *p.add(2)]) as f32 / 2_147_483_648.0
                    }
                    Sample::I32 => (data as *const i32).add(i).read_unaligned() as f32 / 2_147_483_648.0,
                }
            }
        };
        let ch = c.channels;
        for f in 0..frames {
            let base = f * ch;
            let (mut l, mut r) = if ch == 1 {
                let m = get(base);
                (m, m)
            } else {
                (get(base), get(base + 1))
            };
            // Standard order FL, FR, FC, LFE, …: 5.1 and up carry dialogue in the center.
            if ch >= 6 {
                let center = get(base + 2) * std::f32::consts::FRAC_1_SQRT_2;
                l += center;
                r += center;
            }
            stereo[f * 2] = l;
            stereo[f * 2 + 1] = r;
        }
    }
    resample_stereo(&stereo, c.rate, SAMPLE_RATE)
}

/// Live microphone level for the Settings "test your microphone" meter.
pub struct MicMonitor {
    stop: Arc<AtomicBool>,
    level: Arc<AtomicU32>,
    thread: Option<JoinHandle<()>>,
}

impl MicMonitor {
    pub fn start(id: Option<&str>) -> AppResult<MicMonitor> {
        let id = id.map(str::to_owned);
        let stop = Arc::new(AtomicBool::new(false));
        let level = Arc::new(AtomicU32::new(0));
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (s, l) = (Arc::clone(&stop), Arc::clone(&level));
        let thread = std::thread::Builder::new()
            .name("framely-mic-test".into())
            .spawn(move || {
                let mut capture = match Capture::microphone(id.as_deref()) {
                    Ok(c) => {
                        let _ = ready_tx.send(Ok(()));
                        c
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let mut peak = 0.0f32;
                while !s.load(Ordering::Relaxed) {
                    // Decay like a VU meter between buffers.
                    peak *= 0.85;
                    if capture
                        .read(|_, samples| {
                            peak = samples.iter().fold(peak, |m, x| m.max(x.abs()));
                        })
                        .is_err()
                    {
                        break;
                    }
                    l.store(peak.min(1.0).to_bits(), Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(30));
                }
                l.store(0, Ordering::Relaxed);
            })
            .map_err(|e| AppError::Runtime(format!("mic test thread: {e}")))?;
        ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| AppError::Platform("microphone did not start".into()))??;
        Ok(MicMonitor { stop, level, thread: Some(thread) })
    }

    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

impl Drop for MicMonitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
