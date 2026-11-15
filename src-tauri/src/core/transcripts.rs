//! Transcription store (SQLite, `transcripts.sqlite3`): one transcript per media file, linked by
//! path + size + modification time; segments are appended block by block (so an interrupted job
//! resumes where it stopped); the current summary and topics; and a performance history that
//! feeds the time estimates. Pure (no Tauri) and tested.

use super::{
    errors::{AppError, AppResult},
    transcript::Segment,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Created, waiting in the queue.
    Queued,
    Running,
    /// The app closed or crashed mid-way; resumable.
    Interrupted,
    Cancelled,
    Failed,
    Done,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Queued => "queued",
            Status::Running => "running",
            Status::Interrupted => "interrupted",
            Status::Cancelled => "cancelled",
            Status::Failed => "failed",
            Status::Done => "done",
        }
    }
    fn parse(s: &str) -> Status {
        match s {
            "queued" => Status::Queued,
            "running" => Status::Running,
            "interrupted" => Status::Interrupted,
            "cancelled" => Status::Cancelled,
            "failed" => Status::Failed,
            _ => Status::Done,
        }
    }
}

/// The media a transcript belongs to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRef {
    pub path: String,
    pub name: String,
    /// "video" | "audio"
    pub kind: String,
    pub size: u64,
    /// Unix ms.
    pub mtime: i64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub id: i64,
    pub media: MediaRef,
    /// Requested language ("auto" or an ISO code).
    pub language: String,
    pub detected_language: Option<String>,
    pub precision: String,
    pub model_id: String,
    pub status: Status,
    /// Audio processed so far.
    pub done_ms: u64,
    pub blocks_done: u32,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub has_summary: bool,
    pub has_topics: bool,
    /// Text was edited by hand after recognition.
    pub edited: bool,
}

/// A stored generation (summary or topics). `content` is the JSON the UI renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generated {
    pub detail: String,
    pub provider: String,
    pub model: String,
    pub content: serde_json::Value,
    pub created_at: i64,
    pub edited: bool,
}

pub struct Transcripts {
    conn: Connection,
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS transcripts (
  id INTEGER PRIMARY KEY,
  media_path TEXT NOT NULL UNIQUE,
  media_name TEXT NOT NULL,
  media_kind TEXT NOT NULL,
  media_size INTEGER NOT NULL,
  media_mtime INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL,
  language TEXT NOT NULL,
  detected_language TEXT,
  precision TEXT NOT NULL,
  model_id TEXT NOT NULL,
  status TEXT NOT NULL,
  done_ms INTEGER NOT NULL DEFAULT 0,
  blocks_done INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS segments (
  transcript_id INTEGER NOT NULL REFERENCES transcripts(id) ON DELETE CASCADE,
  idx INTEGER NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  text TEXT NOT NULL,
  words TEXT NOT NULL DEFAULT '[]',
  block INTEGER NOT NULL,
  edited INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (transcript_id, idx)
);
CREATE TABLE IF NOT EXISTS generated (
  transcript_id INTEGER NOT NULL REFERENCES transcripts(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  detail TEXT NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  edited INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (transcript_id, kind)
);
CREATE TABLE IF NOT EXISTS perf (
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL,
  model_id TEXT NOT NULL,
  backend TEXT NOT NULL,
  work REAL NOT NULL,
  elapsed_ms INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS perf_key ON perf(kind, model_id, backend);
";

const SELECT: &str = "SELECT t.id, media_path, media_name, media_kind, media_size, media_mtime, duration_ms, language,
  detected_language, precision, model_id, status, done_ms, blocks_done, error, t.created_at, updated_at,
  EXISTS(SELECT 1 FROM generated g WHERE g.transcript_id = t.id AND g.kind = 'summary'),
  EXISTS(SELECT 1 FROM generated g WHERE g.transcript_id = t.id AND g.kind = 'topics'),
  EXISTS(SELECT 1 FROM segments s WHERE s.transcript_id = t.id AND s.edited = 1)
  FROM transcripts t";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Transcript> {
    Ok(Transcript {
        id: r.get(0)?,
        media: MediaRef {
            path: r.get(1)?,
            name: r.get(2)?,
            kind: r.get(3)?,
            size: r.get::<_, i64>(4)? as u64,
            mtime: r.get(5)?,
            duration_ms: r.get::<_, i64>(6)? as u64,
        },
        language: r.get(7)?,
        detected_language: r.get(8)?,
        precision: r.get(9)?,
        model_id: r.get(10)?,
        status: Status::parse(&r.get::<_, String>(11)?),
        done_ms: r.get::<_, i64>(12)? as u64,
        blocks_done: r.get::<_, i64>(13)? as u32,
        error: r.get(14)?,
        created_at: r.get(15)?,
        updated_at: r.get(16)?,
        has_summary: r.get(17)?,
        has_topics: r.get(18)?,
        edited: r.get(19)?,
    })
}

fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Platform(format!("transcripts database: {e}"))
}

impl Transcripts {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path).map_err(db_err)?;
        Self::init(conn)
    }

    #[cfg(test)]
    pub fn in_memory() -> AppResult<Self> {
        Self::init(Connection::open_in_memory().map_err(db_err)?)
    }

    fn init(conn: Connection) -> AppResult<Self> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").map_err(db_err)?;
        conn.execute_batch(SCHEMA).map_err(db_err)?;
        Ok(Self { conn })
    }

    /// Jobs left "running" by a previous session become resumable.
    pub fn mark_interrupted(&self) -> AppResult<usize> {
        self.conn
            .execute("UPDATE transcripts SET status = 'interrupted' WHERE status IN ('running', 'queued')", [])
            .map_err(db_err)
    }

    pub fn get(&self, id: i64) -> AppResult<Transcript> {
        self.conn
            .query_row(&format!("{SELECT} WHERE t.id = ?1"), [id], row)
            .optional()
            .map_err(db_err)?
            .ok_or(AppError::NotFound)
    }

    pub fn find_by_path(&self, path: &str) -> AppResult<Option<Transcript>> {
        self.conn.query_row(&format!("{SELECT} WHERE media_path = ?1"), [path], row).optional().map_err(db_err)
    }

    pub fn list(&self) -> AppResult<Vec<Transcript>> {
        let mut st = self.conn.prepare(&format!("{SELECT} ORDER BY updated_at DESC")).map_err(db_err)?;
        let rows = st.query_map([], row).map_err(db_err)?;
        rows.collect::<Result<_, _>>().map_err(db_err)
    }

    /// Starts (or restarts) the transcript of `media`. An existing finished transcript of the
    /// same file and settings is returned untouched (no duplicate work); a resumable one keeps
    /// its blocks; anything else starts over.
    pub fn begin(&mut self, media: &MediaRef, language: &str, precision: &str, model_id: &str) -> AppResult<Transcript> {
        let now = now_ms();
        if let Some(t) = self.find_by_path(&media.path)? {
            let same_media = t.media.size == media.size && t.media.mtime == media.mtime;
            let same_settings = t.precision == precision && t.model_id == model_id && t.language == language;
            if same_media && same_settings {
                match t.status {
                    Status::Done => return Ok(t),
                    Status::Interrupted | Status::Cancelled | Status::Failed | Status::Queued | Status::Running => {
                        self.conn
                            .execute("UPDATE transcripts SET status = 'queued', error = NULL, updated_at = ?2 WHERE id = ?1", params![t.id, now])
                            .map_err(db_err)?;
                        return self.get(t.id);
                    }
                }
            }
            // Different file contents or settings: start over, dropping derived content.
            let tx = self.conn.transaction().map_err(db_err)?;
            tx.execute("DELETE FROM segments WHERE transcript_id = ?1", [t.id]).map_err(db_err)?;
            tx.execute("DELETE FROM generated WHERE transcript_id = ?1", [t.id]).map_err(db_err)?;
            tx.execute(
                "UPDATE transcripts SET media_name = ?2, media_kind = ?3, media_size = ?4, media_mtime = ?5, duration_ms = ?6,
                 language = ?7, detected_language = NULL, precision = ?8, model_id = ?9, status = 'queued', done_ms = 0,
                 blocks_done = 0, error = NULL, updated_at = ?10 WHERE id = ?1",
                params![t.id, media.name, media.kind, media.size as i64, media.mtime, media.duration_ms as i64, language, precision, model_id, now],
            )
            .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            return self.get(t.id);
        }
        self.conn
            .execute(
                "INSERT INTO transcripts (media_path, media_name, media_kind, media_size, media_mtime, duration_ms, language,
                 precision, model_id, status, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'queued',?10,?10)",
                params![media.path, media.name, media.kind, media.size as i64, media.mtime, media.duration_ms as i64, language, precision, model_id, now],
            )
            .map_err(db_err)?;
        self.get(self.conn.last_insert_rowid())
    }

    pub fn set_status(&self, id: i64, status: Status, error: Option<&str>) -> AppResult<()> {
        let n = self
            .conn
            .execute(
                "UPDATE transcripts SET status = ?2, error = ?3, updated_at = ?4 WHERE id = ?1",
                params![id, status.as_str(), error, now_ms()],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    /// Persists one finished block atomically (segments + progress), so a crash never leaves a
    /// half-written block behind.
    pub fn append_block(&mut self, id: i64, block: u32, segments: &[Segment], done_ms: u64, detected: Option<&str>) -> AppResult<()> {
        let tx = self.conn.transaction().map_err(db_err)?;
        // Re-running a block replaces it.
        tx.execute("DELETE FROM segments WHERE transcript_id = ?1 AND block >= ?2", params![id, block]).map_err(db_err)?;
        let base: i64 = tx
            .query_row("SELECT COALESCE(MAX(idx) + 1, 0) FROM segments WHERE transcript_id = ?1", [id], |r| r.get(0))
            .map_err(db_err)?;
        {
            let mut ins = tx
                .prepare("INSERT INTO segments (transcript_id, idx, start_ms, end_ms, text, words, block) VALUES (?1,?2,?3,?4,?5,?6,?7)")
                .map_err(db_err)?;
            for (i, s) in segments.iter().enumerate() {
                ins.execute(params![
                    id,
                    base + i as i64,
                    s.start_ms as i64,
                    s.end_ms as i64,
                    s.text,
                    serde_json::to_string(&s.words)?,
                    block
                ])
                .map_err(db_err)?;
            }
        }
        tx.execute(
            "UPDATE transcripts SET blocks_done = ?2, done_ms = ?3, detected_language = COALESCE(?4, detected_language), updated_at = ?5 WHERE id = ?1",
            params![id, block + 1, done_ms as i64, detected, now_ms()],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)
    }

    pub fn segments(&self, id: i64) -> AppResult<Vec<Segment>> {
        let mut st = self
            .conn
            .prepare("SELECT idx, start_ms, end_ms, text, words, block, edited FROM segments WHERE transcript_id = ?1 ORDER BY idx")
            .map_err(db_err)?;
        let rows = st
            .query_map([id], |r| {
                Ok(Segment {
                    idx: r.get::<_, i64>(0)? as u32,
                    start_ms: r.get::<_, i64>(1)? as u64,
                    end_ms: r.get::<_, i64>(2)? as u64,
                    text: r.get(3)?,
                    words: serde_json::from_str(&r.get::<_, String>(4)?).unwrap_or_default(),
                    block: r.get::<_, i64>(5)? as u32,
                    edited: r.get(6)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<_, _>>().map_err(db_err)
    }

    /// Manual correction. Word timing no longer matches edited text, so it is dropped.
    pub fn edit_segment(&self, id: i64, idx: u32, text: &str) -> AppResult<()> {
        let n = self
            .conn
            .execute(
                "UPDATE segments SET text = ?3, words = '[]', edited = 1 WHERE transcript_id = ?1 AND idx = ?2",
                params![id, idx, text.trim()],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(AppError::NotFound);
        }
        self.conn.execute("UPDATE transcripts SET updated_at = ?2 WHERE id = ?1", params![id, now_ms()]).map_err(db_err)?;
        Ok(())
    }

    pub fn set_generated(&self, id: i64, kind: &str, g: &Generated) -> AppResult<()> {
        self.conn
            .execute(
                "INSERT INTO generated (transcript_id, kind, detail, provider, model, content, created_at, edited)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
                 ON CONFLICT(transcript_id, kind) DO UPDATE SET detail=excluded.detail, provider=excluded.provider,
                 model=excluded.model, content=excluded.content, created_at=excluded.created_at, edited=excluded.edited",
                params![id, kind, g.detail, g.provider, g.model, g.content.to_string(), g.created_at, g.edited],
            )
            .map_err(db_err)?;
        self.conn.execute("UPDATE transcripts SET updated_at = ?2 WHERE id = ?1", params![id, now_ms()]).map_err(db_err)?;
        Ok(())
    }

    pub fn generated(&self, id: i64, kind: &str) -> AppResult<Option<Generated>> {
        self.conn
            .query_row(
                "SELECT detail, provider, model, content, created_at, edited FROM generated WHERE transcript_id = ?1 AND kind = ?2",
                params![id, kind],
                |r| {
                    Ok(Generated {
                        detail: r.get(0)?,
                        provider: r.get(1)?,
                        model: r.get(2)?,
                        content: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or(serde_json::Value::Null),
                        created_at: r.get(4)?,
                        edited: r.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(db_err)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.conn.execute("DELETE FROM transcripts WHERE id = ?1", [id]).map_err(db_err)?;
        Ok(())
    }

    pub fn record_perf(&self, kind: &str, model_id: &str, backend: &str, work: f64, elapsed_ms: u64) -> AppResult<()> {
        if work <= 0.0 || elapsed_ms == 0 {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO perf (kind, model_id, backend, work, elapsed_ms, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![kind, model_id, backend, work, elapsed_ms as i64, now_ms()],
            )
            .map_err(db_err)?;
        Ok(())
    }

    /// Median milliseconds per unit of work over the last 10 runs, and how many runs it's based on.
    pub fn perf_rate(&self, kind: &str, model_id: &str, backend: &str) -> AppResult<Option<(f64, usize)>> {
        let mut st = self
            .conn
            .prepare("SELECT elapsed_ms / work FROM perf WHERE kind = ?1 AND model_id = ?2 AND backend = ?3 ORDER BY id DESC LIMIT 10")
            .map_err(db_err)?;
        let mut v: Vec<f64> = st
            .query_map(params![kind, model_id, backend], |r| r.get(0))
            .map_err(db_err)?
            .collect::<Result<_, _>>()
            .map_err(db_err)?;
        if v.is_empty() {
            return Ok(None);
        }
        v.sort_by(|a, b| a.total_cmp(b));
        Ok(Some((v[v.len() / 2], v.len())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transcript::Word;

    fn media(path: &str, size: u64) -> MediaRef {
        MediaRef { path: path.into(), name: "a.mp4".into(), kind: "video".into(), size, mtime: 1, duration_ms: 600_000 }
    }

    fn seg(a: u64, t: &str) -> Segment {
        Segment {
            idx: 0,
            start_ms: a,
            end_ms: a + 900,
            text: t.into(),
            words: vec![Word { text: t.into(), start_ms: a, end_ms: a + 900, p: 0.9 }],
            block: 0,
            edited: false,
        }
    }

    #[test]
    fn blocks_append_and_resume() {
        let mut db = Transcripts::in_memory().unwrap();
        let t = db.begin(&media("/m/a.mp4", 10), "auto", "balanced", "whisper-turbo").unwrap();
        assert_eq!(t.status, Status::Queued);
        db.set_status(t.id, Status::Running, None).unwrap();
        db.append_block(t.id, 0, &[seg(0, "olá"), seg(1000, "mundo")], 300_000, Some("pt")).unwrap();
        // App closes mid-job.
        assert_eq!(db.mark_interrupted().unwrap(), 1);
        let again = db.begin(&media("/m/a.mp4", 10), "auto", "balanced", "whisper-turbo").unwrap();
        assert_eq!((again.id, again.blocks_done, again.done_ms), (t.id, 1, 300_000));
        assert_eq!(again.detected_language.as_deref(), Some("pt"));
        db.append_block(t.id, 1, &[seg(300_000, "fim")], 600_000, None).unwrap();
        let s = db.segments(t.id).unwrap();
        assert_eq!(s.iter().map(|s| s.idx).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(s[0].words.len(), 1);
        // Re-running block 1 replaces it instead of duplicating.
        db.append_block(t.id, 1, &[seg(300_000, "fim!")], 600_000, None).unwrap();
        assert_eq!(db.segments(t.id).unwrap().len(), 3);
    }

    #[test]
    fn finished_transcripts_are_reused_and_changes_restart() {
        let mut db = Transcripts::in_memory().unwrap();
        let t = db.begin(&media("/m/a.mp4", 10), "auto", "balanced", "m").unwrap();
        db.append_block(t.id, 0, &[seg(0, "x")], 600_000, None).unwrap();
        db.set_status(t.id, Status::Done, None).unwrap();
        db.set_generated(t.id, "summary", &Generated { detail: "balanced".into(), provider: "framely".into(), model: "q".into(), content: serde_json::json!({"overview": "x"}), created_at: 1, edited: false }).unwrap();
        // Same file and settings: cached result.
        let same = db.begin(&media("/m/a.mp4", 10), "auto", "balanced", "m").unwrap();
        assert_eq!(same.status, Status::Done);
        assert!(same.has_summary);
        // Different precision: starts over and drops derived content.
        let redo = db.begin(&media("/m/a.mp4", 10), "auto", "max", "m2").unwrap();
        assert_eq!((redo.id, redo.status, redo.blocks_done), (t.id, Status::Queued, 0));
        assert!(db.segments(t.id).unwrap().is_empty());
        assert!(!redo.has_summary);
        // The file changed on disk: also starts over.
        db.append_block(t.id, 0, &[seg(0, "y")], 600_000, None).unwrap();
        db.set_status(t.id, Status::Done, None).unwrap();
        let changed = db.begin(&media("/m/a.mp4", 11), "auto", "max", "m2").unwrap();
        assert_eq!(changed.status, Status::Queued);
    }

    #[test]
    fn edits_and_generations() {
        let mut db = Transcripts::in_memory().unwrap();
        let t = db.begin(&media("/m/b.m4a", 5), "pt", "fast", "m").unwrap();
        db.append_block(t.id, 0, &[seg(0, "ola")], 1000, None).unwrap();
        db.edit_segment(t.id, 0, " Olá. ").unwrap();
        let s = &db.segments(t.id).unwrap()[0];
        assert_eq!((s.text.as_str(), s.edited, s.words.len()), ("Olá.", true, 0));
        assert!(db.get(t.id).unwrap().edited);
        assert_eq!(db.edit_segment(t.id, 9, "x").unwrap_err().code(), "not_found");
        assert!(db.generated(t.id, "topics").unwrap().is_none());
        let g = Generated { detail: "detailed".into(), provider: "apple".into(), model: "system".into(), content: serde_json::json!({"topics": []}), created_at: 2, edited: false };
        db.set_generated(t.id, "topics", &g).unwrap();
        assert_eq!(db.generated(t.id, "topics").unwrap().unwrap(), g);
        db.delete(t.id).unwrap();
        assert!(db.list().unwrap().is_empty());
        assert!(db.segments(t.id).unwrap().is_empty(), "segments cascade");
    }

    #[test]
    fn perf_rate_is_the_recent_median() {
        let db = Transcripts::in_memory().unwrap();
        assert!(db.perf_rate("asr", "m", "metal").unwrap().is_none());
        for (work, ms) in [(100.0, 6_000), (100.0, 5_000), (200.0, 60_000)] {
            db.record_perf("asr", "m", "metal", work, ms).unwrap();
        }
        db.record_perf("asr", "m", "cpu", 100.0, 99_000).unwrap();
        let (rate, n) = db.perf_rate("asr", "m", "metal").unwrap().unwrap();
        assert_eq!(n, 3);
        assert!((rate - 60.0).abs() < 1e-9);
    }
}
