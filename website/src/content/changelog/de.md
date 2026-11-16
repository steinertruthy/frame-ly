## [0.2.0] - 2026-11-16

FrameLy AI Cloud ist online und jetzt der Standard für Zusammenfassungen und Themen.

### Added

- FrameLy AI Cloud in den veröffentlichten Versionen: Zusammenfassungen und Themen in der Cloud, kostenlos, ohne Konto und Schlüssel, mit einem Tageslimit pro Computer. Ein KI-Partneranbieter erzeugt das Ergebnis und kann die Inhalte zur Verbesserung seiner Dienste nutzen – die App weist vor der ersten Nutzung darauf hin.

### Changed

- Neue Installationen und Einstellungen, die noch auf dem bisherigen Standard standen, nutzen FrameLy AI Cloud für Zusammenfassungen und Themen; jede andere Wahl bleibt erhalten. FrameLy fragt vor dem Senden um Erlaubnis, und der Modus „Nur lokal“ blockiert die Cloud weiterhin.
- Die Ersteinrichtung lädt das lokale Zusammenfassungsmodell nur, wenn FrameLy AI Local Zusammenfassungen oder Themen erzeugt.

### Fixed

- Bestätigungsdialoge (etwa die Zustimmungsabfrage) öffnen sich jetzt über dem Fenster „Neue Transkription“, und Esc schließt zuerst den Dialog.

### Known limitations

- FrameLy AI Cloud ist im Europäischen Wirtschaftsraum, in der Schweiz und im Vereinigten Königreich nicht verfügbar; dort funktionieren FrameLy AI Local, Apple Intelligence und dein eigener Anthropic- oder OpenAI-Schlüssel.

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
