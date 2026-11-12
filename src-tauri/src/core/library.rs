//! Media library index (SQLite). The folders on disk are the source of truth; the index caches
//! metadata (size, resolution, duration, thumbnail) and makes search/sort/filter instant.
//! `reconcile` brings it in line with the disk; metadata probing is injected, so this module
//! stays pure and testable.

use super::errors::{AppError, AppResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Video,
    Image,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Video => "video",
            Kind::Image => "image",
        }
    }

    fn parse(s: &str) -> Kind {
        if s == "video" {
            Kind::Video
        } else {
            Kind::Image
        }
    }

    /// File types the library shows.
    pub fn of(path: &Path) -> Option<Kind> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "mp4" | "mov" | "m4v" => Some(Kind::Video),
            "png" | "jpg" | "jpeg" => Some(Kind::Image),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Meta {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub kind: Kind,
    pub path: String,
    pub name: String,
    /// Unix ms.
    pub created_at: i64,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    #[default]
    All,
    Video,
    Image,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Sort {
    #[default]
    Newest,
    Oldest,
    Name,
    Size,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Query {
    pub search: String,
    pub filter: Filter,
    pub sort: Sort,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub videos: u64,
    pub video_bytes: u64,
    pub images: u64,
    pub image_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Changes {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}

impl Changes {
    pub fn any(&self) -> bool {
        self.added + self.updated + self.removed > 0
    }
}

/// A media file found on disk.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub kind: Kind,
    pub size: u64,
    pub mtime_ms: i64,
    pub created_ms: i64,
}

fn ms(t: std::io::Result<std::time::SystemTime>) -> Option<i64> {
    t.ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as i64)
}

pub fn file_entry(path: &Path) -> Option<FileEntry> {
    let kind = Kind::of(path)?;
    let md = std::fs::metadata(path).ok().filter(|m| m.is_file())?;
    let mtime_ms = ms(md.modified()).unwrap_or(0);
    Some(FileEntry {
        path: path.to_path_buf(),
        kind,
        size: md.len(),
        mtime_ms,
        created_ms: ms(md.created()).unwrap_or(mtime_ms),
    })
}

/// Media files directly inside `dirs` (not recursive; hidden files skipped).
pub fn scan(dirs: &[PathBuf]) -> Vec<FileEntry> {
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for e in entries.flatten() {
            let path = e.path();
            if path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.')) {
                continue;
            }
            if let Some(f) = file_entry(&path) {
                out.push(f);
            }
        }
    }
    out
}

pub struct Library {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
    id          INTEGER PRIMARY KEY,
    kind        TEXT    NOT NULL,
    path        TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    mtime       INTEGER NOT NULL,
    size        INTEGER NOT NULL,
    width       INTEGER,
    height      INTEGER,
    duration_ms INTEGER
);
CREATE INDEX IF NOT EXISTS items_created ON items(created_at);
PRAGMA user_version = 1;
";

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

impl Library {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path).map_err(db)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;").map_err(db)?;
        conn.execute_batch(SCHEMA).map_err(db)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        Self { conn }
    }

    /// Makes the index match `files`: inserts new files, re-probes changed ones (size/mtime),
    /// drops missing ones. `probe` is called only for new or changed files.
    pub fn reconcile(
        &mut self,
        files: &[FileEntry],
        probe: &dyn Fn(&FileEntry) -> Meta,
    ) -> AppResult<Changes> {
        let existing: HashMap<String, (i64, i64, u64)> = {
            let mut stmt = self.conn.prepare("SELECT path, id, mtime, size FROM items").map_err(db)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? as u64)))
                })
                .map_err(db)?;
            rows.collect::<Result<_, _>>().map_err(db)?
        };
        let mut changes = Changes::default();
        let tx = self.conn.transaction().map_err(db)?;
        let mut seen = std::collections::HashSet::new();
        for f in files {
            let key = f.path.display().to_string();
            seen.insert(key.clone());
            match existing.get(&key) {
                Some((_, mtime, size)) if *mtime == f.mtime_ms && *size == f.size => {}
                Some((id, ..)) => {
                    let m = probe(f);
                    tx.execute(
                        "UPDATE items SET mtime=?2, size=?3, width=?4, height=?5, duration_ms=?6 WHERE id=?1",
                        params![
                            id,
                            f.mtime_ms,
                            f.size as i64,
                            m.width,
                            m.height,
                            m.duration_ms.map(|d| d as i64)
                        ],
                    )
                    .map_err(db)?;
                    changes.updated += 1;
                }
                None => {
                    let m = probe(f);
                    tx.execute(
                        "INSERT INTO items (kind, path, name, created_at, mtime, size, width, height, duration_ms)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            f.kind.as_str(),
                            key,
                            file_name(&f.path),
                            f.created_ms,
                            f.mtime_ms,
                            f.size as i64,
                            m.width,
                            m.height,
                            m.duration_ms.map(|d| d as i64)
                        ],
                    )
                    .map_err(db)?;
                    changes.added += 1;
                }
            }
        }
        for (path, (id, ..)) in &existing {
            if !seen.contains(path) {
                tx.execute("DELETE FROM items WHERE id=?1", params![id]).map_err(db)?;
                changes.removed += 1;
            }
        }
        tx.commit().map_err(db)?;
        Ok(changes)
    }

    pub fn query(&self, q: &Query) -> AppResult<Vec<Item>> {
        let order = match q.sort {
            Sort::Newest => "created_at DESC",
            Sort::Oldest => "created_at ASC",
            Sort::Name => "name COLLATE NOCASE ASC",
            Sort::Size => "size DESC",
        };
        let kind = match q.filter {
            Filter::All => None,
            Filter::Video => Some("video"),
            Filter::Image => Some("image"),
        };
        let search =
            format!("%{}%", q.search.trim().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let sql = format!(
            "SELECT id, kind, path, name, created_at, size, width, height, duration_ms FROM items
             WHERE (?1 IS NULL OR kind = ?1) AND name LIKE ?2 ESCAPE '\\'
             ORDER BY {order}, id DESC LIMIT ?3"
        );
        let mut stmt = self.conn.prepare(&sql).map_err(db)?;
        let rows = stmt
            .query_map(params![kind, search, q.limit.map(i64::from).unwrap_or(-1)], row_to_item)
            .map_err(db)?;
        rows.collect::<Result<_, _>>().map_err(db)
    }

    pub fn get(&self, id: i64) -> AppResult<Item> {
        self.conn
            .query_row(
                "SELECT id, kind, path, name, created_at, size, width, height, duration_ms FROM items WHERE id=?1",
                params![id],
                row_to_item,
            )
            .optional()
            .map_err(db)?
            .ok_or(AppError::NotFound)
    }

    pub fn stats(&self) -> AppResult<Stats> {
        let mut s = Stats::default();
        let mut stmt = self
            .conn
            .prepare("SELECT kind, COUNT(*), COALESCE(SUM(size), 0) FROM items GROUP BY kind")
            .map_err(db)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64, r.get::<_, i64>(2)? as u64))
            })
            .map_err(db)?;
        for row in rows {
            let (kind, count, bytes) = row.map_err(db)?;
            match Kind::parse(&kind) {
                Kind::Video => (s.videos, s.video_bytes) = (count, bytes),
                Kind::Image => (s.images, s.image_bytes) = (count, bytes),
            }
        }
        Ok(s)
    }

    /// Renames the file on disk and in the index. `new_stem` is the name without extension.
    pub fn rename(&mut self, id: i64, new_stem: &str) -> AppResult<Item> {
        let item = self.get(id)?;
        let stem = validate_stem(new_stem)?;
        let old = PathBuf::from(&item.path);
        let ext = old.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let new = old.with_file_name(format!("{stem}{ext}"));
        if new == old {
            return Ok(item);
        }
        if new.exists() {
            return Err(AppError::InvalidName("a file with that name already exists".into()));
        }
        std::fs::rename(&old, &new)?;
        self.conn
            .execute(
                "UPDATE items SET path=?2, name=?3 WHERE id=?1",
                params![id, new.display().to_string(), file_name(&new)],
            )
            .map_err(db)?;
        self.get(id)
    }

    pub fn remove(&mut self, id: i64) -> AppResult<()> {
        self.conn.execute("DELETE FROM items WHERE id=?1", params![id]).map_err(db)?;
        Ok(())
    }

    /// Adds or refreshes one file right after FrameLy creates it.
    pub fn upsert(&mut self, path: &Path, meta: Meta) -> AppResult<()> {
        let Some(f) = file_entry(path) else { return Ok(()) };
        self.conn
            .execute(
                "INSERT INTO items (kind, path, name, created_at, mtime, size, width, height, duration_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(path) DO UPDATE SET mtime=excluded.mtime, size=excluded.size,
                   width=excluded.width, height=excluded.height, duration_ms=excluded.duration_ms",
                params![
                    f.kind.as_str(),
                    f.path.display().to_string(),
                    file_name(&f.path),
                    f.created_ms,
                    f.mtime_ms,
                    f.size as i64,
                    meta.width,
                    meta.height,
                    meta.duration_ms.map(|d| d as i64)
                ],
            )
            .map_err(db)?;
        Ok(())
    }
}

fn row_to_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: r.get(0)?,
        kind: Kind::parse(&r.get::<_, String>(1)?),
        path: r.get(2)?,
        name: r.get(3)?,
        created_at: r.get(4)?,
        size: r.get::<_, i64>(5)? as u64,
        width: r.get(6)?,
        height: r.get(7)?,
        duration_ms: r.get::<_, Option<i64>>(8)?.map(|d| d as u64),
    })
}

/// File names that work on both macOS and Windows.
pub fn validate_stem(stem: &str) -> AppResult<&str> {
    let s = stem.trim();
    let bad = s.is_empty()
        || s.len() > 200
        || s.starts_with('.')
        || s.ends_with('.')
        || s.chars()
            .any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control());
    if bad {
        return Err(AppError::InvalidName(stem.into()));
    }
    Ok(s)
}

fn db(e: rusqlite::Error) -> AppError {
    AppError::Platform(format!("library database: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str, bytes: usize) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, vec![0u8; bytes]).unwrap();
        p
    }

    fn probe(f: &FileEntry) -> Meta {
        match f.kind {
            Kind::Video => Meta { width: Some(1920), height: Some(1080), duration_ms: Some(5000) },
            Kind::Image => Meta { width: Some(800), height: Some(600), duration_ms: None },
        }
    }

    #[test]
    fn scan_picks_media_only() {
        let tmp = tempfile::tempdir().unwrap();
        touch(tmp.path(), "a.mp4", 10);
        touch(tmp.path(), "b.PNG", 10);
        touch(tmp.path(), "notes.txt", 10);
        touch(tmp.path(), ".hidden.png", 10);
        std::fs::create_dir(tmp.path().join("sub.png")).unwrap();
        let mut names: Vec<_> = scan(&[tmp.path().into()]).into_iter().map(|f| file_name(&f.path)).collect();
        names.sort();
        assert_eq!(names, ["a.mp4", "b.PNG"]);
    }

    #[test]
    fn reconcile_adds_updates_and_removes() {
        let tmp = tempfile::tempdir().unwrap();
        let mut lib = Library::in_memory();
        let a = touch(tmp.path(), "a.mp4", 10);
        touch(tmp.path(), "b.png", 20);
        let c = lib.reconcile(&scan(&[tmp.path().into()]), &probe).unwrap();
        assert_eq!(c, Changes { added: 2, updated: 0, removed: 0 });
        // Unchanged: nothing re-probed.
        let calls = std::cell::Cell::new(0);
        let counting = |f: &FileEntry| {
            calls.set(calls.get() + 1);
            probe(f)
        };
        assert!(!lib.reconcile(&scan(&[tmp.path().into()]), &counting).unwrap().any());
        assert_eq!(calls.get(), 0);
        // Changed size → updated; deleted → removed.
        std::fs::write(&a, vec![0u8; 99]).unwrap();
        std::fs::remove_file(tmp.path().join("b.png")).unwrap();
        let c = lib.reconcile(&scan(&[tmp.path().into()]), &probe).unwrap();
        assert_eq!(c, Changes { added: 0, updated: 1, removed: 1 });
        let items = lib.query(&Query::default()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!((items[0].size, items[0].duration_ms), (99, Some(5000)));
    }

    #[test]
    fn query_filters_searches_and_sorts() {
        let tmp = tempfile::tempdir().unwrap();
        let mut lib = Library::in_memory();
        touch(tmp.path(), "Demo call.mp4", 300);
        touch(tmp.path(), "bug report.png", 100);
        touch(tmp.path(), "100%_done.png", 200);
        lib.reconcile(&scan(&[tmp.path().into()]), &probe).unwrap();
        let names = |q: Query| lib.query(&q).unwrap().into_iter().map(|i| i.name).collect::<Vec<_>>();
        assert_eq!(names(Query { filter: Filter::Video, ..Default::default() }), ["Demo call.mp4"]);
        assert_eq!(names(Query { search: "REPORT".into(), ..Default::default() }), ["bug report.png"]);
        // LIKE wildcards in the search text are literal.
        assert_eq!(names(Query { search: "%_".into(), ..Default::default() }), ["100%_done.png"]);
        assert_eq!(
            names(Query { sort: Sort::Size, ..Default::default() }),
            ["Demo call.mp4", "100%_done.png", "bug report.png"]
        );
        assert_eq!(
            names(Query { sort: Sort::Name, limit: Some(1), ..Default::default() }),
            ["100%_done.png"]
        );
    }

    #[test]
    fn stats_by_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let mut lib = Library::in_memory();
        touch(tmp.path(), "a.mp4", 1000);
        touch(tmp.path(), "b.mov", 500);
        touch(tmp.path(), "c.png", 30);
        lib.reconcile(&scan(&[tmp.path().into()]), &probe).unwrap();
        assert_eq!(lib.stats().unwrap(), Stats { videos: 2, video_bytes: 1500, images: 1, image_bytes: 30 });
    }

    #[test]
    fn rename_keeps_extension_and_rejects_bad_names() {
        let tmp = tempfile::tempdir().unwrap();
        let mut lib = Library::in_memory();
        touch(tmp.path(), "old.png", 5);
        touch(tmp.path(), "taken.png", 5);
        lib.reconcile(&scan(&[tmp.path().into()]), &probe).unwrap();
        let id = lib.query(&Query { search: "old".into(), ..Default::default() }).unwrap()[0].id;
        let item = lib.rename(id, "  Onboarding flow ").unwrap();
        assert_eq!(item.name, "Onboarding flow.png");
        assert!(tmp.path().join("Onboarding flow.png").exists());
        assert!(!tmp.path().join("old.png").exists());
        assert_eq!(lib.rename(id, "taken").unwrap_err().code(), "invalid_name");
        for bad in ["", "a/b", "x:y", ".hidden", "trailing.", "what?"] {
            assert!(validate_stem(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn upsert_adds_new_capture() {
        let tmp = tempfile::tempdir().unwrap();
        let mut lib = Library::in_memory();
        let p = touch(tmp.path(), "FrameLy shot.png", 42);
        lib.upsert(&p, Meta { width: Some(10), height: Some(20), duration_ms: None }).unwrap();
        lib.upsert(&p, Meta { width: Some(10), height: Some(20), duration_ms: None }).unwrap();
        let items = lib.query(&Query::default()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!((items[0].width, items[0].height), (Some(10), Some(20)));
    }
}
