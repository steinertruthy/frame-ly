//! WASAPI capture: system audio (loopback of the default output) and microphones, delivered
//! as interleaved stereo f32 at 48 kHz with QPC timestamps (100 ns), the clock
//! Windows.Graphics.Capture uses for video frames.
//!
//! Shared-mode streams ask Windows to convert to that format (`AUTOCONVERTPCM`); when a driver
//! refuses, the device's mix format is converted here instead.

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
            AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, DEVICE_STATE_ACTIVE, WAVEFORMATEX,
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
    /// Everything one specific output device plays.
    pub fn loopback_of(output_id: &str) -> AppResult<Capture> {
        Self::open(Some(output_id), true)
    }

    pub fn microphone(id: Option<&str>) -> AppResult<Capture> {
        Self::open(id, false)
    }

    fn open(id: Option<&str>, loopback: bool) -> AppResult<Capture> {
        // SAFETY: COM/WASAPI calls with valid arguments on this thread; the mix format pointer
        // is read then freed with CoTaskMemFree.
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
            let client: IAudioClient =
                device.Activate(CLSCTX_ALL, None).map_err(|e| win_err("IAudioClient", e))?;
            let base = if loopback { AUDCLNT_STREAMFLAGS_LOOPBACK } else { 0 };
            let wanted = target_format();
            let converted = client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                base | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                BUFFER_HNS,
                0,
                &wanted,
                None,
            );
            let conversion = match converted {
                Ok(()) => None,
                Err(e) => {
                    tracing::debug!(error = %e, loopback, "WASAPI auto-conversion refused; using the mix format");
                    // A failed Initialize leaves the client unusable: start over.
                    let client2: IAudioClient =
                        device.Activate(CLSCTX_ALL, None).map_err(|e| win_err("IAudioClient", e))?;
                    let mix = client2.GetMixFormat().map_err(|e| win_err("GetMixFormat", e))?;
                    let f = std::ptr::read_unaligned(mix);
                    let tag = f.wFormatTag;
                    let bits = f.wBitsPerSample;
                    let float = tag == WAVE_FORMAT_IEEE_FLOAT
                        || (tag == WAVE_FORMAT_EXTENSIBLE && is_float_extensible(mix));
                    let pcm = tag == WAVE_FORMAT_PCM || tag == WAVE_FORMAT_EXTENSIBLE;
                    let sample = match (float, pcm, bits) {
                        (true, _, 32) => Sample::F32,
                        (false, true, 32) => Sample::I32,
                        (false, true, 16) => Sample::I16,
                        _ => {
                            CoTaskMemFree(Some(mix as *const _));
                            return Err(AppError::Platform(format!(
                                "unsupported audio format (tag {tag}, {bits} bits)"
                            )));
                        }
                    };
                    let conversion =
                        Conversion { channels: f.nChannels.max(1) as usize, rate: f.nSamplesPerSec, sample };
                    let r = client2.Initialize(AUDCLNT_SHAREMODE_SHARED, base, BUFFER_HNS, 0, mix, None);
                    CoTaskMemFree(Some(mix as *const _));
                    r.map_err(|e| win_err("IAudioClient::Initialize", e))?;
                    let capture: IAudioCaptureClient =
                        client2.GetService().map_err(|e| win_err("IAudioCaptureClient", e))?;
                    client2.Start().map_err(|e| win_err("audio Start", e))?;
                    return Ok(Capture { client: client2, capture, conversion: Some(conversion), device_id });
                }
            };
            let capture: IAudioCaptureClient =
                client.GetService().map_err(|e| win_err("IAudioCaptureClient", e))?;
            client.Start().map_err(|e| win_err("audio Start", e))?;
            Ok(Capture { client, capture, conversion, device_id })
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

/// An active output device (speakers, headphones, a headset's stereo or hands-free side…).
pub struct Output {
    pub id: String,
    pub name: String,
}

/// Every active output. System audio is captured from all of them: a headset can move
/// playback between its "stereo" and "hands-free" outputs when its microphone opens, and
/// apps can play to a non-default device.
pub fn active_outputs() -> Vec<Output> {
    // SAFETY: COM enumeration with valid arguments; strings and property values are owned here.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let Ok(enumerator) =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return Vec::new();
        };
        let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) else {
            return Vec::new();
        };
        let count = collection.GetCount().unwrap_or(0);
        (0..count)
            .filter_map(|i| collection.Item(i).ok())
            .map(|d| {
                let name = d
                    .OpenPropertyStore(STGM_READ)
                    .ok()
                    .and_then(|store| store.GetValue(&PKEY_Device_FriendlyName).ok())
                    .map(|v| v.to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "output".into());
                Output { id: endpoint_id(&d), name }
            })
            .collect()
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

/// Device format → interleaved stereo f32 at 48 kHz.
unsafe fn convert(data: *const u8, frames: usize, c: Conversion, silent: bool) -> Vec<f32> {
    let mut stereo = vec![0.0f32; frames * 2];
    if !silent && !data.is_null() {
        for f in 0..frames {
            for ch in 0..2 {
                let src = f * c.channels + ch.min(c.channels - 1);
                // SAFETY: `src < frames * channels`, inside the packet returned by GetBuffer.
                stereo[f * 2 + ch] = unsafe {
                    match c.sample {
                        Sample::F32 => *(data as *const f32).add(src),
                        Sample::I16 => *(data as *const i16).add(src) as f32 / 32_768.0,
                        Sample::I32 => *(data as *const i32).add(src) as f32 / 2_147_483_648.0,
                    }
                };
            }
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
