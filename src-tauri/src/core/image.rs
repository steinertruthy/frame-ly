//! Pixel buffers and encoders for screenshots. Pure Rust, no OS calls, fully unit-tested.
//!
//! Every platform backend returns frames as tightly packed **BGRA8** (the native layout of
//! both CoreGraphics little-endian bitmaps and Windows.Graphics.Capture), so cropping and
//! encoding are shared.

use super::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// BGRA8, `width * 4` bytes per row, no padding.
    pub bgra: Vec<u8>,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Frame({}x{})", self.width, self.height)
    }
}

impl Frame {
    pub fn new(width: u32, height: u32, bgra: Vec<u8>) -> AppResult<Self> {
        if width == 0 || height == 0 || bgra.len() != width as usize * height as usize * 4 {
            return Err(AppError::Platform(format!(
                "invalid frame buffer: {width}x{height} with {} bytes",
                bgra.len()
            )));
        }
        Ok(Self { width, height, bgra })
    }

    /// Repacks a buffer that may have row padding (`stride` > width * 4).
    pub fn from_strided(width: u32, height: u32, stride: usize, data: &[u8]) -> AppResult<Self> {
        let row = width as usize * 4;
        if stride < row || data.len() < stride * (height as usize - 1) + row {
            return Err(AppError::Platform("frame stride/size mismatch".into()));
        }
        let mut bgra = Vec::with_capacity(row * height as usize);
        for y in 0..height as usize {
            bgra.extend_from_slice(&data[y * stride..y * stride + row]);
        }
        Self::new(width, height, bgra)
    }

    /// Crops to `rect` (pixels), clamped to the frame. Errors if nothing remains.
    pub fn crop(&self, rect: PixelRect) -> AppResult<Frame> {
        let x0 = rect.x.min(self.width);
        let y0 = rect.y.min(self.height);
        let x1 = rect.x.saturating_add(rect.width).min(self.width);
        let y1 = rect.y.saturating_add(rect.height).min(self.height);
        if x1 <= x0 || y1 <= y0 {
            return Err(AppError::Platform("selection is empty".into()));
        }
        let (w, h) = (x1 - x0, y1 - y0);
        let src_row = self.width as usize * 4;
        let mut out = Vec::with_capacity(w as usize * h as usize * 4);
        for y in y0..y1 {
            let start = y as usize * src_row + x0 as usize * 4;
            out.extend_from_slice(&self.bgra[start..start + w as usize * 4]);
        }
        Frame::new(w, h, out)
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = self.bgra.clone();
        for px in rgba.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        rgba
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A selection made in an overlay, in the overlay's CSS pixels, plus the overlay viewport size.
/// Converting with the frame size keeps this correct regardless of OS scaling rules
/// (points on macOS, DPI-scaled pixels on Windows).
pub fn css_to_pixels(sel: (f64, f64, f64, f64), viewport: (f64, f64), frame: (u32, u32)) -> PixelRect {
    let (sx, sy) = (frame.0 as f64 / viewport.0.max(1.0), frame.1 as f64 / viewport.1.max(1.0));
    let (x, y, w, h) = sel;
    let (x0, y0) = ((x * sx).round().max(0.0), (y * sy).round().max(0.0));
    let (x1, y1) = (((x + w) * sx).round().max(0.0), ((y + h) * sy).round().max(0.0));
    PixelRect {
        x: x0 as u32,
        y: y0 as u32,
        width: (x1 - x0).max(0.0) as u32,
        height: (y1 - y0).max(0.0) as u32,
    }
}

/// PNG with fast compression: a 5K screenshot encodes in well under 100 ms in release builds.
pub fn encode_png(frame: &Frame) -> AppResult<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, frame.width, frame.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut w = enc.write_header().map_err(|e| AppError::Platform(e.to_string()))?;
        w.write_image_data(&frame.to_rgba()).map_err(|e| AppError::Platform(e.to_string()))?;
    }
    Ok(out)
}

/// Uncompressed, top-down 32-bit BMP. Used only to hand a frozen frame to the selection
/// overlay inside the app: zero encoding cost, and WebKit/WebView2 decode it natively.
pub fn encode_bmp(frame: &Frame) -> Vec<u8> {
    const FILE_HEADER: u32 = 14;
    const INFO_HEADER: u32 = 40;
    let data_len = frame.bgra.len() as u32;
    let mut out = Vec::with_capacity((FILE_HEADER + INFO_HEADER + data_len) as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(FILE_HEADER + INFO_HEADER + data_len).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(FILE_HEADER + INFO_HEADER).to_le_bytes());
    out.extend_from_slice(&INFO_HEADER.to_le_bytes());
    out.extend_from_slice(&(frame.width as i32).to_le_bytes());
    out.extend_from_slice(&(-(frame.height as i32)).to_le_bytes()); // negative = top-down
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bpp
    out.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&[0; 16]); // resolution + palette fields
    out.extend_from_slice(&frame.bgra);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4x3 frame where pixel (x, y) = B:x G:y R:7 A:255.
    fn sample() -> Frame {
        let mut v = Vec::new();
        for y in 0..3u8 {
            for x in 0..4u8 {
                v.extend_from_slice(&[x, y, 7, 255]);
            }
        }
        Frame::new(4, 3, v).unwrap()
    }

    #[test]
    fn rejects_bad_buffers() {
        assert!(Frame::new(2, 2, vec![0; 15]).is_err());
        assert!(Frame::new(0, 2, vec![]).is_err());
    }

    #[test]
    fn repacks_strided_rows() {
        let mut data = Vec::new();
        for y in 0..2u8 {
            data.extend_from_slice(&[y; 8]); // 2 px
            data.extend_from_slice(&[0xEE; 8]); // padding
        }
        let f = Frame::from_strided(2, 2, 16, &data).unwrap();
        assert_eq!(f.bgra, [vec![0u8; 8], vec![1u8; 8]].concat());
    }

    #[test]
    fn crops_and_clamps() {
        let f = sample();
        let c = f.crop(PixelRect { x: 1, y: 1, width: 2, height: 2 }).unwrap();
        assert_eq!((c.width, c.height), (2, 2));
        assert_eq!(&c.bgra[0..4], &[1, 1, 7, 255]);
        assert_eq!(&c.bgra[12..16], &[2, 2, 7, 255]);
        let clamped = f.crop(PixelRect { x: 3, y: 2, width: 10, height: 10 }).unwrap();
        assert_eq!((clamped.width, clamped.height), (1, 1));
        assert!(f.crop(PixelRect { x: 9, y: 0, width: 2, height: 2 }).is_err());
    }

    #[test]
    fn converts_css_selection_on_retina() {
        // 1512pt-wide overlay over a 3024px frame (2x).
        let r = css_to_pixels((10.0, 20.0, 100.5, 50.0), (1512.0, 982.0), (3024, 1964));
        assert_eq!(r, PixelRect { x: 20, y: 40, width: 201, height: 100 });
        // Negative origins (dragged past the edge) clamp to 0.
        let r = css_to_pixels((-5.0, -5.0, 10.0, 10.0), (100.0, 100.0), (100, 100));
        assert_eq!(r, PixelRect { x: 0, y: 0, width: 5, height: 5 });
    }

    #[test]
    fn png_roundtrip_preserves_pixels() {
        let f = sample();
        let bytes = encode_png(&f).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (4, 3));
        assert_eq!(&buf[..info.buffer_size()], f.to_rgba().as_slice());
        assert_eq!(&buf[4..8], &[7, 0, 1, 255]); // RGBA of pixel (1, 0)
    }

    /// `cargo test --release -- --ignored --nocapture png_encode_speed`
    #[test]
    #[ignore]
    fn png_encode_speed() {
        let (w, h) = (3024u32, 1964u32);
        // Screen-like content: flat areas with some detail.
        let bgra: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                let (x, y) = (i % w, i / w);
                let v = if (x / 40 + y / 24) % 7 == 0 { (x ^ y) as u8 } else { 40 };
                [v, v.wrapping_add(10), 60, 255]
            })
            .collect();
        let f = Frame::new(w, h, bgra).unwrap();
        let t = std::time::Instant::now();
        let png = encode_png(&f).unwrap();
        println!("encode {w}x{h}: {:?}, {} KB", t.elapsed(), png.len() / 1024);
    }

    #[test]
    fn bmp_header_is_top_down_32bpp() {
        let f = sample();
        let b = encode_bmp(&f);
        assert_eq!(&b[0..2], b"BM");
        assert_eq!(u32::from_le_bytes(b[2..6].try_into().unwrap()) as usize, b.len());
        assert_eq!(i32::from_le_bytes(b[22..26].try_into().unwrap()), -3);
        assert_eq!(u16::from_le_bytes(b[28..30].try_into().unwrap()), 32);
        assert_eq!(&b[54..58], &f.bgra[0..4]);
    }
}
