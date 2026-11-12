//! Resolves the UI language for strings rendered by Rust (tray menu, notifications).
//! The web UI has its own dictionaries; both follow the same `Language` setting.

use super::settings::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    PtBr,
    En,
}

/// `Language::System` follows the OS locale: any Portuguese locale → pt-BR, otherwise English.
pub fn resolve(setting: Language, system_locale: Option<&str>) -> Lang {
    match setting {
        Language::PtBr => Lang::PtBr,
        Language::En => Lang::En,
        Language::System => match system_locale {
            Some(l) if l.to_ascii_lowercase().starts_with("pt") => Lang::PtBr,
            _ => Lang::En,
        },
    }
}

pub fn current(setting: Language) -> Lang {
    resolve(setting, sys_locale::get_locale().as_deref())
}

pub struct TrayStrings {
    pub record_region: &'static str,
    pub record_window: &'static str,
    pub record_screen: &'static str,
    pub stop: &'static str,
    pub pause: &'static str,
    pub resume: &'static str,
    pub capture_region: &'static str,
    pub capture_window: &'static str,
    pub capture_screen: &'static str,
    pub open: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
}

pub fn tray_strings(lang: Lang) -> TrayStrings {
    match lang {
        Lang::PtBr => TrayStrings {
            record_region: "Gravar região",
            record_window: "Gravar janela",
            record_screen: "Gravar tela inteira",
            stop: "Parar gravação",
            pause: "Pausar",
            resume: "Retomar",
            capture_region: "Capturar região",
            capture_window: "Capturar janela",
            capture_screen: "Capturar tela inteira",
            open: "Abrir FrameLy",
            quit: "Sair do FrameLy",
            tooltip: "FrameLy",
        },
        Lang::En => TrayStrings {
            record_region: "Record Region",
            record_window: "Record Window",
            record_screen: "Record Full Screen",
            stop: "Stop Recording",
            pause: "Pause",
            resume: "Resume",
            capture_region: "Capture Region",
            capture_window: "Capture Window",
            capture_screen: "Capture Full Screen",
            open: "Open FrameLy",
            quit: "Quit FrameLy",
            tooltip: "FrameLy",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_setting_wins() {
        assert_eq!(resolve(Language::En, Some("pt-BR")), Lang::En);
        assert_eq!(resolve(Language::PtBr, Some("en-US")), Lang::PtBr);
    }

    #[test]
    fn system_follows_locale() {
        assert_eq!(resolve(Language::System, Some("pt-PT")), Lang::PtBr);
        assert_eq!(resolve(Language::System, Some("PT_br")), Lang::PtBr);
        assert_eq!(resolve(Language::System, Some("de-DE")), Lang::En);
        assert_eq!(resolve(Language::System, None), Lang::En);
    }
}
