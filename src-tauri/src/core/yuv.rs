//! BGRA → NV12 (BT.709, limited range) with crop and aspect-preserving scale, on the CPU.
//! The Windows recorder uses it when the GPU video processor isn't usable on a machine.
//! Pure and unit-tested.

#![cfg_attr(not(windows), allow(dead_code))]

/// Source pixels: BGRA rows of `stride` bytes.
pub struct Bgra<'a> {
    pub data: &'a [u8],
    pub stride: usize,
    pub width: u32,
    pub height: u32,
}

/// `(x, y, width, height)` in pixels.
pub type Rect = (u32, u32, u32, u32);

/// Largest rectangle of the source's aspect, centered in `out`, with even coordinates.
pub fn fit(src: (u32, u32), out: (u32, u32)) -> Rect {
    let (sw, sh) = (src.0.max(1) as f64, src.1.max(1) as f64);
    let scale = (out.0 as f64 / sw).min(out.1 as f64 / sh);
    let w = ((sw * scale).round() as u32 & !1).clamp(2, out.0);
    let h = ((sh * scale).round() as u32 & !1).clamp(2, out.1);
    (((out.0 - w) / 2) & !1, ((out.1 - h) / 2) & !1, w, h)
}

#[inline]
fn yuv(b: u8, g: u8, r: u8) -> (i32, i32, i32) {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    // BT.709 limited range, 8-bit fixed point (×1024).
    let y = 16 + ((187 * r + 629 * g + 63 * b + 512) >> 10);
    let u = 128 + ((-103 * r - 347 * g + 450 * b + 512) >> 10);
    let v = 128 + ((450 * r - 409 * g - 41 * b + 512) >> 10);
    (y, u, v)
}

/// Converts `crop` of `src` into an NV12 frame of `out` size (Y plane then interleaved UV,
/// both with stride = width), letterboxed in black. `out` must be even.
pub fn bgra_to_nv12(src: &Bgra, crop: Rect, out: (u32, u32)) -> Vec<u8> {
    let (ow, oh) = (out.0 as usize, out.1 as usize);
    let mut nv12 = vec![0u8; ow * oh * 3 / 2];
    let (y_plane, uv_plane) = nv12.split_at_mut(ow * oh);
    y_plane.fill(16);
    uv_plane.fill(128);

    let (cx, cy) = (crop.0.min(src.width.saturating_sub(1)), crop.1.min(src.height.saturating_sub(1)));
    let cw = crop.2.min(src.width - cx).max(1);
    let ch = crop.3.min(src.height - cy).max(1);
    let (dx, dy, dw, dh) = fit((cw, ch), out);
    let (dx, dy, dw, dh) = (dx as usize, dy as usize, dw as usize, dh as usize);

    // Nearest source column/row for every destination column/row.
    let xs: Vec<usize> = (0..dw).map(|i| cx as usize + (i * cw as usize / dw).min(cw as usize - 1)).collect();
    let ys: Vec<usize> = (0..dh).map(|j| cy as usize + (j * ch as usize / dh).min(ch as usize - 1)).collect();
    let px = |x: usize, y: usize| {
        let o = y * src.stride + x * 4;
        (src.data[o], src.data[o + 1], src.data[o + 2])
    };

    for j in (0..dh).step_by(2) {
        for i in (0..dw).step_by(2) {
            let (mut su, mut sv) = (0, 0);
            for (oy, ox) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let (jj, ii) = ((j + oy).min(dh - 1), (i + ox).min(dw - 1));
                let (b, g, r) = px(xs[ii], ys[jj]);
                let (y, u, v) = yuv(b, g, r);
                y_plane[(dy + jj) * ow + dx + ii] = y.clamp(16, 235) as u8;
                su += u;
                sv += v;
            }
            let uv = ((dy + j) / 2) * ow + (dx + i);
            uv_plane[uv] = (su / 4).clamp(16, 240) as u8;
            uv_plane[uv + 1] = (sv / 4).clamp(16, 240) as u8;
        }
    }
    nv12
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, bgr: (u8, u8, u8)) -> Vec<u8> {
        (0..w * h).flat_map(|_| [bgr.0, bgr.1, bgr.2, 255]).collect()
    }

    #[test]
    fn colors_follow_bt709_limited_range() {
        assert_eq!(yuv(255, 255, 255).0, 235);
        assert_eq!(yuv(0, 0, 0), (16, 128, 128));
        let (y, u, v) = yuv(0, 0, 255); // red
        assert!((62..=64).contains(&y) && u < 128 && v > 230, "{y} {u} {v}");
    }

    #[test]
    fn letterboxes_and_scales() {
        let data = solid(400, 100, (255, 255, 255));
        let src = Bgra { data: &data, stride: 400 * 4, width: 400, height: 100 };
        let nv12 = bgra_to_nv12(&src, (0, 0, 400, 100), (200, 100));
        assert_eq!(nv12.len(), 200 * 150);
        // 4:1 source into 2:1 frame → 200×50 image centered vertically (rows 24..74).
        assert_eq!(nv12[10 * 200 + 100], 16, "black bar");
        assert_eq!(nv12[50 * 200 + 100], 235, "white image");
        assert_eq!(nv12[90 * 200 + 100], 16, "black bar");
    }

    #[test]
    fn crops_the_requested_region() {
        let mut data = solid(100, 100, (0, 0, 0));
        // White square at (50..100, 50..100).
        for y in 50..100 {
            for x in 50..100 {
                data[(y * 100 + x) * 4..(y * 100 + x) * 4 + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let src = Bgra { data: &data, stride: 400, width: 100, height: 100 };
        let nv12 = bgra_to_nv12(&src, (50, 50, 50, 50), (50, 50));
        assert!(nv12[..50 * 50].iter().all(|&y| y == 235));
    }

    #[test]
    fn fit_keeps_even_sizes() {
        assert_eq!(fit((1920, 1080), (1920, 1080)), (0, 0, 1920, 1080));
        assert_eq!(fit((1000, 500), (1920, 1080)), (0, 60, 1920, 960));
    }
}
