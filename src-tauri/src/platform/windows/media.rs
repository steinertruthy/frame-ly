//! Media Foundation: the MP4 writer used by the recorder (hardware H.264/HEVC + AAC), and
//! duration/size/thumbnail of existing videos for the library.

use super::win_err;
use crate::core::{
    errors::{AppError, AppResult},
    library::Meta,
    media_timeline::{CHANNELS, SAMPLE_RATE},
    video::Codec,
};
use std::{path::Path, sync::OnceLock};
use windows::{
    core::{Interface, HSTRING},
    Win32::{
        Media::MediaFoundation::*,
        System::{
            Com::{CoInitializeEx, StructuredStorage::PROPVARIANT, COINIT_MULTITHREADED},
            Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_UI4},
        },
    },
};

/// COM (multithreaded) on the calling thread, and Media Foundation once per process.
pub fn init() -> AppResult<()> {
    // SAFETY: COM init on this thread; "already initialized" results are fine.
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    static STARTED: OnceLock<Result<(), String>> = OnceLock::new();
    STARTED
        // SAFETY: plain library initialization; never shut down (process lifetime).
        .get_or_init(|| unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }.map_err(|e| e.to_string()))
        .clone()
        .map_err(|e| AppError::Platform(format!("Media Foundation unavailable: {e}")))
}

fn attributes(count: u32) -> AppResult<IMFAttributes> {
    let mut attrs = None;
    // SAFETY: out-pointer filled by MF.
    unsafe { MFCreateAttributes(&mut attrs, count) }.map_err(|e| win_err("MFCreateAttributes", e))?;
    attrs.ok_or_else(|| AppError::Platform("MFCreateAttributes returned nothing".into()))
}

/// Packs two u32 into the UINT64 attribute layout MF uses for sizes and ratios.
fn pair(hi: u32, lo: u32) -> u64 {
    ((hi as u64) << 32) | lo as u64
}

pub fn video_type(
    subtype: &windows::core::GUID,
    width: u32,
    height: u32,
    fps: u32,
) -> AppResult<IMFMediaType> {
    // SAFETY: attribute setters on a fresh media type.
    unsafe {
        let t = MFCreateMediaType().map_err(|e| win_err("MFCreateMediaType", e))?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video).map_err(|e| win_err("video major type", e))?;
        t.SetGUID(&MF_MT_SUBTYPE, subtype).map_err(|e| win_err("video subtype", e))?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pair(width, height)).map_err(|e| win_err("frame size", e))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pair(fps, 1)).map_err(|e| win_err("frame rate", e))?;
        t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pair(1, 1)).map_err(|e| win_err("aspect ratio", e))?;
        t.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
            .map_err(|e| win_err("interlace mode", e))?;
        // BT.709, limited range — what the GPU converter produces and players assume for HD.
        let _ = t.SetUINT32(&MF_MT_YUV_MATRIX, MFVideoTransferMatrix_BT709.0 as u32);
        let _ = t.SetUINT32(&MF_MT_VIDEO_PRIMARIES, MFVideoPrimaries_BT709.0 as u32);
        let _ = t.SetUINT32(&MF_MT_TRANSFER_FUNCTION, MFVideoTransFunc_709.0 as u32);
        let _ = t.SetUINT32(&MF_MT_VIDEO_NOMINAL_RANGE, MFNominalRange_16_235.0 as u32);
        Ok(t)
    }
}

fn audio_type(subtype: &windows::core::GUID, aac: bool) -> AppResult<IMFMediaType> {
    let channels = CHANNELS as u32;
    // SAFETY: attribute setters on a fresh media type.
    unsafe {
        let t = MFCreateMediaType().map_err(|e| win_err("MFCreateMediaType", e))?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio).map_err(|e| win_err("audio major type", e))?;
        t.SetGUID(&MF_MT_SUBTYPE, subtype).map_err(|e| win_err("audio subtype", e))?;
        t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, SAMPLE_RATE).map_err(|e| win_err("sample rate", e))?;
        t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, channels).map_err(|e| win_err("channels", e))?;
        t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16).map_err(|e| win_err("bits per sample", e))?;
        if aac {
            // 192 kbit/s, like the macOS recorder.
            t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, 24_000).map_err(|e| win_err("AAC bitrate", e))?;
        } else {
            t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, channels * 2).map_err(|e| win_err("block align", e))?;
            t.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, SAMPLE_RATE * channels * 2)
                .map_err(|e| win_err("PCM byte rate", e))?;
            t.SetUINT32(&MF_MT_ALL_SAMPLES_INDEPENDENT, 1).map_err(|e| win_err("independent samples", e))?;
        }
        Ok(t)
    }
}

pub struct WriterConfig<'a> {
    pub path: &'a Path,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    /// Constant-quality target 0…1 (see `configure_rate_control`).
    pub quality: f64,
    pub codec: Codec,
    pub audio: bool,
    pub manager: &'a IMFDXGIDeviceManager,
}

/// MP4 sink writer with one video stream (NV12 in, hardware H.264/HEVC out) and optionally one
/// audio stream (16-bit PCM in, AAC out).
pub struct Writer {
    sink: IMFSinkWriter,
    video: u32,
    audio: Option<u32>,
}

impl Writer {
    pub fn create(cfg: &WriterConfig) -> AppResult<Writer> {
        // Fragmented MP4 first (playable up to the last fragment after a crash, like macOS),
        // plain MP4 where the fragmented muxer is unavailable.
        match Self::create_with(cfg, &MFTranscodeContainerType_FMPEG4, cfg.codec) {
            Ok(w) => Ok(w),
            Err(e) => {
                tracing::warn!(error = %e, "fragmented MP4 unavailable; using plain MP4");
                Self::create_with(cfg, &MFTranscodeContainerType_MPEG4, cfg.codec).or_else(|e| {
                    if cfg.codec == Codec::Hevc {
                        // No HEVC encoder (the "HEVC Video Extensions" aren't installed).
                        tracing::warn!(error = %e, "HEVC encoder unavailable; recording in H.264");
                        Self::create_with(cfg, &MFTranscodeContainerType_FMPEG4, Codec::H264)
                            .or_else(|_| Self::create_with(cfg, &MFTranscodeContainerType_MPEG4, Codec::H264))
                    } else {
                        Err(e)
                    }
                })
            }
        }
    }

    fn create_with(cfg: &WriterConfig, container: &windows::core::GUID, codec: Codec) -> AppResult<Writer> {
        let _ = std::fs::remove_file(cfg.path);
        let attrs = attributes(4)?;
        // SAFETY: MF objects created and configured on this thread; all pointers valid.
        unsafe {
            attrs
                .SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)
                .map_err(|e| win_err("hw transforms", e))?;
            attrs.SetGUID(&MF_TRANSCODE_CONTAINERTYPE, container).map_err(|e| win_err("container", e))?;
            attrs
                .SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, cfg.manager)
                .map_err(|e| win_err("D3D manager", e))?;
            // Real-time capture: never block the capture thread; the sample pool drops frames
            // instead when the encoder falls behind.
            attrs.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1).map_err(|e| win_err("throttling", e))?;
            let sink = MFCreateSinkWriterFromURL(&HSTRING::from(cfg.path.as_os_str()), None, &attrs)
                .map_err(|e| win_err("MFCreateSinkWriterFromURL", e))?;

            let subtype = if codec == Codec::Hevc { MFVideoFormat_HEVC } else { MFVideoFormat_H264 };
            let out = video_type(&subtype, cfg.width, cfg.height, cfg.fps)?;
            out.SetUINT32(&MF_MT_AVG_BITRATE, cfg.bitrate).map_err(|e| win_err("bitrate", e))?;
            if codec == Codec::H264 {
                out.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32)
                    .map_err(|e| win_err("H.264 profile", e))?;
            }
            let video = sink.AddStream(&out).map_err(|e| win_err("video stream", e))?;
            let input = video_type(&MFVideoFormat_NV12, cfg.width, cfg.height, cfg.fps)?;
            // Rate control goes in as encoder creation parameters: many hardware encoders only
            // honor it when set before they are configured (set afterwards, it was accepted but
            // ignored — every quality produced the same constant-bitrate file).
            let params = attributes(3)?;
            // Spend encoder time on quality rather than speed (best effort).
            let _ = params.SetUINT32(&CODECAPI_AVEncCommonQualityVsSpeed, 100);
            params
                .SetUINT32(&CODECAPI_AVEncCommonRateControlMode, eAVEncCommonRateControlMode_Quality.0 as u32)
                .map_err(|e| win_err("rate control param", e))?;
            params
                .SetUINT32(&CODECAPI_AVEncCommonQuality, (cfg.quality.clamp(0.0, 1.0) * 100.0).round() as u32)
                .map_err(|e| win_err("quality param", e))?;
            if let Err(e) = sink.SetInputMediaType(video, &input, &params) {
                tracing::warn!(error = %e, "encoder refused quality parameters at creation");
                sink.SetInputMediaType(video, &input, None).map_err(|e| win_err("video encoder input", e))?;
            }
            let mode = configure_rate_control(&sink, video, cfg.quality, cfg.bitrate);
            let (effective_mode, effective_quality) = read_rate_control(&sink, video);
            tracing::info!(
                ?effective_mode,
                ?effective_quality,
                "recording: encoder reports (mode 0 = CBR, 1 = peak VBR, 2 = VBR, 3 = quality)"
            );
            tracing::info!(
                mode,
                quality = cfg.quality,
                bitrate = cfg.bitrate,
                ?codec,
                "recording: encoder rate control"
            );

            let audio = if cfg.audio {
                let aac = audio_type(&MFAudioFormat_AAC, true)?;
                let stream = sink.AddStream(&aac).map_err(|e| win_err("audio stream", e))?;
                let pcm = audio_type(&MFAudioFormat_PCM, false)?;
                sink.SetInputMediaType(stream, &pcm, None).map_err(|e| win_err("AAC encoder input", e))?;
                Some(stream)
            } else {
                None
            };
            sink.BeginWriting().map_err(|e| win_err("BeginWriting", e))?;
            Ok(Writer { sink, video, audio })
        }
    }

    pub fn has_audio(&self) -> bool {
        self.audio.is_some()
    }

    /// `time`/`duration` in 100 ns units from the start of the file.
    pub fn write_video(&self, sample: &IMFSample, time: i64, duration: i64) -> AppResult<()> {
        // SAFETY: valid sample owned by the caller.
        unsafe {
            sample.SetSampleTime(time).map_err(|e| win_err("sample time", e))?;
            sample.SetSampleDuration(duration).map_err(|e| win_err("sample duration", e))?;
            self.sink.WriteSample(self.video, sample).map_err(|e| win_err("write video", e))
        }
    }

    /// An NV12 frame in system memory (CPU conversion path), stride = width.
    pub fn write_video_nv12(&self, nv12: &[u8], time: i64, duration: i64) -> AppResult<()> {
        let bytes = nv12.len() as u32;
        // SAFETY: the buffer is locked for exactly `bytes` and filled within bounds.
        unsafe {
            let buffer = MFCreateMemoryBuffer(bytes).map_err(|e| win_err("video buffer", e))?;
            let mut data = std::ptr::null_mut();
            buffer.Lock(&mut data, None, None).map_err(|e| win_err("lock video buffer", e))?;
            std::ptr::copy_nonoverlapping(nv12.as_ptr(), data, nv12.len());
            buffer.Unlock().map_err(|e| win_err("unlock video buffer", e))?;
            buffer.SetCurrentLength(bytes).map_err(|e| win_err("video length", e))?;
            let sample = MFCreateSample().map_err(|e| win_err("video sample", e))?;
            sample.AddBuffer(&buffer).map_err(|e| win_err("video AddBuffer", e))?;
            self.write_video(&sample, time, duration)
        }
    }

    /// Interleaved stereo f32 at 48 kHz, starting at `time` (100 ns from the start of the file).
    pub fn write_audio(&self, samples: &[f32], time: i64) -> AppResult<()> {
        let Some(stream) = self.audio else { return Ok(()) };
        let frames = samples.len() / CHANNELS;
        if frames == 0 || time < 0 {
            return Ok(());
        }
        let bytes = (samples.len() * 2) as u32;
        // SAFETY: the buffer is locked for exactly `bytes` and written within bounds.
        unsafe {
            let buffer = MFCreateMemoryBuffer(bytes).map_err(|e| win_err("audio buffer", e))?;
            let mut data = std::ptr::null_mut();
            buffer.Lock(&mut data, None, None).map_err(|e| win_err("lock audio buffer", e))?;
            let out = std::slice::from_raw_parts_mut(data as *mut i16, samples.len());
            for (o, s) in out.iter_mut().zip(samples) {
                *o = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            }
            buffer.Unlock().map_err(|e| win_err("unlock audio buffer", e))?;
            buffer.SetCurrentLength(bytes).map_err(|e| win_err("audio length", e))?;
            let sample = MFCreateSample().map_err(|e| win_err("audio sample", e))?;
            sample.AddBuffer(&buffer).map_err(|e| win_err("audio AddBuffer", e))?;
            sample.SetSampleTime(time).map_err(|e| win_err("audio time", e))?;
            sample
                .SetSampleDuration(frames as i64 * 10_000_000 / SAMPLE_RATE as i64)
                .map_err(|e| win_err("audio duration", e))?;
            self.sink.WriteSample(stream, &sample).map_err(|e| win_err("write audio", e))
        }
    }

    /// Writes the index/last fragment and closes the file.
    pub fn finalize(self) -> AppResult<()> {
        // SAFETY: final call on the writer.
        unsafe { self.sink.Finalize() }.map_err(|e| win_err("Finalize", e))
    }
}

fn uint_variant(v: u32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_UI4,
                Anonymous: VARIANT_0_0_0 { ulVal: v },
                ..Default::default()
            }),
        },
    }
}

/// App quality level (the scale VideoToolbox uses on macOS: Leve 0.5, Boa 0.65, Ótima 0.75,
/// Máxima 0.85) → Media Foundation's H.264/HEVC quality, whose scale runs higher: calibration
/// on screen recordings showed 0.45 visibly soft; 0.60 was judged acceptable but not sharp
/// enough, so Ótima sits at 0.67.
pub fn encoder_quality(level: f64) -> f64 {
    const MAP: [(f64, f64); 4] = [(0.5, 0.50), (0.65, 0.58), (0.75, 0.67), (0.85, 0.80)];
    let level = level.clamp(MAP[0].0, MAP[3].0);
    MAP.windows(2)
        .find(|w| level <= w[1].0)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            a.1 + (level - a.0) / (b.0 - a.0) * (b.1 - a.1)
        })
        .unwrap_or(MAP[3].1)
}

/// Rate control of the video encoder, best first:
/// 1. constant quality: bits only where the picture changes (screen content compresses ~2×
///    smaller than a constant bitrate at equal fidelity);
/// 2. peak-constrained VBR: averages half the old constant rate, never exceeds it;
/// 3. the encoder's default (usually constant bitrate at `bitrate`).
fn configure_rate_control(sink: &IMFSinkWriter, stream: u32, quality: f64, bitrate: u32) -> &'static str {
    // SAFETY: COM calls on the encoder behind the sink writer, before BeginWriting.
    unsafe {
        let mut raw = std::ptr::null_mut();
        if sink
            .GetServiceForStream(stream, &windows::core::GUID::zeroed(), &ICodecAPI::IID, &mut raw)
            .is_err()
            || raw.is_null()
        {
            return "default (no ICodecAPI)";
        }
        let api = ICodecAPI::from_raw(raw);
        let set = |guid: &windows::core::GUID, v: u32| -> bool {
            api.IsSupported(guid).is_ok() && api.SetValue(guid, &uint_variant(v)).is_ok()
        };
        let _ = set(&CODECAPI_AVEncCommonQualityVsSpeed, 100);
        if set(&CODECAPI_AVEncCommonRateControlMode, eAVEncCommonRateControlMode_Quality.0 as u32)
            && set(&CODECAPI_AVEncCommonQuality, (quality.clamp(0.0, 1.0) * 100.0).round() as u32)
        {
            return "quality";
        }
        if set(&CODECAPI_AVEncCommonRateControlMode, eAVEncCommonRateControlMode_PeakConstrainedVBR.0 as u32)
            && set(&CODECAPI_AVEncCommonMeanBitRate, bitrate / 2)
        {
            let _ = set(&CODECAPI_AVEncCommonMaxBitRate, bitrate);
            return "peak-constrained VBR";
        }
        "default"
    }
}

/// What the encoder says it is doing: (rate control mode, quality), when it answers.
fn read_rate_control(sink: &IMFSinkWriter, stream: u32) -> (Option<u32>, Option<u32>) {
    // SAFETY: read-only ICodecAPI queries; UI4 VARIANTs need no cleanup.
    unsafe {
        let mut raw = std::ptr::null_mut();
        if sink
            .GetServiceForStream(stream, &windows::core::GUID::zeroed(), &ICodecAPI::IID, &mut raw)
            .is_err()
            || raw.is_null()
        {
            return (None, None);
        }
        let api = ICodecAPI::from_raw(raw);
        let get = |guid: &windows::core::GUID| {
            api.GetValue(guid).ok().map(|v| v.Anonymous.Anonymous.Anonymous.ulVal)
        };
        (get(&CODECAPI_AVEncCommonRateControlMode), get(&CODECAPI_AVEncCommonQuality))
    }
}

fn source_reader(path: &Path, video_processing: bool) -> AppResult<IMFSourceReader> {
    init()?;
    let attrs = attributes(1)?;
    // SAFETY: attribute set and reader creation with valid arguments.
    unsafe {
        if video_processing {
            attrs
                .SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)
                .map_err(|e| win_err("video processing", e))?;
        }
        MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), &attrs)
            .map_err(|e| win_err("MFCreateSourceReaderFromURL", e))
    }
}

const FIRST_VIDEO: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

fn frame_size(t: &IMFMediaType) -> Option<(u32, u32)> {
    // SAFETY: plain attribute read.
    let v = unsafe { t.GetUINT64(&MF_MT_FRAME_SIZE) }.ok()?;
    Some(((v >> 32) as u32, v as u32))
}

/// Duration and pixel size of a video file.
pub fn video_info(path: &Path) -> AppResult<Meta> {
    let reader = source_reader(path, false)?;
    // SAFETY: reads on a valid reader; the PROPVARIANT is owned and dropped here.
    unsafe {
        let duration = reader
            .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
            .ok()
            .and_then(|v| u64::try_from(&v).ok())
            .map(|hns| hns / 10_000);
        let size = reader.GetNativeMediaType(FIRST_VIDEO, 0).ok().and_then(|t| frame_size(&t));
        Ok(Meta { width: size.map(|s| s.0), height: size.map(|s| s.1), duration_ms: duration })
    }
}

/// JPEG thumbnail of a video: a frame half a second in (or the first one), at most
/// `max_pixels` on its longest side.
pub fn video_thumbnail(path: &Path, dest: &Path, max_pixels: u32) -> AppResult<()> {
    let reader = source_reader(path, true)?;
    // SAFETY: reader calls with valid arguments; the locked buffer is read within its pitch
    // and height and unlocked before returning.
    unsafe {
        let rgb = video_type(&MFVideoFormat_RGB32, 0, 0, 1)?;
        rgb.DeleteItem(&MF_MT_FRAME_SIZE).ok();
        rgb.DeleteItem(&MF_MT_FRAME_RATE).ok();
        rgb.DeleteItem(&MF_MT_PIXEL_ASPECT_RATIO).ok();
        reader.SetCurrentMediaType(FIRST_VIDEO, None, &rgb).map_err(|e| win_err("RGB32 output", e))?;
        let current = reader.GetCurrentMediaType(FIRST_VIDEO).map_err(|e| win_err("current type", e))?;
        let (w, h) =
            frame_size(&current).ok_or_else(|| AppError::Platform("video has no frame size".into()))?;

        // Half a second in (100 ns units); videos shorter than that keep the first frame.
        let position = PROPVARIANT::from(5_000_000i64);
        let _ = reader.SetCurrentPosition(&windows::core::GUID::zeroed(), &position);

        let mut sample = None;
        for _ in 0..30 {
            let mut flags = 0u32;
            reader
                .ReadSample(FIRST_VIDEO, 0, None, Some(&mut flags), None, Some(&mut sample))
                .map_err(|e| win_err("ReadSample", e))?;
            if sample.is_some() || flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        let sample = sample.ok_or_else(|| AppError::Platform("no video frame to thumbnail".into()))?;
        let buffer = sample.ConvertToContiguousBuffer().map_err(|e| win_err("contiguous buffer", e))?;
        let mut bgra = vec![0u8; (w * h * 4) as usize];
        if let Ok(b2) = buffer.cast::<IMF2DBuffer>() {
            let (mut line0, mut pitch) = (std::ptr::null_mut(), 0i32);
            b2.Lock2D(&mut line0, &mut pitch).map_err(|e| win_err("Lock2D", e))?;
            for y in 0..h as usize {
                let row = line0.offset(pitch as isize * y as isize);
                let src = std::slice::from_raw_parts(row, w as usize * 4);
                bgra[y * w as usize * 4..(y + 1) * w as usize * 4].copy_from_slice(src);
            }
            let _ = b2.Unlock2D();
        } else {
            let (mut data, mut len) = (std::ptr::null_mut(), 0u32);
            buffer.Lock(&mut data, None, Some(&mut len)).map_err(|e| win_err("Lock", e))?;
            let n = bgra.len().min(len as usize);
            bgra[..n].copy_from_slice(std::slice::from_raw_parts(data, n));
            let _ = buffer.Unlock();
        }
        // RGB32 from the video processor has no meaningful alpha.
        bgra.chunks_exact_mut(4).for_each(|p| p[3] = 255);
        let frame = crate::core::image::Frame::new(w, h, bgra)?;
        save_jpeg_thumbnail(&frame, dest, max_pixels)
    }
}

/// Scales a BGRA frame to fit `max_pixels` and writes it as JPEG.
pub fn save_jpeg_thumbnail(frame: &crate::core::image::Frame, dest: &Path, max_pixels: u32) -> AppResult<()> {
    let rgba = ::image::RgbaImage::from_raw(frame.width, frame.height, frame.to_rgba())
        .ok_or_else(|| AppError::Platform("invalid frame".into()))?;
    let thumb = ::image::DynamicImage::ImageRgba8(rgba).thumbnail(max_pixels, max_pixels).to_rgb8();
    thumb
        .save_with_format(dest, ::image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Platform(format!("cannot write thumbnail: {e}")))
}

/// 16 kHz mono f32 PCM of the first audio stream, streamed into `dest` (see
/// `platform::decode_audio`). Media Foundation decodes to float at the native rate; the
/// conversion to 16 kHz mono is `core::resample`.
pub fn decode_audio(
    path: &Path,
    dest: &Path,
    progress: &mut dyn FnMut(f64),
    cancel: &std::sync::atomic::AtomicBool,
) -> AppResult<u64> {
    use std::io::Write;
    const FIRST_AUDIO: u32 = MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32;
    let reader = source_reader(path, false).map_err(|e| AppError::Unsupported(e.to_string()))?;
    // SAFETY: reader calls with valid arguments; locked buffers are read within their length
    // and unlocked before the next read.
    unsafe {
        let duration_hns = reader
            .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
            .ok()
            .and_then(|v| u64::try_from(&v).ok())
            .unwrap_or(0);
        let _ = reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false);
        reader.SetStreamSelection(FIRST_AUDIO, true).map_err(|_| AppError::NoAudio)?;
        let wanted = MFCreateMediaType().map_err(|e| win_err("MFCreateMediaType", e))?;
        wanted.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio).map_err(|e| win_err("audio type", e))?;
        wanted.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_Float).map_err(|e| win_err("float type", e))?;
        reader
            .SetCurrentMediaType(FIRST_AUDIO, None, &wanted)
            .map_err(|e| AppError::Unsupported(format!("audio decoder: {e}")))?;
        let current = reader.GetCurrentMediaType(FIRST_AUDIO).map_err(|e| win_err("current type", e))?;
        let rate =
            current.GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND).map_err(|e| win_err("sample rate", e))?;
        let channels = current.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS).map_err(|e| win_err("channels", e))?;

        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        let mut file = std::io::BufWriter::new(opts.open(dest)?);
        let mut resampler =
            crate::core::resample::Resampler::new(rate, channels as u16, crate::core::resample::ASR_RATE);
        let mut out: Vec<f32> = Vec::new();
        let mut written = 0u64;
        let mut last_report = 0i64;
        let flush = |out: &mut Vec<f32>, file: &mut std::io::BufWriter<std::fs::File>| -> AppResult<u64> {
            let bytes: Vec<u8> = out.iter().flat_map(|v| v.to_le_bytes()).collect();
            file.write_all(&bytes)?;
            let n = out.len() as u64;
            out.clear();
            Ok(n)
        };
        loop {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(AppError::Cancelled);
            }
            let (mut flags, mut ts, mut sample) = (0u32, 0i64, None);
            reader
                .ReadSample(FIRST_AUDIO, 0, None, Some(&mut flags), Some(&mut ts), Some(&mut sample))
                .map_err(|e| win_err("ReadSample", e))?;
            if let Some(sample) = sample {
                let buffer =
                    sample.ConvertToContiguousBuffer().map_err(|e| win_err("contiguous buffer", e))?;
                let (mut data, mut len) = (std::ptr::null_mut(), 0u32);
                buffer.Lock(&mut data, None, Some(&mut len)).map_err(|e| win_err("Lock", e))?;
                let floats = std::slice::from_raw_parts(data as *const f32, len as usize / 4);
                resampler.push(floats, &mut out);
                let _ = buffer.Unlock();
                written += flush(&mut out, &mut file)?;
                if duration_hns > 0 && ts - last_report >= 20_000_000 {
                    last_report = ts;
                    progress((ts as f64 / duration_hns as f64).min(1.0));
                }
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }
        }
        resampler.finish(&mut out);
        written += flush(&mut out, &mut file)?;
        file.flush()?;
        progress(1.0);
        Ok(written)
    }
}
