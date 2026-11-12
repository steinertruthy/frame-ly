//! Resolves the UI language for strings rendered by Rust (tray menu, notifications).
//! The web UI has its own dictionaries; both follow the same `Language` setting.

use super::settings::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    PtBr,
    En,
    Es,
    Fr,
    De,
}

/// `Language::System` follows the OS locale: any Portuguese locale → pt-BR, otherwise English.
pub fn resolve(setting: Language, system_locale: Option<&str>) -> Lang {
    match setting {
        Language::PtBr => Lang::PtBr,
        Language::En => Lang::En,
        Language::Es => Lang::Es,
        Language::Fr => Lang::Fr,
        Language::De => Lang::De,
        Language::System => {
            let l = system_locale.unwrap_or("").to_ascii_lowercase();
            match l.get(..2) {
                Some("pt") => Lang::PtBr,
                Some("es") => Lang::Es,
                Some("fr") => Lang::Fr,
                Some("de") => Lang::De,
                _ => Lang::En,
            }
        }
    }
}

pub fn current(setting: Language) -> Lang {
    resolve(setting, sys_locale::get_locale().as_deref())
}

/// Text shown by native UI (tray menu, notifications) — the web UI has its own dictionaries.
pub struct TrayStrings {
    pub screenshot: &'static str,
    pub capture_screen: &'static str,
    pub capture_window: &'static str,
    pub record_screen: &'static str,
    pub record_window: &'static str,
    pub record_region: &'static str,
    pub stop: &'static str,
    pub pause: &'static str,
    pub resume: &'static str,
    pub open: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
    pub screenshot_saved: &'static str,
    pub recording_saved: &'static str,
}

pub fn tray_strings(lang: Lang) -> TrayStrings {
    match lang {
        Lang::PtBr => TrayStrings {
            screenshot: "Fazer screenshot",
            capture_screen: "Capturar a tela inteira",
            capture_window: "Capturar uma janela",
            record_screen: "Gravar a tela inteira",
            record_window: "Gravar uma janela",
            record_region: "Gravar uma área",
            stop: "Parar gravação",
            pause: "Pausar",
            resume: "Continuar",
            open: "Abrir o FrameLy",
            settings: "Ajustes…",
            quit: "Sair do FrameLy",
            tooltip: "FrameLy",
            screenshot_saved: "Screenshot salvo",
            recording_saved: "Gravação salva",
        },
        Lang::En => TrayStrings {
            screenshot: "Take Screenshot",
            capture_screen: "Capture Entire Screen",
            capture_window: "Capture a Window",
            record_screen: "Record Entire Screen",
            record_window: "Record a Window",
            record_region: "Record an Area",
            stop: "Stop Recording",
            pause: "Pause",
            resume: "Resume",
            open: "Open FrameLy",
            settings: "Settings…",
            quit: "Quit FrameLy",
            tooltip: "FrameLy",
            screenshot_saved: "Screenshot saved",
            recording_saved: "Recording saved",
        },
        Lang::Es => TrayStrings {
            screenshot: "Hacer captura",
            capture_screen: "Capturar toda la pantalla",
            capture_window: "Capturar una ventana",
            record_screen: "Grabar toda la pantalla",
            record_window: "Grabar una ventana",
            record_region: "Grabar un área",
            stop: "Detener grabación",
            pause: "Pausar",
            resume: "Reanudar",
            open: "Abrir FrameLy",
            settings: "Ajustes…",
            quit: "Salir de FrameLy",
            tooltip: "FrameLy",
            screenshot_saved: "Captura guardada",
            recording_saved: "Grabación guardada",
        },
        Lang::Fr => TrayStrings {
            screenshot: "Faire une capture",
            capture_screen: "Capturer tout l’écran",
            capture_window: "Capturer une fenêtre",
            record_screen: "Enregistrer tout l’écran",
            record_window: "Enregistrer une fenêtre",
            record_region: "Enregistrer une zone",
            stop: "Arrêter l’enregistrement",
            pause: "Pause",
            resume: "Reprendre",
            open: "Ouvrir FrameLy",
            settings: "Réglages…",
            quit: "Quitter FrameLy",
            tooltip: "FrameLy",
            screenshot_saved: "Capture enregistrée",
            recording_saved: "Enregistrement sauvegardé",
        },
        Lang::De => TrayStrings {
            screenshot: "Bildschirmfoto aufnehmen",
            capture_screen: "Ganzen Bildschirm aufnehmen",
            capture_window: "Fenster aufnehmen",
            record_screen: "Ganzen Bildschirm aufzeichnen",
            record_window: "Fenster aufzeichnen",
            record_region: "Bereich aufzeichnen",
            stop: "Aufzeichnung beenden",
            pause: "Pausieren",
            resume: "Fortsetzen",
            open: "FrameLy öffnen",
            settings: "Einstellungen …",
            quit: "FrameLy beenden",
            tooltip: "FrameLy",
            screenshot_saved: "Bildschirmfoto gespeichert",
            recording_saved: "Aufzeichnung gespeichert",
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
    fn system_detects_all_supported_languages() {
        assert_eq!(resolve(Language::System, Some("es-MX")), Lang::Es);
        assert_eq!(resolve(Language::System, Some("fr_CA")), Lang::Fr);
        assert_eq!(resolve(Language::System, Some("de-AT")), Lang::De);
        assert_eq!(resolve(Language::System, Some("it-IT")), Lang::En);
        assert_eq!(resolve(Language::Fr, Some("pt-BR")), Lang::Fr);
    }

    #[test]
    fn system_follows_locale() {
        assert_eq!(resolve(Language::System, Some("pt-PT")), Lang::PtBr);
        assert_eq!(resolve(Language::System, Some("PT_br")), Lang::PtBr);
        assert_eq!(resolve(Language::System, Some("ja-JP")), Lang::En);
        assert_eq!(resolve(Language::System, None), Lang::En);
    }
}
