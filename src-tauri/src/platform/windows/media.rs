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
        System::Com::{CoInitializeEx, StructuredStorage::PROPVARIANT, COINIT_MULTITHREADED},
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
            sink.SetInputMediaType(video, &input, None).map_err(|e| win_err("video encoder input", e))?;

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
