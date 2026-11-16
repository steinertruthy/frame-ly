## [0.3.1] - 2026-11-16

Eigener Windows-Installer von FrameLy, bearbeitbare Transkript-Titel und vollständige Credits.

### Added

- Transkripte haben einen eigenen Titel: Klicke im Transkript darauf, um ihn umzubenennen (Enter speichert, Esc bricht ab). Die Mediendatei wird nie umbenannt; der Titel erscheint in der Liste, in der Suche und in den Dateinamen beim Export.
- Einstellungen › Über führt jede Open-Source-Komponente und jedes KI-Modell auf, das FrameLy nutzt – mit Lizenz und Link.

### Changed

- Installer und Deinstaller unter Windows stammen jetzt vollständig von FrameLy: Vollfenster-Seiten in den Farben der App, mit Installationsort, Desktop-Verknüpfung und „FrameLy jetzt öffnen“. Eine bestehende Installation wird direkt aktualisiert; Einstellungen, Transkripte und KI-Modelle bleiben erhalten.

### Known limitations

- Die Installer-Seiten wurden noch nicht bei einer Anzeigeskalierung über 100 % geprüft.

## [0.3.0] - 2026-11-16

FrameLy aktualisiert sich jetzt selbst, und der Windows-Installer trägt das FrameLy-Design.

### Added

- Automatische Updates: FrameLy prüft im Hintergrund die Versionen auf GitHub (abschaltbar unter Einstellungen › Über, nie im Modus „Nur lokal“), zeigt einen dezenten Hinweis mit den Neuerungen und aktualisiert mit einem Klick – Download-Fortschritt, Signaturprüfung, Installation und Neustart. Eine laufende Aufnahme wird nie unterbrochen: Das Update wird nach ihrem Ende installiert. `framely --update` sucht und installiert über die Befehlszeile.

### Changed

- Der Windows-Installer hat jetzt das FrameLy-Design, die fünf Sprachen der App, eigene Texte auf Begrüßungs- und Abschlussseite und eine Deinstallationsoption, die auch gespeicherte API-Schlüssel entfernt.

### Known limitations

- Die Versionen 0.1.0 und 0.2.0 haben keinen Updater: Lade die erste Version mit Updater einmal von der Website.
- macOS-Versionen sind noch nicht notariell beglaubigt, daher kann macOS nach einem Update erneut nach der Berechtigung zur Bildschirmaufnahme fragen.

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
