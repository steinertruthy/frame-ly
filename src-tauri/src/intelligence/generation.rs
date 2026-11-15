//! Summaries and topics: prompts (shared by every provider), output schemas, splitting long
//! transcripts to fit a provider's context (map-reduce), and turning model output into the
//! stored JSON (timestamps parsed, clamped and snapped to the transcript). Pure and tested;
//! running a model is the caller's job (`Run`).

use crate::core::{
    errors::{AppError, AppResult},
    settings::Detail,
    transcript::{self, Segment},
};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Summary,
    Topics,
}

impl Task {
    pub fn as_str(self) -> &'static str {
        match self {
            Task::Summary => "summary",
            Task::Topics => "topics",
        }
    }
}

pub fn detail_name(d: Detail) -> &'static str {
    match d {
        Detail::Brief => "brief",
        Detail::Balanced => "balanced",
        Detail::Detailed => "detailed",
    }
}

fn language_name(code: Option<&str>) -> &'static str {
    match code.unwrap_or("") {
        "pt" => "Portuguese (Brazil)",
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "nl" => "Dutch",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "ru" => "Russian",
        "pl" => "Polish",
        "tr" => "Turkish",
        "ar" => "Arabic",
        "hi" => "Hindi",
        _ => "the same language as the transcript",
    }
}

pub fn system_prompt(language: Option<&str>) -> String {
    format!(
        "You turn transcripts of recordings into faithful notes.\n\
         Rules:\n\
         - Use only information stated in the transcript. Never invent names, numbers, dates, decisions, owners or conclusions, and never change the meaning.\n\
         - Do not repeat the same information.\n\
         - Speakers are not labeled. Attribute something to a person only when the transcript makes it explicit (for example, someone is addressed by name or introduces themselves).\n\
         - Each transcript line starts with its time as [mm:ss] or [h:mm:ss]. For every item, give the time of the passage it comes from, copied from those labels.\n\
         - If something is unclear or inaudible, leave it out.\n\
         - Write in {}. Answer with JSON only.",
        language_name(language)
    )
}

fn summary_instructions(d: Detail) -> &'static str {
    match d {
        Detail::Brief => "Write a brief summary: an overview of one or two sentences and 3 to 5 key points.",
        Detail::Balanced => "Write a balanced summary: an overview paragraph of three or four sentences and 5 to 8 key points.",
        Detail::Detailed => {
            "Write a detailed summary: an overview paragraph of up to six sentences and 8 to 15 key points that cover every subject discussed, in order."
        }
    }
}

fn topics_instructions(d: Detail) -> &'static str {
    match d {
        Detail::Brief => "List only the main topics, in the order they come up, each with a few short points; add subtopics only when essential.",
        Detail::Balanced => "Organize the content into topics and subtopics in a logical order, each with short points; use a third level only when it really helps.",
        Detail::Detailed => "Organize all the content into a complete hierarchy of topics, subtopics and details (up to three levels), each with short points.",
    }
}

const TOPICS_EXTRAS: &str = "Also list: decisions that were made; actions or tasks (owner and due date only when explicitly said, otherwise empty strings); dates and deadlines mentioned; relevant numbers with what they refer to; and up to 12 keywords or names that appear verbatim in the transcript.";

pub fn user_prompt(task: Task, detail: Detail, transcript: &str, part: Option<(usize, usize)>) -> String {
    let what = match task {
        Task::Summary => summary_instructions(detail).to_string(),
        Task::Topics => format!("{} {}", topics_instructions(detail), TOPICS_EXTRAS),
    };
    let scope = match part {
        Some((i, n)) => format!("This is part {} of {} of a longer transcript; cover only this part.\n", i + 1, n),
        None => String::new(),
    };
    format!("{what}\n{scope}\nTRANSCRIPT:\n{transcript}")
}

/// Combining partial summaries of consecutive parts into the final one.
pub fn reduce_prompt(detail: Detail, partials: &[Value]) -> String {
    let mut text = String::new();
    for (i, p) in partials.iter().enumerate() {
        text.push_str(&format!("PART {}:\n{}\n", i + 1, p["overview"].as_str().unwrap_or("")));
        for pt in p["points"].as_array().into_iter().flatten() {
            let ms = pt["ms"].as_u64().map(transcript::fmt_clock).unwrap_or_default();
            text.push_str(&format!("[{ms}] {}\n", pt["text"].as_str().unwrap_or("")));
        }
        text.push('\n');
    }
    format!(
        "The following are summaries of consecutive parts of one recording, in order. Combine them into a single summary of the whole recording, without adding anything new. Keep the [mm:ss] times of the points you keep.\n{}\n\n{text}",
        summary_instructions(detail)
    )
}

fn timed(extra: &[(&str, Value)]) -> Value {
    let mut props = serde_json::Map::new();
    props.insert("text".into(), json!({"type": "string"}));
    for (k, v) in extra {
        props.insert((*k).into(), v.clone());
    }
    props.insert("time".into(), json!({"type": "string"}));
    let required: Vec<&String> = props.keys().collect();
    json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
}

/// JSON Schema of the model output (strict: no extra fields, everything required; three
/// explicit levels because recursive schemas aren't accepted everywhere).
pub fn schema(task: Task) -> Value {
    match task {
        Task::Summary => json!({
            "type": "object",
            "properties": {"overview": {"type": "string"}, "points": {"type": "array", "items": timed(&[])}},
            "required": ["overview", "points"],
            "additionalProperties": false
        }),
        Task::Topics => {
            let node = |children: Option<Value>| {
                let mut props = json!({"title": {"type": "string"}, "time": {"type": "string"}, "points": {"type": "array", "items": {"type": "string"}}});
                let mut required = vec!["title", "time", "points"];
                if let Some(c) = children {
                    props["subtopics"] = json!({"type": "array", "items": c});
                    required.push("subtopics");
                }
                json!({"type": "object", "properties": props, "required": required, "additionalProperties": false})
            };
            let leaf = node(None);
            let mid = node(Some(leaf));
            let top = node(Some(mid));
            json!({
                "type": "object",
                "properties": {
                    "topics": {"type": "array", "items": top},
                    "decisions": {"type": "array", "items": timed(&[])},
                    "actions": {"type": "array", "items": timed(&[("owner", json!({"type": "string"})), ("due", json!({"type": "string"}))])},
                    "dates": {"type": "array", "items": timed(&[])},
                    "numbers": {"type": "array", "items": timed(&[])},
                    "keywords": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["topics", "decisions", "actions", "dates", "numbers", "keywords"],
                "additionalProperties": false
            })
        }
    }
}

pub fn max_tokens(task: Task, detail: Detail) -> u32 {
    match (task, detail) {
        (Task::Summary, Detail::Brief) => 700,
        (Task::Summary, Detail::Balanced) => 1_400,
        (Task::Summary, Detail::Detailed) => 2_800,
        (Task::Topics, Detail::Brief) => 1_800,
        (Task::Topics, Detail::Balanced) => 3_500,
        (Task::Topics, Detail::Detailed) => 5_500,
    }
}

/// Conservative token estimate for planning (real counts come from the model when available).
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() / 3 + 16
}

/// Splits transcript lines into parts that fit `budget` tokens each (never splits a line).
pub fn split_lines(lines: &[(u64, String)], budget: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut used = 0;
    for (_, l) in lines {
        let t = estimate_tokens(l);
        if used + t > budget && !cur.is_empty() {
            parts.push(std::mem::take(&mut cur));
            used = 0;
        }
        cur.push_str(l);
        cur.push('\n');
        used += t;
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    parts
}

/// Extracts the JSON object from model output (tolerates code fences or stray text around it).
pub fn parse_json(text: &str) -> AppResult<Value> {
    let start = text.find('{').ok_or_else(|| AppError::Provider("no JSON in response".into()))?;
    let end = text.rfind('}').ok_or_else(|| AppError::Provider("no JSON in response".into()))?;
    serde_json::from_str(&text[start..=end]).map_err(|e| AppError::Provider(format!("invalid JSON: {e}")))
}

/// Resolves a model-given time to a transcript position: parsed, within the recording, and
/// snapped to the start of the segment that contains it.
fn locate(t: &Value, segments: &[Segment], duration_ms: u64) -> Value {
    let Some(ms) = t.as_str().and_then(transcript::parse_clock) else { return Value::Null };
    if ms > duration_ms + 1_000 {
        return Value::Null;
    }
    match transcript::segment_at(segments, ms) {
        Some(i) => json!(segments[i].start_ms),
        None => json!(ms),
    }
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").trim().to_string()
}

/// Model output → stored summary `{overview, points: [{text, ms}]}`.
pub fn normalize_summary(raw: &Value, segments: &[Segment], duration_ms: u64) -> Value {
    let points: Vec<Value> = raw["points"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| json!({"text": s(&p["text"]), "ms": locate(&p["time"], segments, duration_ms)}))
        .filter(|p| !p["text"].as_str().unwrap_or("").is_empty())
        .collect();
    json!({"overview": s(&raw["overview"]), "points": points})
}

fn normalize_node(n: &Value, segments: &[Segment], duration_ms: u64, depth: usize) -> Value {
    let children: Vec<Value> = if depth < 2 {
        n["subtopics"].as_array().into_iter().flatten().map(|c| normalize_node(c, segments, duration_ms, depth + 1)).collect()
    } else {
        Vec::new()
    };
    let points: Vec<String> = n["points"].as_array().into_iter().flatten().map(s).filter(|p| !p.is_empty()).collect();
    json!({"title": s(&n["title"]), "ms": locate(&n["time"], segments, duration_ms), "points": points, "children": children})
}

fn timed_list(v: &Value, segments: &[Segment], duration_ms: u64, extra: &[&str]) -> Vec<Value> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|i| !s(&i["text"]).is_empty())
        .map(|i| {
            let mut o = json!({"text": s(&i["text"]), "ms": locate(&i["time"], segments, duration_ms)});
            for k in extra {
                o[*k] = json!(s(&i[*k]));
            }
            o
        })
        .collect()
}

/// Model output → stored topics.
pub fn normalize_topics(raw: &Value, segments: &[Segment], duration_ms: u64, full_text: &str) -> Value {
    let lower = full_text.to_lowercase();
    // Keywords must really appear in the transcript (used to highlight it).
    let mut keywords: Vec<String> = Vec::new();
    for k in raw["keywords"].as_array().into_iter().flatten().map(s) {
        if !k.is_empty() && lower.contains(&k.to_lowercase()) && !keywords.iter().any(|x| x.eq_ignore_ascii_case(&k)) {
            keywords.push(k);
        }
    }
    keywords.truncate(12);
    json!({
        "topics": raw["topics"].as_array().into_iter().flatten().map(|t| normalize_node(t, segments, duration_ms, 0)).filter(|t| !t["title"].as_str().unwrap_or("").is_empty()).collect::<Vec<_>>(),
        "decisions": timed_list(&raw["decisions"], segments, duration_ms, &[]),
        "actions": timed_list(&raw["actions"], segments, duration_ms, &["owner", "due"]),
        "dates": timed_list(&raw["dates"], segments, duration_ms, &[]),
        "numbers": timed_list(&raw["numbers"], segments, duration_ms, &[]),
        "keywords": keywords,
    })
}

/// Joins normalized topics of consecutive parts (in order; nothing is rewritten).
pub fn merge_topics(parts: &[Value]) -> Value {
    let mut out = json!({"topics": [], "decisions": [], "actions": [], "dates": [], "numbers": [], "keywords": []});
    for p in parts {
        for k in ["topics", "decisions", "actions", "dates", "numbers"] {
            let mut items = p[k].as_array().cloned().unwrap_or_default();
            out[k].as_array_mut().unwrap().append(&mut items);
        }
        for kw in p["keywords"].as_array().into_iter().flatten() {
            let list = out["keywords"].as_array_mut().unwrap();
            if !list.iter().any(|x| x.as_str().map(str::to_lowercase) == kw.as_str().map(str::to_lowercase)) && list.len() < 12 {
                list.push(kw.clone());
            }
        }
    }
    out
}

/// Plain text of a stored summary or topics, for copying and TXT/Markdown/PDF exports.
pub fn summary_markdown(v: &Value) -> String {
    let mut out = format!("{}\n\n", v["overview"].as_str().unwrap_or(""));
    for p in v["points"].as_array().into_iter().flatten() {
        let time = p["ms"].as_u64().map(|m| format!("[{}] ", transcript::fmt_clock(m))).unwrap_or_default();
        out.push_str(&format!("- {time}{}\n", p["text"].as_str().unwrap_or("")));
    }
    out
}

pub fn topics_markdown(v: &Value, labels: &Labels) -> String {
    fn node(out: &mut String, n: &Value, depth: usize) {
        let time = n["ms"].as_u64().map(|m| format!(" [{}]", transcript::fmt_clock(m))).unwrap_or_default();
        if depth == 0 {
            out.push_str(&format!("\n## {}{time}\n", n["title"].as_str().unwrap_or("")));
        } else {
            out.push_str(&format!("{}- **{}**{time}\n", "  ".repeat(depth - 1), n["title"].as_str().unwrap_or("")));
        }
        for p in n["points"].as_array().into_iter().flatten() {
            out.push_str(&format!("{}- {}\n", "  ".repeat(depth), p.as_str().unwrap_or("")));
        }
        for c in n["children"].as_array().into_iter().flatten() {
            node(out, c, depth + 1);
        }
    }
    let mut out = String::new();
    for t in v["topics"].as_array().into_iter().flatten() {
        node(&mut out, t, 0);
    }
    for (key, title) in [("decisions", &labels.decisions), ("actions", &labels.actions), ("dates", &labels.dates), ("numbers", &labels.numbers)] {
        let items = v[key].as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("\n## {title}\n"));
        for i in items {
            let time = i["ms"].as_u64().map(|m| format!("[{}] ", transcript::fmt_clock(m))).unwrap_or_default();
            let mut line = format!("- {time}{}", i["text"].as_str().unwrap_or(""));
            for (k, l) in [("owner", &labels.owner), ("due", &labels.due)] {
                if let Some(x) = i[k].as_str().filter(|x| !x.is_empty()) {
                    line.push_str(&format!(" — {l}: {x}"));
                }
            }
            out.push_str(&line);
            out.push('\n');
        }
    }
    out.trim_start().to_string()
}

/// Localized section names for exports (the UI sends them).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Labels {
    pub decisions: String,
    pub actions: String,
    pub dates: String,
    pub numbers: String,
    pub owner: String,
    pub due: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(a: u64, t: &str) -> Segment {
        Segment { idx: 0, start_ms: a, end_ms: a + 4_000, text: t.into(), words: vec![], block: 0, edited: false }
    }

    #[test]
    fn schemas_are_strict() {
        fn check(v: &Value) {
            if v["type"] == "object" {
                assert_eq!(v["additionalProperties"], false);
                let props: Vec<&String> = v["properties"].as_object().unwrap().keys().collect();
                let req: Vec<&str> = v["required"].as_array().unwrap().iter().map(|r| r.as_str().unwrap()).collect();
                assert_eq!(props.len(), req.len(), "all required");
                for p in v["properties"].as_object().unwrap().values() {
                    check(p);
                }
            }
            if v["type"] == "array" {
                check(&v["items"]);
            }
        }
        check(&schema(Task::Summary));
        check(&schema(Task::Topics));
        // Three levels, the last without children.
        let lvl3 = &schema(Task::Topics)["properties"]["topics"]["items"]["properties"]["subtopics"]["items"]["properties"]["subtopics"]["items"];
        assert!(lvl3["properties"].get("subtopics").is_none());
    }

    #[test]
    fn prompts_carry_rules_detail_and_language() {
        let sys = system_prompt(Some("pt"));
        assert!(sys.contains("Never invent") && sys.contains("Portuguese (Brazil)"));
        assert!(system_prompt(None).contains("same language as the transcript"));
        let u = user_prompt(Task::Summary, Detail::Brief, "[00:01] oi", Some((1, 3)));
        assert!(u.contains("3 to 5 key points") && u.contains("part 2 of 3") && u.ends_with("[00:01] oi"));
        assert!(user_prompt(Task::Topics, Detail::Detailed, "x", None).contains("decisions"));
    }

    #[test]
    fn splits_long_transcripts_by_line() {
        let lines: Vec<(u64, String)> = (0..100).map(|i| (i, format!("[00:{i:02}] {}", "palavra ".repeat(30)))).collect();
        let parts = split_lines(&lines, 1_000);
        assert!(parts.len() > 3);
        assert!(parts.iter().all(|p| estimate_tokens(p) <= 1_100));
        assert_eq!(parts.concat().lines().count(), 100, "nothing lost");
        assert_eq!(split_lines(&lines[..2], 100_000).len(), 1);
    }

    #[test]
    fn parses_json_from_noisy_output() {
        assert_eq!(parse_json("```json\n{\"a\": 1}\n```").unwrap()["a"], 1);
        assert_eq!(parse_json("nada").unwrap_err().code(), "provider");
    }

    #[test]
    fn normalizes_times_and_drops_inventions() {
        let segs = vec![seg(0, "Bom dia"), seg(40_000, "Lançamento no dia 15"), seg(72_000, "Plano anual")];
        let raw = json!({"overview": " Visão ", "points": [
            {"text": "Data", "time": "00:46"}, {"text": "Fora", "time": "59:00"}, {"text": "", "time": "00:01"}, {"text": "Sem hora", "time": "agora"}
        ]});
        let n = normalize_summary(&raw, &segs, 80_000);
        assert_eq!(n["overview"], "Visão");
        assert_eq!(n["points"].as_array().unwrap().len(), 3, "empty point dropped");
        assert_eq!(n["points"][0]["ms"], 40_000, "snapped to the segment start");
        assert!(n["points"][1]["ms"].is_null(), "beyond the recording");
        assert!(n["points"][2]["ms"].is_null());

        let topics = json!({"topics": [{"title": "Lançamento", "time": "[00:40]", "points": ["15/12"], "subtopics": [
            {"title": "Riscos", "time": "00:41", "points": [], "subtopics": [{"title": "Erros", "time": "00:42", "points": ["47"]}]}]}],
            "decisions": [{"text": "Lançar dia 15", "time": "01:12"}],
            "actions": [{"text": "Atualizar cronograma", "owner": "", "due": "hoje", "time": "01:21"}],
            "dates": [], "numbers": [], "keywords": ["Lançamento", "unicórnio", "lançamento"]});
        let t = normalize_topics(&topics, &segs, 80_000, "Bom dia Lançamento no dia 15 Plano anual");
        assert_eq!(t["topics"][0]["children"][0]["children"][0]["title"], "Erros");
        assert_eq!(t["actions"][0]["due"], "hoje");
        assert_eq!(t["keywords"], json!(["Lançamento"]), "keywords must exist in the transcript, once");
        let merged = merge_topics(&[t.clone(), t]);
        assert_eq!(merged["topics"].as_array().unwrap().len(), 2);
        assert_eq!(merged["keywords"].as_array().unwrap().len(), 1);
        let labels = Labels { decisions: "Decisões".into(), actions: "Tarefas".into(), dates: "Datas".into(), numbers: "Números".into(), owner: "Responsável".into(), due: "Prazo".into() };
        let md = topics_markdown(&merged, &labels);
        assert!(md.starts_with("## Lançamento [00:40]") && md.contains("## Decisões") && md.contains("Prazo: hoje"));
        assert!(summary_markdown(&n).contains("- [00:40] Data"));
    }

    #[test]
    fn reduce_prompt_keeps_times() {
        let p = reduce_prompt(Detail::Balanced, &[json!({"overview": "A", "points": [{"text": "x", "ms": 65_000}]})]);
        assert!(p.contains("PART 1:\nA\n[01:05] x"));
    }
}
