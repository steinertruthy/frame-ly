//! Global shortcut model: actions, defaults, accelerator parsing/normalization and the
//! checks that keep FrameLy from stealing system or typing shortcuts. Pure and tested.
//!
//! Accelerators use Tauri's syntax, e.g. `CmdOrCtrl+Alt+Shift+4`. `CmdOrCtrl` is ⌘ on macOS
//! and Ctrl on Windows; `Super` is ⌘ / the Windows key.

use super::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    OpenPanel,
    CaptureRegion,
    CaptureWindow,
    CaptureScreen,
    RecordRegion,
    RecordWindow,
    RecordScreen,
    StopRecording,
    TogglePause,
}

#[cfg(test)]
pub const ALL_ACTIONS: [Action; 9] = [
    Action::OpenPanel,
    Action::CaptureRegion,
    Action::CaptureWindow,
    Action::CaptureScreen,
    Action::RecordRegion,
    Action::RecordWindow,
    Action::RecordScreen,
    Action::StopRecording,
    Action::TogglePause,
];

/// Action → accelerator. An empty string disables the shortcut (so a default can be turned off).
pub type Shortcuts = BTreeMap<Action, String>;

/// ⌘⌥⇧ / Ctrl+Alt+Shift chords: free on both systems, and never produce typed characters
/// (Option+Shift+digit does on macOS keyboards).
pub fn defaults() -> Shortcuts {
    [
        (Action::OpenPanel, "CmdOrCtrl+Alt+Shift+Space"),
        (Action::CaptureScreen, "CmdOrCtrl+Alt+Shift+3"),
        (Action::CaptureRegion, "CmdOrCtrl+Alt+Shift+4"),
        (Action::CaptureWindow, "CmdOrCtrl+Alt+Shift+5"),
        (Action::RecordRegion, "CmdOrCtrl+Alt+Shift+R"),
        (Action::RecordWindow, ""),
        (Action::RecordScreen, ""),
        (Action::StopRecording, "CmdOrCtrl+Alt+Shift+S"),
        (Action::TogglePause, "CmdOrCtrl+Alt+Shift+P"),
    ]
    .into_iter()
    .map(|(a, s)| (a, s.to_string()))
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Mac,
    Windows,
}

impl Os {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else {
            Os::Windows
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Chord {
    cmd_or_ctrl: bool,
    ctrl: bool,
    sup: bool,
    alt: bool,
    shift: bool,
    key: String,
}

const NAMED_KEYS: &[&str] = &[
    "Space",
    "Enter",
    "Tab",
    "Backspace",
    "Delete",
    "Escape",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Up",
    "Down",
    "Left",
    "Right",
    "Minus",
    "Equal",
    "Comma",
    "Period",
    "Slash",
    "Backslash",
    "Semicolon",
    "Quote",
    "Backquote",
    "BracketLeft",
    "BracketRight",
    "PrintScreen",
];

fn normalize_key(raw: &str) -> Option<String> {
    let k = raw.trim();
    if k.len() == 1 {
        let c = k.chars().next()?.to_ascii_uppercase();
        return (c.is_ascii_alphanumeric()).then(|| c.to_string());
    }
    if let Some(n) = k.strip_prefix(['F', 'f']).and_then(|n| n.parse::<u8>().ok()) {
        return (1..=24).contains(&n).then(|| format!("F{n}"));
    }
    NAMED_KEYS.iter().find(|n| n.eq_ignore_ascii_case(k)).map(|n| n.to_string())
}

fn parse(accel: &str) -> AppResult<Chord> {
    let bad = |m: &str| AppError::InvalidSettings(format!("shortcut \"{accel}\": {m}"));
    let parts: Vec<&str> = accel.split('+').map(str::trim).filter(|p| !p.is_empty()).collect();
    let (key, mods) = parts.split_last().ok_or_else(|| bad("empty"))?;
    let mut c =
        Chord { key: normalize_key(key).ok_or_else(|| bad("unsupported key"))?, ..Default::default() };
    for m in mods {
        match m.to_ascii_lowercase().as_str() {
            "cmdorctrl" | "commandorcontrol" => c.cmd_or_ctrl = true,
            "ctrl" | "control" => c.ctrl = true,
            "cmd" | "command" | "super" | "meta" | "win" => c.sup = true,
            "alt" | "option" => c.alt = true,
            "shift" => c.shift = true,
            _ => return Err(bad("unknown modifier")),
        }
    }
    Ok(c)
}

impl Chord {
    /// Canonical text form (stable order) used for storage and display.
    fn canonical(&self) -> String {
        let mut p = Vec::new();
        if self.cmd_or_ctrl {
            p.push("CmdOrCtrl");
        }
        if self.ctrl {
            p.push("Ctrl");
        }
        if self.sup {
            p.push("Super");
        }
        if self.alt {
            p.push("Alt");
        }
        if self.shift {
            p.push("Shift");
        }
        p.push(&self.key);
        p.join("+")
    }

    /// Physical modifiers on `os`: (cmd/win, ctrl, alt, shift).
    fn resolved(&self, os: Os) -> (bool, bool, bool, bool, &str) {
        let (cmd, ctrl) = match os {
            Os::Mac => (self.sup || self.cmd_or_ctrl, self.ctrl),
            Os::Windows => (self.sup, self.ctrl || self.cmd_or_ctrl),
        };
        (cmd, ctrl, self.alt, self.shift, self.key.as_str())
    }
}

/// Shortcuts the operating system owns; registering them would break expected behavior.
fn reserved(c: &Chord, os: Os) -> bool {
    let (cmd, ctrl, alt, shift, key) = c.resolved(os);
    match os {
        Os::Mac => matches!(
            (cmd, ctrl, alt, shift, key),
            (true, false, false, false, "Q" | "W" | "Tab" | "Space" | "H" | "M")
                | (true, false, false, true, "3" | "4" | "5" | "6")
                | (true, true, false, true, "3" | "4")
                | (false, true, false, false, "Space")
                | (true, true, false, false, "Q" | "Space")
                | (true, false, true, false, "Escape")
        ),
        Os::Windows => matches!(
            (cmd, ctrl, alt, shift, key),
            (false, false, true, false, "F4" | "Tab")
                | (false, true, true, false, "Delete")
                | (false, true, false, true, "Escape")
                | (true, false, false, true, "S")
                | (true, false, false, false, "L" | "D" | "E" | "R" | "Tab")
        ),
    }
}

/// Validates and normalizes one accelerator for `os`. Returns the canonical form.
pub fn normalize(accel: &str, os: Os) -> AppResult<String> {
    let c = parse(accel)?;
    let (cmd, ctrl, alt, _shift, key) = c.resolved(os);
    let function_like = key.starts_with('F') && key.len() > 1 || key == "PrintScreen";
    // Without ⌘/Ctrl/Alt a global shortcut would swallow normal typing (Shift+A = "A").
    if !(cmd || ctrl || alt) && !function_like {
        return Err(AppError::InvalidSettings(format!(
            "shortcut \"{accel}\" needs {} or Alt",
            if os == Os::Mac { "⌘, Ctrl" } else { "Ctrl, Win" }
        )));
    }
    if reserved(&c, os) {
        return Err(AppError::InvalidSettings(format!("shortcut \"{accel}\" is reserved by the system")));
    }
    Ok(c.canonical())
}

/// Validates a whole set: every accelerator valid and no two actions sharing a key chord.
pub fn validate(set: &Shortcuts, os: Os) -> AppResult<()> {
    let mut seen: BTreeMap<(bool, bool, bool, bool, String), Action> = BTreeMap::new();
    for (action, accel) in set.iter().filter(|(_, a)| !a.is_empty()) {
        normalize(accel, os)?;
        let c = parse(accel)?;
        let (cmd, ctrl, alt, shift, key) = c.resolved(os);
        if let Some(other) = seen.insert((cmd, ctrl, alt, shift, key.to_string()), *action) {
            return Err(AppError::InvalidSettings(format!("{other:?} and {action:?} use the same shortcut")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_on_both_systems() {
        validate(&defaults(), Os::Mac).unwrap();
        validate(&defaults(), Os::Windows).unwrap();
        assert_eq!(defaults().len(), ALL_ACTIONS.len());
    }

    #[test]
    fn normalizes_spelling_and_order() {
        assert_eq!(normalize("shift+alt+cmdorctrl+r", Os::Mac).unwrap(), "CmdOrCtrl+Alt+Shift+R");
        assert_eq!(normalize("Ctrl+Option+f5", Os::Mac).unwrap(), "Ctrl+Alt+F5");
        assert_eq!(normalize("Super+Alt+space", Os::Windows).unwrap(), "Super+Alt+Space");
    }

    #[test]
    fn rejects_garbage_and_typing_keys() {
        assert!(normalize("", Os::Mac).is_err());
        assert!(normalize("Ctrl+Hyper+A", Os::Mac).is_err());
        assert!(normalize("Ctrl+Ü", Os::Mac).is_err());
        assert!(normalize("Shift+A", Os::Mac).is_err(), "would hijack typing");
        assert!(normalize("A", Os::Windows).is_err());
        assert!(normalize("F13", Os::Mac).is_ok(), "function keys are fine alone");
        assert!(normalize("PrintScreen", Os::Windows).is_ok());
    }

    #[test]
    fn rejects_system_shortcuts() {
        assert!(normalize("CmdOrCtrl+Shift+4", Os::Mac).is_err(), "macOS screenshot");
        assert!(normalize("CmdOrCtrl+Q", Os::Mac).is_err());
        assert!(normalize("CmdOrCtrl+Space", Os::Mac).is_err(), "Spotlight");
        assert!(normalize("Super+Shift+S", Os::Windows).is_err(), "Snipping Tool");
        assert!(normalize("Alt+F4", Os::Windows).is_err());
        // Same text, different meaning per OS: Cmd+Shift+4 is free on Windows (Ctrl+Shift+4).
        assert!(normalize("CmdOrCtrl+Shift+4", Os::Windows).is_ok());
    }

    #[test]
    fn detects_duplicates_after_resolution() {
        let mut s = defaults();
        s.insert(Action::RecordScreen, "CmdOrCtrl+Alt+Shift+R".into());
        assert!(validate(&s, Os::Mac).is_err());
        // "Ctrl+X" and "CmdOrCtrl+X" collide on Windows only.
        let mut s = Shortcuts::new();
        s.insert(Action::CaptureRegion, "Ctrl+Alt+X".into());
        s.insert(Action::CaptureWindow, "CmdOrCtrl+Alt+X".into());
        assert!(validate(&s, Os::Mac).is_ok());
        assert!(validate(&s, Os::Windows).is_err());
    }

    #[test]
    fn empty_disables() {
        let mut s = defaults();
        s.insert(Action::OpenPanel, String::new());
        s.insert(Action::CaptureRegion, String::new());
        validate(&s, Os::Mac).unwrap();
    }

    #[test]
    fn serializes_actions_in_camel_case() {
        let json = serde_json::to_value(defaults()).unwrap();
        assert_eq!(json["openPanel"], "CmdOrCtrl+Alt+Shift+Space");
        assert_eq!(json["recordWindow"], "");
    }
}
