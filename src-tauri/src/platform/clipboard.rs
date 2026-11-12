//! Copying images to the system clipboard.
//!
//! macOS: the PNG we already encoded goes straight onto NSPasteboard (`public.png`).
//! Windows: `arboard` writes a DIB (what Paint, Office and browsers paste).

use crate::core::{
    errors::{AppError, AppResult},
    image::Frame,
};

#[cfg(target_os = "macos")]
pub fn copy_image(png: &[u8], _frame: &Frame) -> AppResult<()> {
    extern "C" {
        fn fl_copy_png(bytes: *const u8, len: usize) -> bool;
    }
    // SAFETY: pointer/len describe a live slice for the duration of the call; Swift copies it.
    if unsafe { fl_copy_png(png.as_ptr(), png.len()) } {
        Ok(())
    } else {
        Err(AppError::Platform("could not write to the pasteboard".into()))
    }
}

#[cfg(windows)]
pub fn copy_image(_png: &[u8], frame: &Frame) -> AppResult<()> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| AppError::Platform(format!("clipboard: {e}")))?;
    clipboard
        .set_image(arboard::ImageData {
            width: frame.width as usize,
            height: frame.height as usize,
            bytes: std::borrow::Cow::Owned(frame.to_rgba()),
        })
        .map_err(|e| AppError::Platform(format!("clipboard: {e}")))
}

#[cfg(target_os = "macos")]
pub fn copy_file(path: &std::path::Path) -> AppResult<()> {
    extern "C" {
        fn fl_copy_file(path: *const std::ffi::c_char) -> bool;
    }
    let c = std::ffi::CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| AppError::Platform("invalid path".into()))?;
    // SAFETY: NUL-terminated string valid for the call.
    if unsafe { fl_copy_file(c.as_ptr()) } {
        Ok(())
    } else {
        Err(AppError::Platform("could not write to the pasteboard".into()))
    }
}

#[cfg(windows)]
pub fn copy_file(path: &std::path::Path) -> AppResult<()> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| AppError::Platform(format!("clipboard: {e}")))?;
    clipboard.set().file_list(&[path]).map_err(|e| AppError::Platform(format!("clipboard: {e}")))
}

/// Copies an image file (PNG/JPEG) as image data.
pub fn copy_image_file(path: &std::path::Path) -> AppResult<()> {
    copy_image_bytes(std::fs::read(path)?)
}

/// Copies encoded image bytes (PNG/JPEG) as image data.
pub fn copy_image_bytes(bytes: Vec<u8>) -> AppResult<()> {
    let is_png = bytes.starts_with(b"\x89PNG");
    let rgba = ::image::load_from_memory(&bytes)
        .map_err(|e| AppError::Platform(format!("cannot read image: {e}")))?
        .to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut bgra = rgba.into_raw();
    for px in bgra.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let frame = Frame::new(w, h, bgra)?;
    let png = if is_png { bytes } else { crate::core::image::encode_png(&frame)? };
    copy_image(&png, &frame)
}
