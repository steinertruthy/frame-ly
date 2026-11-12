//! macOS window styling done through AppKit (via FrameLyKit), on the main thread.

use crate::platform::Display;
use tauri::WebviewWindow;

extern "C" {
    fn fl_configure_overlay(ns_window: *mut std::ffi::c_void, display_id: u32);
    fn fl_configure_floating(ns_window: *mut std::ffi::c_void);
}

fn on_main(window: &WebviewWindow, f: impl FnOnce(*mut std::ffi::c_void) + Send + 'static) {
    let Ok(ptr) = window.ns_window() else { return };
    let ptr = ptr as usize; // raw pointers aren't Send; the window outlives this call.
    let _ = window.run_on_main_thread(move || f(ptr as *mut std::ffi::c_void));
}

/// Covers the whole display (menu bar and Dock included) at screen-saver level.
pub fn configure_overlay(window: &WebviewWindow, display: &Display) {
    let id: u32 = display.id.parse().unwrap_or(0);
    // SAFETY: valid NSWindow pointer used on the main thread.
    on_main(window, move |w| unsafe { fl_configure_overlay(w, id) });
}

/// Floating level, visible on every Space (post-capture preview).
pub fn configure_floating(window: &WebviewWindow) {
    // SAFETY: valid NSWindow pointer used on the main thread.
    on_main(window, |w| unsafe { fl_configure_floating(w) });
}
