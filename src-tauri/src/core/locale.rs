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
    pub open: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
}

pub fn tray_strings(lang: Lang) -> TrayStrings {
    match lang {
        Lang::PtBr => TrayStrings {
            open: "Abrir FrameLy",
            quit: "Sair do FrameLy",
            tooltip: "FrameLy",
        },
        Lang::En => TrayStrings {
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
