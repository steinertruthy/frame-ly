## [0.1.0] - 2026-11-15

Erste öffentliche Version.

### Added

- Screenshots vom ganzen Bildschirm, einem Fenster oder einem Bereich, mit Bearbeitung direkt an der Auswahl: Pfeile, Formen, Text, Markierung, Weichzeichnen und Zuschneiden, mit Rückgängig und Wiederholen. In die Zwischenablage kopieren oder sichern.
- Aufnahme des ganzen Bildschirms, eines Fensters oder eines Bereichs mit Systemton und Mikrofon, Pause und Fortsetzen, Countdown, H.264 oder HEVC und geschätzter Dateigröße vor dem Start.
- Schnellzugriff-Panel in der Menüleiste oder im Infobereich, globale Kurzbefehle für jeden Aufnahmemodus und eine Befehlszeile (`--capture`, `--record`, `--panel`, `--show`, `--edit`, `--transcribe`).
- Mediathek für Screenshots und Aufnahmen sowie Einstellungen nach Kategorien.
- Oberfläche auf Deutsch, Englisch, Portugiesisch (Brasilien), Spanisch und Französisch.
- Transkription von Aufnahmen und importierten Audio- oder Videodateien auf deinem Computer (whisper.cpp), mit automatischer Spracherkennung, Wortzeiten, Bearbeitung und Export als TXT, SRT, VTT, Markdown und PDF.
- Zusammenfassungen und Themen mit FrameLy AI auf deinem Computer (llama.cpp), Apple Intelligence ab macOS 26 oder Anthropic und OpenAI mit deinem eigenen API-Schlüssel – nach deiner Zustimmung.
- Modellverwaltung mit per SHA-256 geprüften Downloads, Pause und Fortsetzen sowie einem Profil passend zum Arbeitsspeicher.
- Modus „Nur lokal“, der jede Verbindung blockiert.

### Known limitations

- Der Windows-Rekorder wurde für Vollbildaufnahmen auf echter Hardware geprüft; Fenster- und Bereichsaufnahme, Pause, Ton und lokale KI brauchen noch weitere Tests auf Windows-PCs.
- Die Installer sind noch nicht mit einer Apple Developer ID oder einem Windows-Codesignaturzertifikat signiert.
- Versionen gibt es nur für Macs mit Apple Chip und 64-Bit-Windows mit Intel/AMD.
- Transkripte unterscheiden keine Sprecher.
- Eine Aufnahme erfasst jeweils einen Bildschirm.
