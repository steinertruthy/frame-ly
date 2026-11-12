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
