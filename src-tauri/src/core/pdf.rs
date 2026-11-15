//! Small PDF writer for exports (transcript, summary, topics): A4 pages, the standard Helvetica
//! fonts (no embedding) with WinAnsi encoding, word wrapping, headings, timestamped paragraphs
//! and bullet lists. WinAnsi covers the Latin scripts of the interface languages (pt, en, es, fr,
//! de…); text in other scripts can't be shown with the standard fonts, so `supports` lets the
//! caller refuse instead of writing `?`. Pure and tested.

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Title(String),
    /// Small grey line under the title (date, duration, source).
    Meta(String),
    Heading(String),
    /// Paragraph with an optional time label in the margin.
    Para { time: Option<String>, text: String },
    Bullet { level: u8, time: Option<String>, text: String },
}

const PAGE_W: f32 = 595.0;
const PAGE_H: f32 = 842.0;
const MARGIN: f32 = 56.0;
const TIME_COL: f32 = 46.0;

/// Helvetica advance widths (1/1000 em) for ASCII 32..=126.
const HELV: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833,
    722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556,
    556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334,
    260, 334, 584,
];

/// WinAnsi byte for a character, if the standard fonts can show it.
fn win_ansi(c: char) -> Option<u8> {
    let u = c as u32;
    match c {
        '\u{20}'..='\u{7e}' => Some(u as u8),
        '\u{a0}'..='\u{ff}' => Some(u as u8),
        '€' => Some(0x80),
        '…' => Some(0x85),
        '‘' => Some(0x91),
        '’' => Some(0x92),
        '“' => Some(0x93),
        '”' => Some(0x94),
        '•' => Some(0x95),
        '–' => Some(0x96),
        '—' => Some(0x97),
        '\t' | '\n' | '\r' => Some(b' '),
        _ => None,
    }
}

/// True when every character can be rendered with the standard fonts.
pub fn supports(text: &str) -> bool {
    text.chars().all(|c| win_ansi(c).is_some() || c.is_whitespace())
}

fn base_letter(c: char) -> char {
    // Accented Latin letters take the width of their base letter.
    match c {
        'à'..='å' => 'a',
        'À'..='Å' => 'A',
        'ç' => 'c',
        'Ç' => 'C',
        'è'..='ë' => 'e',
        'È'..='Ë' => 'E',
        'ì'..='ï' => 'i',
        'Ì'..='Ï' => 'I',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ò'..='ö' | 'ø' => 'o',
        'Ò'..='Ö' | 'Ø' => 'O',
        'ù'..='ü' => 'u',
        'Ù'..='Ü' => 'U',
        'ý' | 'ÿ' => 'y',
        'Ý' => 'Y',
        'ß' => 'B',
        '–' => '-',
        '—' => 'M',
        '“' | '”' => '"',
        '‘' | '’' => '\'',
        '…' => 'm',
        _ => c,
    }
}

fn width(text: &str, size: f32, bold: bool) -> f32 {
    let em: u32 = text
        .chars()
        .map(|c| {
            let b = base_letter(c);
            if (' '..='~').contains(&b) {
                HELV[b as usize - 32] as u32
            } else {
                556
            }
        })
        .sum();
    em as f32 * size / 1000.0 * if bold { 1.06 } else { 1.0 }
}

fn wrap(text: &str, max: f32, size: f32, bold: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let candidate = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if width(&candidate, size, bold) <= max || cur.is_empty() {
            cur = candidate;
        } else {
            lines.push(std::mem::replace(&mut cur, word.to_string()));
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn pdf_string(text: &str) -> Vec<u8> {
    let mut out = vec![b'('];
    for c in text.chars() {
        let b = win_ansi(c).unwrap_or(b'?');
        if matches!(b, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(b);
    }
    out.push(b')');
    out
}

struct Layout {
    pages: Vec<Vec<u8>>,
    cur: Vec<u8>,
    y: f32,
}

impl Layout {
    fn ensure(&mut self, h: f32) {
        if self.y - h < MARGIN {
            self.pages.push(std::mem::take(&mut self.cur));
            self.y = PAGE_H - MARGIN;
        }
    }
    fn text(&mut self, x: f32, font: &str, size: f32, grey: f32, s: &str) {
        self.cur.extend_from_slice(format!("BT /{font} {size} Tf {grey} g {x:.1} {:.1} Td ", self.y).as_bytes());
        self.cur.extend_from_slice(&pdf_string(s));
        self.cur.extend_from_slice(b" Tj ET\n");
    }
    #[allow(clippy::too_many_arguments)]
    fn lines(&mut self, x: f32, max: f32, font: &str, size: f32, grey: f32, text: &str, label: Option<&str>, marker: Option<&str>) {
        let lead = size * 1.42;
        for (i, l) in wrap(text, max, size, font == "F2").into_iter().enumerate() {
            self.ensure(lead);
            self.y -= lead;
            if i == 0 {
                if let Some(t) = label {
                    self.text(MARGIN, "F1", size - 1.5, 0.5, t);
                }
                if let Some(m) = marker {
                    self.text(x - 10.0, "F1", size, 0.4, m);
                }
            }
            self.text(x, font, size, grey, &l);
        }
    }
}

/// Renders `blocks` to a complete PDF file.
pub fn render(blocks: &[Block], doc_title: &str) -> Vec<u8> {
    let mut l = Layout { pages: Vec::new(), cur: Vec::new(), y: PAGE_H - MARGIN };
    let full = PAGE_W - 2.0 * MARGIN;
    for b in blocks {
        match b {
            Block::Title(t) => l.lines(MARGIN, full, "F2", 20.0, 0.05, t, None, None),
            Block::Meta(t) => {
                l.lines(MARGIN, full, "F1", 9.5, 0.45, t, None, None);
                l.y -= 8.0;
            }
            Block::Heading(t) => {
                l.ensure(40.0);
                l.y -= 10.0;
                l.lines(MARGIN, full, "F2", 13.0, 0.1, t, None, None);
                l.y -= 2.0;
            }
            Block::Para { time, text } => {
                let (x, w) = if time.is_some() { (MARGIN + TIME_COL, full - TIME_COL) } else { (MARGIN, full) };
                l.lines(x, w, "F1", 10.5, 0.12, text, time.as_deref(), None);
                l.y -= 6.0;
            }
            Block::Bullet { level, time, text } => {
                let indent = 12.0 + *level as f32 * 16.0;
                let base = if time.is_some() { MARGIN + TIME_COL } else { MARGIN };
                let x = base + indent;
                let marker = if *level == 0 { "•" } else { "–" };
                l.lines(x, full - (x - MARGIN), "F1", 10.5, 0.12, text, time.as_deref(), Some(marker));
                l.y -= 2.0;
            }
        }
    }
    l.pages.push(std::mem::take(&mut l.cur));

    // Objects: 1 catalog, 2 pages, 3 F1, 4 F2, 5 info, then (page, content) pairs.
    let mut objs: Vec<Vec<u8>> = Vec::new();
    let n = l.pages.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 6 + i * 2)).collect();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")).into_bytes());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
    let mut info = b"<< /Producer (FrameLy) /Title ".to_vec();
    info.extend_from_slice(&pdf_string(doc_title));
    info.extend_from_slice(b" >>");
    objs.push(info);
    for (i, content) in l.pages.iter().enumerate() {
        let content_id = 7 + i * 2;
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        );
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend_from_slice(content);
        stream.extend_from_slice(b"\nendstream");
        objs.push(stream);
    }

    let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = Vec::with_capacity(objs.len());
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_latin_but_not_other_scripts() {
        assert!(supports("Reunião às 10h — decisão: “sim” (€ 290)…"));
        assert!(!supports("会議の要約"));
    }

    #[test]
    fn wraps_to_width() {
        let lines = wrap(&"palavra ".repeat(60), 300.0, 10.0, false);
        assert!(lines.len() > 3);
        assert!(lines.iter().all(|l| width(l, 10.0, false) <= 300.0));
    }

    #[test]
    fn renders_a_valid_multi_page_document() {
        let mut blocks = vec![Block::Title("Reunião (planejamento)".into()), Block::Meta("12/11 · 03:37".into())];
        for i in 0..120 {
            blocks.push(Block::Para { time: Some(format!("{i:02}:00")), text: "Texto com acentuação: ação, café, Ñ. ".repeat(4) });
            blocks.push(Block::Bullet { level: (i % 2) as u8, time: None, text: "Item".into() });
        }
        let pdf = render(&blocks, "Teste");
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.starts_with("%PDF-1.4"));
        assert!(s.trim_end().ends_with("%%EOF"));
        let count: usize = s.split("/Count ").nth(1).unwrap().split(' ').next().unwrap().trim_end_matches('>').trim().parse().unwrap();
        assert!(count > 1, "multiple pages");
        // Parentheses are escaped and accents are single WinAnsi bytes.
        assert!(s.contains("\\(planejamento\\)"));
        assert!(pdf.windows(4).any(|w| w == [b'a', 0xE7, 0xE3, b'o']), "ação in WinAnsi");
        // xref offsets point at objects (byte offsets, so search the raw bytes).
        let tail = &pdf[pdf.len() - 40..];
        let tail = std::str::from_utf8(tail).unwrap();
        let xref_at: usize = tail.rsplit("startxref\n").next().unwrap().lines().next().unwrap().parse().unwrap();
        assert!(pdf[xref_at..].starts_with(b"xref"));
        let first = std::str::from_utf8(&pdf[xref_at..xref_at + 80]).unwrap().lines().nth(3).unwrap().to_string();
        let off: usize = first[..10].parse().unwrap();
        assert!(pdf[off..].starts_with(b"1 0 obj"));
    }
}
