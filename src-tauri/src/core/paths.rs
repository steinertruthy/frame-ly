//! File naming for captures. Used by screenshots (Stage 2) and recordings (Stage 3).

use chrono::{DateTime, TimeZone};
use std::path::{Path, PathBuf};

/// `FrameLy 2026-11-11 at 19.42.10.png` — sortable, readable, valid on every filesystem
/// (no `:` which Windows forbids and Finder displays as `/`).
pub fn capture_file_name<Tz: TimeZone>(at: &DateTime<Tz>, ext: &str) -> String
where
    Tz::Offset: std::fmt::Display,
{
    format!(
        "FrameLy {}.{}",
        at.format("%Y-%m-%d at %H.%M.%S"),
        ext.trim_start_matches('.')
    )
}

/// Returns `dir/name`, or `dir/stem (2).ext`, `(3)`… if the name is taken.
pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let p = Path::new(name);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = p
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|c| !c.exists())
        .expect("unbounded search always finds a free name")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn formats_name() {
        let t = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2026, 11, 11, 9, 5, 7)
            .unwrap();
        assert_eq!(
            capture_file_name(&t, ".mp4"),
            "FrameLy 2026-11-11 at 09.05.07.mp4"
        );
        assert_eq!(
            capture_file_name(&t, "png"),
            "FrameLy 2026-11-11 at 09.05.07.png"
        );
    }

    #[test]
    fn avoids_collisions() {
        let tmp = tempfile::tempdir().unwrap();
        let first = unique_path(tmp.path(), "a.png");
        assert_eq!(first, tmp.path().join("a.png"));
        std::fs::write(&first, b"x").unwrap();
        let second = unique_path(tmp.path(), "a.png");
        assert_eq!(second, tmp.path().join("a (2).png"));
        std::fs::write(&second, b"x").unwrap();
        assert_eq!(
            unique_path(tmp.path(), "a.png"),
            tmp.path().join("a (3).png")
        );
    }
}
