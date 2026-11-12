//! `framely://` URI scheme: serves in-memory images to FrameLy's own web views without
//! writing temp files or opening the asset protocol to the file system.

use crate::{screenshot, AppState};
use tauri::{
    http::{header, Response, StatusCode},
    AppHandle, Manager,
};

fn image(content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body)
        .expect("static response parts are valid")
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).expect("valid response")
}

pub fn respond(app: &AppHandle, path: &str) -> Response<Vec<u8>> {
    let path = percent_encoding::percent_decode_str(path).decode_utf8_lossy();
    let path = path.trim_start_matches('/');
    if let Some(display_id) = path.strip_prefix("frame/") {
        return screenshot::frozen_frame_bmp(app, display_id)
            .map(|b| image("image/bmp", b))
            .unwrap_or_else(not_found);
    }
    if let Some(id) = path.strip_prefix("thumb/").and_then(|s| s.parse::<i64>().ok()) {
        return match crate::library::thumbnail(app, id) {
            Ok(bytes) => image("image/jpeg", bytes),
            Err(e) => {
                tracing::debug!(id, error = %e, "thumbnail unavailable");
                not_found()
            }
        };
    }
    if path == "edit-source" || path.starts_with("edit-source/") {
        return crate::editor::source_bytes()
            .map(|b| image(if b.starts_with(b"\x89PNG") { "image/png" } else { "image/jpeg" }, b))
            .unwrap_or_else(not_found);
    }
    if path == "last-recording-thumb" {
        let thumb =
            app.state::<AppState>().recordings.last.lock().unwrap().as_ref().and_then(|l| l.thumb.clone());
        return thumb
            .and_then(|t| std::fs::read(t).ok())
            .map(|b| image("image/jpeg", b))
            .unwrap_or_else(not_found);
    }
    if path == "last" || path.starts_with("last/") {
        let last = app.state::<AppState>().screenshots.last.lock().unwrap().clone();
        return last.map(|l| image("image/png", l.png.as_ref().clone())).unwrap_or_else(not_found);
    }
    not_found()
}
