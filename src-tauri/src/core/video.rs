//! Output size and bitrate decisions for recordings. Pure and unit-tested.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    /// Plays everywhere (incl. WebView2 without extensions). Default.
    #[default]
    H264,
    /// ~40% smaller at the same quality; macOS/iOS/modern players.
    Hevc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Low,
    Medium,
    #[default]
    High,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Resolution {
    /// Source pixels (Retina-sharp).
    #[default]
    Native,
    #[serde(rename = "1080p")]
    P1080,
    #[serde(rename = "720p")]
    P720,
}

impl Codec {
    /// Largest frame the hardware encoders accept everywhere we ship (H.264 level 5.2).
    fn max_size(self) -> (u32, u32) {
        match self {
            Codec::H264 => (4096, 2304),
            Codec::Hevc => (8192, 4320),
        }
    }
}

/// Final encoded size for a source of `src` pixels: applies the resolution preset (by height,
/// keeping aspect), the codec's maximum, and rounds down to even numbers (4:2:0 chroma).
pub fn output_size(src: (u32, u32), resolution: Resolution, codec: Codec) -> (u32, u32) {
    let (w, h) = (src.0.max(2) as f64, src.1.max(2) as f64);
    let mut scale: f64 = match resolution {
        Resolution::Native => 1.0,
        Resolution::P1080 => (1080.0 / h).min(1.0),
        Resolution::P720 => (720.0 / h).min(1.0),
    };
    let (mw, mh) = codec.max_size();
    scale = scale.min(mw as f64 / w).min(mh as f64 / h);
    let even = |v: f64| ((v.floor() as u32) & !1).max(2);
    (even(w * scale), even(h * scale))
}

/// Average bitrate in bits/s from a bits-per-pixel-per-frame budget, clamped to sane bounds.
pub fn bitrate(size: (u32, u32), fps: u32, quality: Quality, codec: Codec) -> u32 {
    let bpp = match quality {
        Quality::Low => 0.04,
        Quality::Medium => 0.07,
        Quality::High => 0.11,
        Quality::Max => 0.18,
    } * match codec {
        Codec::H264 => 1.0,
        Codec::Hevc => 0.6,
    };
    let bits = size.0 as f64 * size.1 as f64 * fps as f64 * bpp;
    bits.clamp(1_500_000.0, 80_000_000.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_keeps_retina_pixels() {
        assert_eq!(output_size((3024, 1964), Resolution::Native, Codec::H264), (3024, 1964));
    }

    #[test]
    fn presets_scale_by_height_and_keep_aspect() {
        assert_eq!(output_size((3024, 1964), Resolution::P1080, Codec::H264), (1662, 1080));
        assert_eq!(output_size((1920, 1080), Resolution::P720, Codec::H264), (1280, 720));
        // Never upscale.
        assert_eq!(output_size((800, 600), Resolution::P1080, Codec::H264), (800, 600));
    }

    #[test]
    fn respects_codec_limits_and_even_sizes() {
        // 5K display: H.264 can't encode it natively.
        assert_eq!(output_size((5120, 2880), Resolution::Native, Codec::H264), (4096, 2304));
        assert_eq!(output_size((5120, 2880), Resolution::Native, Codec::Hevc), (5120, 2880));
        // Odd region sizes are rounded down to even.
        assert_eq!(output_size((801, 451), Resolution::Native, Codec::H264), (800, 450));
    }

    #[test]
    fn bitrates_are_sensible() {
        let b = bitrate((1920, 1080), 30, Quality::High, Codec::H264);
        assert!((6_000_000..=8_000_000).contains(&b), "{b}");
        assert!(bitrate((1920, 1080), 30, Quality::High, Codec::Hevc) < b);
        assert_eq!(bitrate((100, 100), 24, Quality::Low, Codec::H264), 1_500_000);
        assert_eq!(bitrate((8192, 4320), 60, Quality::Max, Codec::Hevc), 80_000_000);
    }
}
