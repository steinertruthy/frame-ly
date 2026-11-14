//! Screen recording on Windows, the counterpart of the macOS `Recorder.swift`:
//!
//! - Video: Windows.Graphics.Capture frames (monitor or window) → D3D11 video processor on the
//!   GPU (crop for regions, scale to the output size keeping aspect, BGRA → NV12, BT.709) →
//!   Media Foundation sink writer (hardware H.264/HEVC) into a fragmented MP4, so a crash
//!   leaves a playable file that the app recovers on the next launch.
//! - Frames arrive only when the screen changes (variable frame rate) and are capped at `fps`.
//! - Audio: WASAPI loopback (system audio) and microphone, mixed on the video clock (QPC) by
//!   `AudioMixer`, encoded to AAC in the same file.
//! - Pause keeps capture running and drops/retimes samples (`PauseTimeline`), like macOS.
//! - The OS ending the capture (window closed, display gone) finalizes the file and sends
//!   `NativeEvent::Stopped`.

use super::{capture, displays, media, wasapi::Capture, win_err};
use crate::{
    core::{
        errors::{AppError, AppResult},
        image::PixelRect,
        media_timeline::{AudioMixer, PauseTimeline},
        yuv,
    },
    platform::{AudioLevels, NativeEvent, NativeRecording, RecordingEvents, RecordingStats},
};
use std::{
    mem::ManuallyDrop,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread::JoinHandle,
    time::Duration,
};
use windows::{
    core::{IInspectable, Interface},
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{
            Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
        },
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
        SizeInt32,
    },
    Win32::{
        Foundation::{HMODULE, HWND, RECT},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::*,
            Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_RATIONAL, DXGI_SAMPLE_DESC},
        },
        Media::MediaFoundation::{
            IMF2DBuffer, IMFDXGIBuffer, IMFDXGIDeviceManager, IMFVideoSampleAllocatorEx,
            MFCreateDXGIDeviceManager, MFCreateVideoSampleAllocatorEx, MFVideoFormat_NV12,
            MF_E_SAMPLEALLOCATOR_EMPTY, MF_SA_D3D11_BINDFLAGS,
        },
        System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
    },
};

/// Clock shared by capture frames and WASAPI packets: QPC in 100 ns units.
const TICKS: i64 = 10_000_000;

fn now() -> i64 {
    let (mut c, mut f) = (0i64, 0i64);
    // SAFETY: plain out-parameter reads; both calls cannot fail on supported Windows.
    unsafe {
        let _ = QueryPerformanceCounter(&mut c);
        let _ = QueryPerformanceFrequency(&mut f);
    }
    (c as i128 * TICKS as i128 / f.max(1) as i128) as i64
}

/// GPU objects and the capture pool. Only touched under `Engine::video`.
struct Video {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    video_device: ID3D11VideoDevice,
    video_context: ID3D11VideoContext,
    winrt: IDirect3DDevice,
    allocator: IMFVideoSampleAllocatorEx,
    pool: Direct3D11CaptureFramePool,
    session: GraphicsCaptureSession,
    pool_size: SizeInt32,
    /// Latest frame, copied out of the capture pool (input of the video processor).
    source: Option<Source>,
    processor: Option<Processor>,
    /// Size of the valid content inside `source` (windows can be smaller than the pool).
    content: (u32, u32),
    /// Region recordings: the part of the monitor to keep.
    crop: Option<RECT>,
    out: (u32, u32),
    /// Output time (paused spans removed) of the last written frame, and whether `source`
    /// holds newer content than that (skipped by the frame-rate cap).
    last_written: i64,
    newer: bool,
    frames: u64,
    finished: bool,
    /// The GPU video processor failed on this machine: convert on the CPU instead.
    cpu: bool,
    staging: Option<Source>,
    stats: Diagnostics,
}

/// What happened to the frames, logged when the recording ends (and the first error of each
/// path as soon as it happens), so a failure on a user's machine is visible in the log.
#[derive(Default, Debug)]
struct Diagnostics {
    arrived: u64,
    paused: u64,
    capped: u64,
    busy: u64,
    errors: u64,
    last_error: Option<String>,
}

struct Source {
    texture: ID3D11Texture2D,
    size: (u32, u32),
}

struct Processor {
    enumerator: ID3D11VideoProcessorEnumerator,
    processor: ID3D11VideoProcessor,
    input: ID3D11VideoProcessorInputView,
    input_size: (u32, u32),
}

struct AudioState {
    mixer: Option<AudioMixer>,
    gains: Vec<f32>,
    /// (system, microphone) silenced by the user during the recording.
    muted: (bool, bool),
    system: Option<usize>,
    mic: Option<usize>,
}

struct Engine {
    video: Mutex<Video>,
    writer: Mutex<Option<media::Writer>>,
    timeline: Mutex<PauseTimeline>,
    audio: Mutex<AudioState>,
    /// Clock time of the first written frame = time zero of the file.
    origin: OnceLock<i64>,
    interval: i64,
    output: PathBuf,
    thumbnail: PathBuf,
    events: RecordingEvents,
    stop_audio: AtomicBool,
    /// Frames the capture handler could not even hand to `on_frame` processing.
    frame_errors: std::sync::atomic::AtomicU64,
}

// SAFETY: every COM object is created in the MTA, the D3D device is multithread-protected, and
// all mutable access goes through the mutexes above.
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

pub struct Recorder {
    engine: Arc<Engine>,
    item: GraphicsCaptureItem,
    tokens: (i64, i64),
    audio_thread: Option<JoinHandle<()>>,
}

// SAFETY: see `Engine`; the capture item is an agile WinRT object.
unsafe impl Send for Recorder {}

static CURRENT: Mutex<Option<Recorder>> = Mutex::new(None);

pub fn start(config: &NativeRecording, events: RecordingEvents) -> AppResult<()> {
    let mut current = CURRENT.lock().unwrap();
    if current.is_some() {
        return Err(AppError::Runtime("a recording is already in progress".into()));
    }
    *current = Some(Recorder::start(config, events)?);
    Ok(())
}

pub fn pause() {
    if let Some(r) = CURRENT.lock().unwrap().as_ref() {
        r.engine.timeline.lock().unwrap().pause(now());
    }
}

pub fn resume() {
    if let Some(r) = CURRENT.lock().unwrap().as_ref() {
        r.engine.timeline.lock().unwrap().resume(now());
    }
}

pub fn stop() -> AppResult<RecordingStats> {
    let recorder = CURRENT
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AppError::Platform("no recording in progress".into()))?;
    recorder.finish()
}

/// Releases a recorder the system already stopped (after a `Stopped` event): nothing left.
pub fn discard() {
    if let Some(r) = CURRENT.lock().unwrap().take() {
        let _ = r.finish();
    }
}

/// Silences (or restores) system audio / microphone in the mix; recording continues.
pub fn set_muted(system: bool, microphone: bool) {
    if let Some(r) = CURRENT.lock().unwrap().as_ref() {
        let mut a = r.engine.audio.lock().unwrap();
        a.muted = (system, microphone);
        a.apply_gains();
    }
}

pub fn levels() -> AudioLevels {
    let current = CURRENT.lock().unwrap();
    let Some(r) = current.as_ref() else { return AudioLevels::default() };
    let audio = r.engine.audio.lock().unwrap();
    let peak =
        |i: Option<usize>| i.and_then(|i| audio.mixer.as_ref().map(|m| m.sources[i].peak)).unwrap_or(0.0);
    AudioLevels { system: peak(audio.system), microphone: peak(audio.mic) }
}

fn create_device() -> AppResult<ID3D11Device> {
    let mut device = None;
    // SAFETY: standard device creation with out-pointer.
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
        .map_err(|e| win_err("D3D11CreateDevice (video)", e))?;
    }
    let device = device.ok_or_else(|| AppError::Platform("no Direct3D device".into()))?;
    // Capture callbacks, the encoder and the stop path all use the device.
    if let Ok(mt) = device.cast::<ID3D11Multithread>() {
        // SAFETY: valid interface.
        unsafe {
            let _ = mt.SetMultithreadProtected(true);
        }
    }
    Ok(device)
}

fn parse_window(id: &str) -> AppResult<HWND> {
    let raw = usize::from_str_radix(id.trim_start_matches("0x"), 16)
        .map_err(|_| AppError::Platform(format!("invalid window id {id:?}")))?;
    Ok(HWND(raw as *mut _))
}

impl Recorder {
    fn start(config: &NativeRecording, events: RecordingEvents) -> AppResult<Recorder> {
        media::init()?;
        let t = &config.target;
        let (item, crop) = match t.kind {
            "window" => {
                let id =
                    t.window_id.as_deref().ok_or_else(|| AppError::Platform("window id missing".into()))?;
                (capture::item_for_window(parse_window(id)?)?, None)
            }
            "display" | "region" => {
                let id =
                    t.display_id.as_deref().ok_or_else(|| AppError::Platform("display id missing".into()))?;
                let monitor = displays::monitor_by_id(id)
                    .ok_or_else(|| AppError::Platform(format!("display {id} not found")))?;
                let crop = match (t.kind, t.x, t.y, t.width, t.height) {
                    ("region", Some(x), Some(y), Some(w), Some(h)) => Some(RECT {
                        left: x.round() as i32,
                        top: y.round() as i32,
                        right: (x + w).round() as i32,
                        bottom: (y + h).round() as i32,
                    }),
                    _ => None,
                };
                (capture::item_for_monitor(monitor)?, crop)
            }
            other => return Err(AppError::Platform(format!("unknown target {other}"))),
        };

        let device = create_device()?;
        let winrt = capture::winrt_device(&device)?;
        let out = (config.width & !1, config.height & !1);

        // SAFETY: COM/D3D/MF setup with valid arguments; out-pointers are owned locals.
        let (context, video_device, video_context, manager, allocator) = unsafe {
            let context = device.GetImmediateContext().map_err(|e| win_err("device context", e))?;
            let video_device: ID3D11VideoDevice =
                device.cast().map_err(|e| win_err("ID3D11VideoDevice", e))?;
            let video_context: ID3D11VideoContext =
                context.cast().map_err(|e| win_err("ID3D11VideoContext", e))?;
            let (mut token, mut manager) = (0u32, None::<IMFDXGIDeviceManager>);
            MFCreateDXGIDeviceManager(&mut token, &mut manager)
                .map_err(|e| win_err("DXGI device manager", e))?;
            let manager = manager.ok_or_else(|| AppError::Platform("no DXGI device manager".into()))?;
            manager.ResetDevice(&device, token).map_err(|e| win_err("ResetDevice", e))?;

            let mut raw = std::ptr::null_mut();
            MFCreateVideoSampleAllocatorEx(&IMFVideoSampleAllocatorEx::IID, &mut raw)
                .map_err(|e| win_err("sample allocator", e))?;
            let allocator = IMFVideoSampleAllocatorEx::from_raw(raw);
            allocator.SetDirectXManager(&manager).map_err(|e| win_err("allocator device", e))?;
            let mut attrs = None;
            windows::Win32::Media::MediaFoundation::MFCreateAttributes(&mut attrs, 1)
                .map_err(|e| win_err("allocator attributes", e))?;
            let attrs = attrs.ok_or_else(|| AppError::Platform("no attributes".into()))?;
            attrs
                .SetUINT32(&MF_SA_D3D11_BINDFLAGS, D3D11_BIND_RENDER_TARGET.0 as u32)
                .map_err(|e| win_err("allocator bind flags", e))?;
            let nv12 = media::video_type(&MFVideoFormat_NV12, out.0, out.1, config.fps)?;
            // A few frames in flight; when the encoder falls behind, allocation fails and the
            // frame is dropped instead of stalling capture.
            allocator
                .InitializeSampleAllocatorEx(4, 12, &attrs, &nv12)
                .map_err(|e| win_err("allocator init", e))?;
            (context, video_device, video_context, manager, allocator)
        };

        let wants_system = config.audio.as_ref().is_some_and(|a| a.system);
        let wants_mic = config.audio.as_ref().is_some_and(|a| a.microphone);
        let writer = media::Writer::create(&media::WriterConfig {
            path: std::path::Path::new(&config.output_path),
            width: out.0,
            height: out.1,
            fps: config.fps,
            bitrate: config.bitrate,
            quality: config.quality,
            codec: config.codec,
            audio: wants_system || wants_mic,
            manager: &manager,
        })?;

        // Audio devices open before capture starts, so their first packets are not late.
        let system = if wants_system {
            Capture::loopback().map_err(|e| tracing::warn!(error = %e, "system audio unavailable")).ok()
        } else {
            None
        };
        let mic_id = config.audio.as_ref().and_then(|a| a.microphone_id.clone());
        let mic = if wants_mic {
            Capture::microphone(mic_id.as_deref())
                .map_err(|e| tracing::warn!(error = %e, "microphone unavailable"))
                .ok()
        } else {
            None
        };
        let mut gains = Vec::new();
        let system_index = system.as_ref().map(|_| {
            gains.push(config.audio.as_ref().map_or(1.0, |a| a.system_gain));
            gains.len() - 1
        });
        let mic_index = mic.as_ref().map(|_| {
            gains.push(config.audio.as_ref().map_or(1.0, |a| a.microphone_gain));
            gains.len() - 1
        });

        let size = item.Size().map_err(|e| win_err("capture size", e))?;
        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )
        .map_err(|e| win_err("frame pool", e))?;
        let session = pool.CreateCaptureSession(&item).map_err(|e| win_err("capture session", e))?;
        // Optional properties: missing on older Windows 10 builds — ignore if unsupported.
        let _ = session.SetIsCursorCaptureEnabled(config.shows_cursor);
        let _ = session.SetIsBorderRequired(false);

        let engine = Arc::new(Engine {
            video: Mutex::new(Video {
                device,
                context,
                video_device,
                video_context,
                winrt,
                allocator,
                pool: pool.clone(),
                session: session.clone(),
                pool_size: size,
                source: None,
                processor: None,
                content: (size.Width.max(1) as u32, size.Height.max(1) as u32),
                crop,
                out,
                last_written: 0,
                newer: false,
                frames: 0,
                finished: false,
                cpu: false,
                staging: None,
                stats: Diagnostics::default(),
            }),
            writer: Mutex::new(Some(writer)),
            timeline: Mutex::new(PauseTimeline::default()),
            audio: Mutex::new(AudioState {
                mixer: None,
                gains,
                muted: (false, false),
                system: system_index,
                mic: mic_index,
            }),
            origin: OnceLock::new(),
            interval: TICKS / config.fps.max(1) as i64,
            output: PathBuf::from(&config.output_path),
            thumbnail: PathBuf::from(&config.thumbnail_path),
            events,
            stop_audio: AtomicBool::new(false),
            frame_errors: std::sync::atomic::AtomicU64::new(0),
        });

        let e = Arc::clone(&engine);
        let on_frame = TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(move |pool, _| {
            if let Some(pool) = pool.as_ref() {
                while let Ok(frame) = pool.TryGetNextFrame() {
                    if let Err(err) = e.on_frame(&frame) {
                        if e.frame_errors.fetch_add(1, Ordering::Relaxed) == 0 {
                            tracing::warn!(error = %err, "recording: captured frame could not be processed");
                        }
                    }
                    let _ = frame.Close();
                }
            }
            Ok(())
        });
        let frame_token = pool.FrameArrived(&on_frame).map_err(|e| win_err("FrameArrived", e))?;

        let weak = Arc::downgrade(&engine);
        let on_closed = TypedEventHandler::<GraphicsCaptureItem, IInspectable>::new(move |_, _| {
            // The window was closed or the display went away: finish on another thread (this
            // one belongs to the capture system).
            if let Some(engine) = weak.upgrade() {
                std::thread::spawn(move || closed_by_system(engine));
            }
            Ok(())
        });
        let closed_token = item.Closed(&on_closed).map_err(|e| win_err("Closed", e))?;

        session.StartCapture().map_err(|e| win_err("StartCapture", e))?;

        let audio_engine = Arc::clone(&engine);
        let audio_thread = (system.is_some() || mic.is_some())
            .then(|| {
                std::thread::Builder::new()
                    .name("framely-audio".into())
                    .spawn(move || audio_engine.run_audio(system, mic))
                    .ok()
            })
            .flatten();

        Ok(Recorder { engine, item, tokens: (frame_token, closed_token), audio_thread })
    }

    /// Stops capture, writes the tail of the recording and closes the file.
    fn finish(mut self) -> AppResult<RecordingStats> {
        let end_clock = now();
        let _ = self.item.RemoveClosed(self.tokens.1);
        let engine = &self.engine;

        let (frames, last_written) = {
            let mut v = engine.video.lock().unwrap();
            let _ = v.pool.RemoveFrameArrived(self.tokens.0);
            let _ = v.session.Close();
            let _ = v.pool.Close();
            if v.finished {
                return Err(AppError::Platform("recording already finished".into()));
            }
            // Keep the last image on screen until the moment stop was pressed (static screens
            // deliver no new frames), minus time spent paused.
            let end = {
                let mut tl = engine.timeline.lock().unwrap();
                if tl.is_paused() {
                    tl.resume(end_clock);
                }
                end_clock - tl.paused_duration(end_clock)
            };
            if let Some(&origin) = engine.origin.get() {
                if v.frames > 0 && (v.newer || end - v.last_written > engine.interval) {
                    let at = (end - engine.interval).max(v.last_written + 1);
                    if let Err(e) = engine.render(&mut v, at - origin) {
                        tracing::debug!(error = %e, "final frame not written");
                    }
                    v.last_written = at;
                }
            }
            v.finished = true;
            tracing::info!(
                written = v.frames,
                cpu = v.cpu,
                stats = ?v.stats,
                handler_errors = engine.frame_errors.load(Ordering::Relaxed),
                "recording: frames"
            );
            (v.frames, v.last_written.max(end))
        };

        engine.stop_audio.store(true, Ordering::SeqCst);
        if let Some(t) = self.audio_thread.take() {
            let _ = t.join();
        }

        let writer = engine.writer.lock().unwrap().take();
        let Some(origin) = engine.origin.get().copied().filter(|_| frames > 0) else {
            drop(writer); // releases the file
            let _ = std::fs::remove_file(&engine.output);
            return Err(AppError::EmptyRecording);
        };
        let audio = writer.as_ref().is_some_and(|w| w.has_audio());
        if let Some(w) = writer {
            w.finalize()?;
        }
        let video = engine.video.lock().unwrap();
        Ok(RecordingStats {
            duration_ms: ((last_written - origin + engine.interval).max(0) / 10_000) as u64,
            frames,
            width: video.out.0,
            height: video.out.1,
            audio,
        })
    }
}

/// The capture item went away (window closed, display unplugged): finalize and tell the app.
fn closed_by_system(engine: Arc<Engine>) {
    let recorder = {
        let mut current = CURRENT.lock().unwrap();
        match current.as_ref() {
            Some(r) if Arc::ptr_eq(&r.engine, &engine) => current.take(),
            _ => None,
        }
    };
    let Some(recorder) = recorder else { return };
    let result = recorder.finish();
    let (duration_ms, frames) = result.as_ref().map(|s| (s.duration_ms, s.frames)).unwrap_or((0, 0));
    (engine.events)(NativeEvent::Stopped {
        reason: "the captured window or display is no longer available".into(),
        duration_ms,
        frames,
    });
}

impl Engine {
    fn on_frame(&self, frame: &Direct3D11CaptureFrame) -> AppResult<()> {
        let clock = frame.SystemRelativeTime().map_err(|e| win_err("frame time", e))?.Duration;
        let content = frame.ContentSize().map_err(|e| win_err("content size", e))?;
        let texture = capture::frame_texture(frame)?;
        let mut v = self.video.lock().unwrap();
        if v.finished {
            return Ok(());
        }
        v.stats.arrived += 1;
        // Windows resized: grow/shrink the pool for the next frames.
        if content.Width > 0
            && content.Height > 0
            && (content.Width != v.pool_size.Width || content.Height != v.pool_size.Height)
        {
            let (pool, winrt) = (v.pool.clone(), v.winrt.clone());
            if pool.Recreate(&winrt, DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, content).is_ok() {
                v.pool_size = content;
            }
        }
        v.copy_source(&texture)?;
        v.content = (content.Width.max(1) as u32, content.Height.max(1) as u32);

        let Some(at) = self.timeline.lock().unwrap().adjusted(clock) else {
            v.stats.paused += 1;
            return Ok(());
        };
        let mut first = false;
        let origin = *self.origin.get_or_init(|| {
            first = true;
            at
        });
        v.newer = true;
        if v.frames > 0 && at - v.last_written < self.interval * 9 / 10 {
            v.stats.capped += 1;
            return Ok(()); // frame-rate cap
        }
        if let Err(e) = self.render(&mut v, at - origin) {
            v.stats.errors += 1;
            if v.stats.errors == 1 {
                tracing::warn!(error = %e, cpu = v.cpu, "recording: frame not written");
            }
            v.stats.last_error = Some(e.to_string());
            return Ok(());
        }
        v.last_written = at;
        if first {
            self.audio.lock().unwrap().start(clock);
            (self.events)(NativeEvent::Started);
            self.write_thumbnail(&v);
        }
        Ok(())
    }

    /// Converts the current source into NV12 and hands it to the encoder: on the GPU, or on
    /// the CPU once the GPU path has failed on this machine.
    fn render(&self, v: &mut Video, time: i64) -> AppResult<()> {
        if !v.cpu {
            match self.render_gpu(v, time) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::warn!(error = %e, "recording: GPU conversion unavailable; converting frames on the CPU");
                    v.cpu = true;
                }
            }
        }
        self.render_cpu(v, time)
    }

    fn render_cpu(&self, v: &mut Video, time: i64) -> AppResult<()> {
        let src = v.source.as_ref().ok_or_else(|| AppError::Platform("no frame yet".into()))?;
        let (texture, size) = (src.texture.clone(), src.size);
        // SAFETY: staging copy/map on the protected device; the mapped memory is read only
        // between Map and Unmap, within RowPitch × height.
        let nv12 = unsafe {
            if !v.staging.as_ref().is_some_and(|s| s.size == size) {
                let mut desc = D3D11_TEXTURE2D_DESC::default();
                texture.GetDesc(&mut desc);
                desc.Usage = D3D11_USAGE_STAGING;
                desc.BindFlags = 0;
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
                desc.MiscFlags = 0;
                let mut staging = None;
                v.device
                    .CreateTexture2D(&desc, None, Some(&mut staging))
                    .map_err(|e| win_err("staging texture", e))?;
                let texture = staging.ok_or_else(|| AppError::Platform("no staging texture".into()))?;
                v.staging = Some(Source { texture, size });
            }
            let staging = &v.staging.as_ref().unwrap().texture;
            v.context.CopyResource(staging, &texture);
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            v.context.Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).map_err(|e| win_err("Map", e))?;
            let stride = mapped.RowPitch as usize;
            let data = std::slice::from_raw_parts(mapped.pData as *const u8, stride * size.1 as usize);
            let (cw, ch) = (v.content.0.min(size.0), v.content.1.min(size.1));
            let crop = match v.crop {
                Some(c) => {
                    let x = c.left.clamp(0, cw as i32) as u32;
                    let y = c.top.clamp(0, ch as i32) as u32;
                    (
                        x,
                        y,
                        (c.right.clamp(0, cw as i32) as u32).saturating_sub(x),
                        (c.bottom.clamp(0, ch as i32) as u32).saturating_sub(y),
                    )
                }
                None => (0, 0, cw, ch),
            };
            let nv12 =
                yuv::bgra_to_nv12(&yuv::Bgra { data, stride, width: size.0, height: size.1 }, crop, v.out);
            v.context.Unmap(staging, 0);
            nv12
        };
        let writer = self.writer.lock().unwrap();
        if let Some(w) = writer.as_ref() {
            w.write_video_nv12(&nv12, time.max(0), self.interval)?;
            v.frames += 1;
            v.newer = false;
        }
        Ok(())
    }

    fn render_gpu(&self, v: &mut Video, time: i64) -> AppResult<()> {
        v.ensure_processor()?;
        // SAFETY: D3D11/MF calls on objects owned by `v`, serialized by the video mutex; the
        // stream descriptor's input view is dropped right after the blit.
        unsafe {
            let sample = match v.allocator.AllocateSample() {
                Ok(s) => s,
                Err(e) if e.code() == MF_E_SAMPLEALLOCATOR_EMPTY => {
                    v.stats.busy += 1; // encoder busy: drop
                    return Ok(());
                }
                Err(e) => return Err(win_err("AllocateSample", e)),
            };
            let buffer = sample.GetBufferByIndex(0).map_err(|e| win_err("sample buffer", e))?;
            let dxgi: IMFDXGIBuffer = buffer.cast().map_err(|e| win_err("IMFDXGIBuffer", e))?;
            let mut raw = std::ptr::null_mut();
            dxgi.GetResource(&ID3D11Texture2D::IID, &mut raw).map_err(|e| win_err("sample texture", e))?;
            let target = ID3D11Texture2D::from_raw(raw);
            let slice = dxgi.GetSubresourceIndex().map_err(|e| win_err("subresource", e))?;
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            target.GetDesc(&mut desc);

            let p = v.processor.as_ref().ok_or_else(|| AppError::Platform("no video processor".into()))?;
            let mut view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC::default();
            if desc.ArraySize > 1 {
                view_desc.ViewDimension = D3D11_VPOV_DIMENSION_TEXTURE2DARRAY;
                view_desc.Anonymous.Texture2DArray =
                    D3D11_TEX2D_ARRAY_VPOV { MipSlice: 0, FirstArraySlice: slice, ArraySize: 1 };
            } else {
                view_desc.ViewDimension = D3D11_VPOV_DIMENSION_TEXTURE2D;
            }
            let mut out_view = None;
            v.video_device
                .CreateVideoProcessorOutputView(&target, &p.enumerator, &view_desc, Some(&mut out_view))
                .map_err(|e| win_err("output view", e))?;
            let out_view = out_view.ok_or_else(|| AppError::Platform("no output view".into()))?;

            let src = match v.crop {
                Some(c) => RECT {
                    left: c.left.clamp(0, v.content.0 as i32),
                    top: c.top.clamp(0, v.content.1 as i32),
                    right: c.right.clamp(0, v.content.0 as i32),
                    bottom: c.bottom.clamp(0, v.content.1 as i32),
                },
                None => RECT { left: 0, top: 0, right: v.content.0 as i32, bottom: v.content.1 as i32 },
            };
            let src_size = ((src.right - src.left).max(1) as u32, (src.bottom - src.top).max(1) as u32);
            let (dx, dy, dw, dh) = yuv::fit(src_size, v.out);
            let dest =
                RECT { left: dx as i32, top: dy as i32, right: (dx + dw) as i32, bottom: (dy + dh) as i32 };
            let vc = &v.video_context;
            vc.VideoProcessorSetStreamSourceRect(&p.processor, 0, true, Some(&src));
            vc.VideoProcessorSetStreamDestRect(&p.processor, 0, true, Some(&dest));

            let stream = D3D11_VIDEO_PROCESSOR_STREAM {
                Enable: true.into(),
                pInputSurface: ManuallyDrop::new(Some(p.input.clone())),
                ..Default::default()
            };
            let streams = [stream];
            let blit = vc.VideoProcessorBlt(&p.processor, &out_view, 0, &streams);
            let [mut stream] = streams;
            ManuallyDrop::drop(&mut stream.pInputSurface);
            blit.map_err(|e| win_err("VideoProcessorBlt", e))?;
            // Pooled DXGI samples come with a current length of 0, which the encoder rejects
            // as an empty frame (E_INVALIDARG): declare the whole NV12 surface as filled.
            let length = match buffer.cast::<IMF2DBuffer>() {
                Ok(b2) => b2.GetContiguousLength().map_err(|e| win_err("contiguous length", e))?,
                Err(_) => buffer.GetMaxLength().map_err(|e| win_err("max length", e))?,
            };
            buffer.SetCurrentLength(length).map_err(|e| win_err("sample length", e))?;

            let writer = self.writer.lock().unwrap();
            if let Some(w) = writer.as_ref() {
                w.write_video(&sample, time.max(0), self.interval)?;
                v.frames += 1;
                v.newer = false;
            }
        }
        Ok(())
    }

    /// First frame → JPEG preview for the library (off the capture thread).
    fn write_thumbnail(&self, v: &Video) {
        let Some(src) = v.source.as_ref() else { return };
        let frame = match capture::texture_to_frame(&v.device, &src.texture) {
            Ok(f) => f,
            Err(e) => return tracing::debug!(error = %e, "thumbnail skipped"),
        };
        let (cw, ch) = v.content;
        let rect = match v.crop {
            Some(c) => PixelRect {
                x: c.left.max(0) as u32,
                y: c.top.max(0) as u32,
                width: (c.right - c.left).max(1) as u32,
                height: (c.bottom - c.top).max(1) as u32,
            },
            None => PixelRect { x: 0, y: 0, width: cw.min(frame.width), height: ch.min(frame.height) },
        };
        let path = self.thumbnail.clone();
        std::thread::spawn(move || {
            if let Err(e) = frame.crop(rect).and_then(|f| media::save_jpeg_thumbnail(&f, &path, 640)) {
                tracing::debug!(error = %e, "thumbnail not written");
            }
        });
    }

    /// Audio thread: polls both devices every ~10 ms, mixes on the video clock and writes AAC.
    fn run_audio(self: Arc<Self>, mut system: Option<Capture>, mut mic: Option<Capture>) {
        let _ = media::init();
        loop {
            let stopping = self.stop_audio.load(Ordering::SeqCst);
            let (sys_index, mic_index) = {
                let a = self.audio.lock().unwrap();
                (a.system, a.mic)
            };
            for (capture, index) in [(&mut system, sys_index), (&mut mic, mic_index)] {
                let (Some(c), Some(i)) = (capture.as_mut(), index) else { continue };
                let result = c.read(|t, samples| {
                    if let Some(m) = self.audio.lock().unwrap().mixer.as_mut() {
                        m.push(i, t, samples);
                    }
                });
                if let Err(e) = result {
                    tracing::warn!(error = %e, "audio device stopped; continuing without it");
                    if let Some(m) = self.audio.lock().unwrap().mixer.as_mut() {
                        m.deactivate(i);
                    }
                    *capture = None;
                }
            }
            self.drain(stopping);
            if stopping {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn drain(&self, flush: bool) {
        let Some(&origin) = self.origin.get() else { return };
        loop {
            let chunk = {
                let mut a = self.audio.lock().unwrap();
                let Some(m) = a.mixer.as_mut() else { return };
                m.pull(flush).map(|(start, samples)| (m.time_of_frame(start), samples))
            };
            let Some((clock, samples)) = chunk else { return };
            // Paused spans are dropped and later audio shifts back, exactly like video.
            let Some(at) = self.timeline.lock().unwrap().adjusted(clock) else { continue };
            if let Some(w) = self.writer.lock().unwrap().as_ref() {
                if let Err(e) = w.write_audio(&samples, at - origin) {
                    tracing::debug!(error = %e, "audio chunk dropped");
                }
            }
        }
    }
}

impl AudioState {
    /// Audio shares the video timeline: frame 0 of the mix = first video frame.
    fn start(&mut self, origin_clock: i64) {
        if self.mixer.is_none() && !self.gains.is_empty() {
            self.mixer = Some(AudioMixer::new(origin_clock, TICKS, &self.gains));
            self.apply_gains();
        }
    }

    /// Mixer gains = configured gains, zeroed for muted sources.
    fn apply_gains(&mut self) {
        let Some(m) = self.mixer.as_mut() else { return };
        for (index, muted) in [(self.system, self.muted.0), (self.mic, self.muted.1)] {
            if let Some(i) = index {
                m.sources[i].gain = if muted { 0.0 } else { self.gains[i] };
            }
        }
    }
}

impl Video {
    /// Copies the captured frame into our own texture (the pool's surfaces are recycled and
    /// cannot be bound to the video processor).
    fn copy_source(&mut self, frame: &ID3D11Texture2D) -> AppResult<()> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: descriptor read and texture creation/copy on the protected device.
        unsafe {
            frame.GetDesc(&mut desc);
            let size = (desc.Width, desc.Height);
            if !self.source.as_ref().is_some_and(|s| s.size == size) {
                let mut own = D3D11_TEXTURE2D_DESC {
                    Width: desc.Width,
                    Height: desc.Height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                    CPUAccessFlags: 0,
                    MiscFlags: 0,
                };
                own.Format = desc.Format;
                let mut texture = None;
                self.device
                    .CreateTexture2D(&own, None, Some(&mut texture))
                    .map_err(|e| win_err("source texture", e))?;
                let texture = texture.ok_or_else(|| AppError::Platform("no source texture".into()))?;
                self.source = Some(Source { texture, size });
                self.processor = None; // input view belongs to the old texture
            }
            let src = self.source.as_ref().unwrap();
            self.context.CopyResource(&src.texture, frame);
        }
        Ok(())
    }

    /// (Re)creates the video processor for the current source size.
    fn ensure_processor(&mut self) -> AppResult<()> {
        let src = self.source.as_ref().ok_or_else(|| AppError::Platform("no frame yet".into()))?;
        if self.processor.as_ref().is_some_and(|p| p.input_size == src.size) {
            return Ok(());
        }
        let rate = DXGI_RATIONAL { Numerator: 60, Denominator: 1 };
        let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
            InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            InputFrameRate: rate,
            InputWidth: src.size.0,
            InputHeight: src.size.1,
            OutputFrameRate: rate,
            OutputWidth: self.out.0,
            OutputHeight: self.out.1,
            Usage: D3D11_VIDEO_USAGE_OPTIMAL_SPEED,
        };
        // SAFETY: video device calls with valid descriptors; out-pointers are owned locals.
        unsafe {
            let enumerator = self
                .video_device
                .CreateVideoProcessorEnumerator(&content)
                .map_err(|e| win_err("processor enumerator", e))?;
            let processor = self
                .video_device
                .CreateVideoProcessor(&enumerator, 0)
                .map_err(|e| win_err("video processor", e))?;
            let input_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
                FourCC: 0,
                ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
                Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                    Texture2D: D3D11_TEX2D_VPIV { MipSlice: 0, ArraySlice: 0 },
                },
            };
            let mut input = None;
            self.video_device
                .CreateVideoProcessorInputView(&src.texture, &enumerator, &input_desc, Some(&mut input))
                .map_err(|e| win_err("input view", e))?;
            let input = input.ok_or_else(|| AppError::Platform("no input view".into()))?;

            let vc = &self.video_context;
            vc.VideoProcessorSetStreamFrameFormat(&processor, 0, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE);
            vc.VideoProcessorSetStreamAutoProcessingMode(&processor, 0, false);
            let full = RECT { left: 0, top: 0, right: self.out.0 as i32, bottom: self.out.1 as i32 };
            vc.VideoProcessorSetOutputTargetRect(&processor, true, Some(&full));
            let black = D3D11_VIDEO_COLOR {
                Anonymous: D3D11_VIDEO_COLOR_0 {
                    RGBA: D3D11_VIDEO_COLOR_RGBA { R: 0.0, G: 0.0, B: 0.0, A: 1.0 },
                },
            };
            vc.VideoProcessorSetOutputBackgroundColor(&processor, false, &black);
            // Input: full-range RGB. Output: BT.709 YCbCr, limited (16–235) range.
            // Bitfield: Usage:1 RGB_Range:1 YCbCr_Matrix:1 YCbCr_xvYCC:1 Nominal_Range:2.
            vc.VideoProcessorSetStreamColorSpace(
                &processor,
                0,
                &D3D11_VIDEO_PROCESSOR_COLOR_SPACE { _bitfield: 0 },
            );
            vc.VideoProcessorSetOutputColorSpace(
                &processor,
                &D3D11_VIDEO_PROCESSOR_COLOR_SPACE { _bitfield: (1 << 2) | (1 << 4) },
            );
            self.processor = Some(Processor { enumerator, processor, input, input_size: src.size });
        }
        Ok(())
    }
}
