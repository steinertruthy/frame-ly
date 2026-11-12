//! Windows window styling: overlays sized to the monitor in physical pixels, excluded from
//! screen capture so FrameLy never captures its own UI.

use crate::platform::Display;
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use windows::Win32::UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE};

fn exclude_from_capture(window: &WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        // SAFETY: valid top-level window handle owned by this process.
        if let Err(e) = unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) } {
            tracing::debug!(error = %e, "SetWindowDisplayAffinity unavailable");
        }
    }
}

pub fn configure_overlay(window: &WebviewWindow, display: &Display) {
    let b = display.bounds;
    let _ = window.set_position(PhysicalPosition::new(b.x as i32, b.y as i32));
    let _ = window.set_size(PhysicalSize::new(display.pixel_width, display.pixel_height));
    exclude_from_capture(window);
}

pub fn configure_floating(window: &WebviewWindow) {
    exclude_from_capture(window);
}

pub fn configure_panel(window: &WebviewWindow) {
    exclude_from_capture(window);
}

pub fn show_panel(window: &WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}
