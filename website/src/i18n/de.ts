import type { Messages } from './index';

export const de: Messages = {
  "meta": {
    "title": "FrameLy — Bildschirmaufnahme, Screenshots und Transkription auf deinem Computer",
    "description": "FrameLy erstellt Screenshots und nimmt den Bildschirm unter macOS und Windows auf – und transkribiert, fasst zusammen und gliedert Aufnahmen in Themen, mit einer KI, die vollständig auf deinem Computer laufen kann.",
    "ogAlt": "FrameLy-Fenster mit einer Aufnahme samt Transkript, Zusammenfassung und Themen"
  },
  "a11y": {
    "skip": "Zum Inhalt springen",
    "menu": "Menü öffnen",
    "close": "Menü schließen",
    "language": "Sprache",
    "home": "FrameLy Startseite",
    "screenshotOf": "Screenshot von FrameLy:",
    "playPause": "Demovideo abspielen oder pausieren"
  },
  "nav": {
    "overview": "Überblick",
    "features": "Funktionen",
    "ai": "Künstliche Intelligenz",
    "how": "So funktioniert’s",
    "download": "Download",
    "cta": "Herunterladen"
  },
  "os": {
    "macos": "macOS",
    "windows": "Windows"
  },
  "hero": {
    "eyebrow": "Bildschirmaufnahme · Screenshots · Transkription",
    "title": "Festhalten. Aufnehmen.\nVerstehen.",
    "subtitle": "FrameLy ist ein schneller, unaufdringlicher Bildschirmrekorder und Screenshot-Werkzeug für macOS und Windows, der Aufnahmen in Transkripte, Zusammenfassungen und Themen verwandelt – mit einer KI, die vollständig auf deinem Computer laufen kann.",
    "ctaFor": "Für {os} herunterladen",
    "ctaGeneric": "FrameLy herunterladen",
    "ctaSecondary": "So funktioniert’s",
    "detected": "Erkannt: {os}",
    "notDetected": "Wähle unten dein System",
    "compat": "macOS 14+ mit Apple Silicon · Windows 10 22H2+ (x64)",
    "version": "Version {version}",
    "noRelease": "Installer werden vorbereitet",
    "free": "Kostenlos und Open Source (MIT)"
  },
  "intro": {
    "eyebrow": "Überblick",
    "title": "Eine App von der Aufnahme bis zur Aussage.",
    "body": "FrameLy lebt in der Menüleiste oder im Infobereich. Mach einen Screenshot und beschrifte ihn direkt in der Auswahl, nimm den Bildschirm mit Systemton und Mikrofon auf und lass FrameLy die Aufnahme transkribieren und das Wichtigste herausarbeiten. Deine Dateien bleiben in Ordnern deiner Wahl.",
    "points": [
      {
        "title": "Nativ und leicht",
        "body": "Mit Rust und den Aufnahme-APIs des jeweiligen Systems gebaut – ScreenCaptureKit unter macOS, Windows.Graphics.Capture unter Windows. Kein FFmpeg, keine Browser-Erweiterung."
      },
      {
        "title": "Für die Tastatur gemacht",
        "body": "Globale Kurzbefehle für jeden Aufnahmemodus, ein Schnellzugriff-Panel und eine Befehlszeile für Raycast, Alfred, Kurzbefehle oder AutoHotkey."
      },
      {
        "title": "Die KI deiner Wahl",
        "body": "Die Transkription läuft immer auf deinem Computer. Zusammenfassungen und Themen nutzen standardmäßig FrameLy AI Cloud – oder die lokalen Modelle von FrameLy, Apple Intelligence oder deinen eigenen API-Schlüssel."
      }
    ]
  },
  "features": {
    "eyebrow": "Funktionen",
    "title": "Fünf Werkzeuge, ein ruhiger Ablauf.",
    "benefit": "Warum es hilft",
    "items": {
      "screenshot": {
        "kicker": "Screenshots",
        "title": "Auswählen, beschriften, teilen – ohne die Auswahl zu verlassen.",
        "body": "Nimm den ganzen Bildschirm, ein Fenster oder einen beliebigen Bereich auf. Pfeile, Formen, Text, Markierung, Weichzeichnen und Zuschneiden erscheinen direkt an der Auswahl, mit Rückgängig und Wiederholen. In die Zwischenablage kopieren oder im eigenen Ordner sichern.",
        "benefit": "Erkläre einen Fehler oder ein Design-Detail in Sekunden – mit unkenntlich gemachten sensiblen Stellen.",
        "alt": "Screenshot-Editor von FrameLy mit Beschriftungswerkzeugen um einen ausgewählten Bereich"
      },
      "recording": {
        "kicker": "Bildschirmaufnahme",
        "title": "Bildschirm, Fenster oder Bereich aufnehmen – mit Ton.",
        "body": "Mische Systemton und Mikrofon, pausiere und setze fort, und beende über eine kleine Aufnahmeleiste oder einen Kurzbefehl. H.264 oder HEVC, mit Countdown und geschätzter Dateigröße vor dem Start.",
        "benefit": "Anleitungen, Demos und Besprechungsnotizen, ganz ohne Studio.",
        "alt": "FrameLy-Startbildschirm mit Kurzbefehlen für Screenshots und Aufnahmen"
      },
      "transcription": {
        "kicker": "Transkription",
        "title": "Jedes Wort, auf deinem Computer.",
        "body": "FrameLy transkribiert Aufnahmen sowie importierte Audio- und Videodateien mit offenen Sprachmodellen, die lokal laufen, erkennt die Sprache automatisch und speichert die Zeit jedes Wortes. Klicke auf eine Zeile, um zu dieser Stelle zu springen, bearbeite den Text und exportiere als TXT, SRT, VTT, Markdown oder PDF.",
        "benefit": "Durchsuchbare Aufnahmen und fertige Untertitel, ohne das Audio hochzuladen.",
        "alt": "Transkript in FrameLy mit Videoplayer und Text mit Zeitmarken"
      },
      "summaries": {
        "kicker": "Zusammenfassungen",
        "title": "Die Besprechung in einem Absatz.",
        "body": "Erstelle aus dem Transkript eine kurze, ausgewogene oder ausführliche Zusammenfassung – mit FrameLy AI Cloud (Standard), den lokalen Modellen von FrameLy, Apple Intelligence auf unterstützten Macs oder mit Anthropic oder OpenAI und deinem eigenen Schlüssel.",
        "benefit": "Teile Entscheidungen und nächste Schritte statt eines 40-minütigen Videos.",
        "alt": "Zusammenfassung in FrameLy mit Kernpunkten und nächsten Schritten"
      },
      "topics": {
        "kicker": "Themen",
        "title": "Eine Karte des Gesagten.",
        "body": "FrameLy gliedert das Transkript in Themen und Unterthemen, jeweils mit ihrer Zeit in der Aufnahme. Öffne ein Thema, um seine Punkte zu lesen, oder klicke auf die Zeit, um diese Stelle abzuspielen.",
        "benefit": "Finde in einer langen Aufnahme auf einen Blick die richtige Stelle.",
        "alt": "Themen in FrameLy mit Themen und Unterthemen samt Zeitmarken"
      }
    }
  },
  "ai": {
    "eyebrow": "Künstliche Intelligenz",
    "title": "Drei Arten zu denken. Du entscheidest, wo.",
    "subtitle": "Die Transkription findet immer auf deinem Computer statt. Für Zusammenfassungen und Themen wählst du die passende Engine – und wechselst jederzeit in den Einstellungen.",
    "modes": {
      "local": {
        "name": "FrameLy AI Local",
        "label": "Auf deinem Computer",
        "tag": "Auf diesem Computer · funktioniert offline",
        "body": "Offene Modelle, die die App herunterlädt und auf deinem Computer ausführt. Kein Konto, kein Schlüssel, keine Kosten pro Nutzung. Sind die Modelle geladen, funktionieren Transkription, Zusammenfassungen und Themen ohne Internet.",
        "points": [
          "Sprache zu Text: 3 Genauigkeitsstufen (181 MB bis 1 GB)",
          "Sprachmodell: 3 Profile (2,6 bis 5,3 GB), passend zu deinem Arbeitsspeicher vorgeschlagen",
          "Downloads per SHA-256 geprüft; jederzeit pausieren, fortsetzen oder entfernen",
          "Der Modus „Nur lokal“ blockiert jede Verbindung"
        ]
      },
      "cloud": {
        "name": "In der Cloud",
        "label": "FrameLy AI Cloud · Anthropic · OpenAI",
        "tag": "Standard für Zusammenfassungen und Themen",
        "body": "Zusammenfassungen und Themen entstehen auf Servern statt auf deinem Computer. Gesendet wird nur der Text des Transkripts – nie Audio oder Video – und erst nach deiner Zustimmung.",
        "options": [
          {
            "name": "FrameLy AI Cloud",
            "status": "Verfügbar · kostenlos",
            "body": "Der eigene Cloud-Dienst von FrameLy und der Standard für Zusammenfassungen und Themen: kostenlos, ohne Konto und Schlüssel, mit einem Tageslimit pro Computer. Ein KI-Partneranbieter erzeugt das Ergebnis und kann die Inhalte zur Verbesserung seiner Dienste nutzen – für sensible Aufnahmen lieber die lokale KI. Nicht verfügbar im EWR, in der Schweiz und im Vereinigten Königreich."
          },
          {
            "name": "Anthropic (Claude) oder OpenAI",
            "status": "Verfügbar · kostenpflichtig, mit deinem Schlüssel",
            "body": "Nutze deinen eigenen API-Schlüssel. Standardmäßig aus: externe Anbieter einschalten, Schlüssel hinzufügen und jeden Anbieter erlauben. Die Nutzung wird über dein Konto beim Anbieter abgerechnet."
          }
        ],
        "points": [
          "Schlüssel im macOS-Schlüsselbund oder in der Windows-Anmeldeinformationsverwaltung",
          "Der Modus „Nur lokal“ blockiert alle Cloud-Optionen"
        ]
      },
      "apple": {
        "name": "Apple Intelligence",
        "label": "Auf deinem Mac",
        "tag": "macOS 26 oder neuer",
        "body": "Auf einem Mac mit Apple Intelligence kann FrameLy Zusammenfassungen und Themen mit Apples Modell auf dem Gerät erstellen. In FrameLy muss nichts geladen werden, kein Schlüssel nötig.",
        "points": [
          "Erfordert macOS 26 oder neuer",
          "Ein Mac, der Apple Intelligence unterstützt, mit aktivierter Funktion in den Systemeinstellungen",
          "In den Sprachen, die Apple Intelligence unterstützt",
          "Unter Windows nicht verfügbar"
        ]
      }
    },
    "table": {
      "caption": "Vergleich der KI-Optionen für Zusammenfassungen und Themen",
      "feature": "Option",
      "rows": {
        "availability": "Verfügbarkeit",
        "where": "Verarbeitung",
        "connection": "Internet",
        "platforms": "Plattformen",
        "hardware": "Hardware",
        "cost": "Kosten",
        "setup": "Einrichtung"
      },
      "columns": {
        "local": {
          "name": "FrameLy AI Local",
          "availability": "Verfügbar",
          "where": "Auf deinem Computer",
          "connection": "Nur zum Laden der Modelle",
          "platforms": "macOS und Windows",
          "hardware": "Apple Silicon oder x64-CPU mit AVX2; je nach Profil etwa 3 bis 6 GB freier Arbeitsspeicher",
          "cost": "Kostenlos",
          "setup": "Modelle in der App laden (insgesamt 3 bis 6,5 GB)"
        },
        "cloud": {
          "name": "FrameLy AI Cloud",
          "availability": "Verfügbar, außer im EWR, in der Schweiz und im Vereinigten Königreich",
          "where": "Dienst von FrameLy und ein KI-Partneranbieter",
          "connection": "Erforderlich",
          "platforms": "macOS und Windows",
          "hardware": "Jeder Computer, auf dem FrameLy läuft",
          "cost": "Kostenlos, mit Tageslimit",
          "setup": "Standard; bei der ersten Nutzung erlauben, kein Schlüssel"
        },
        "external": {
          "name": "Anthropic / OpenAI",
          "availability": "Verfügbar",
          "where": "Server von Anthropic oder OpenAI",
          "connection": "Erforderlich",
          "platforms": "macOS und Windows",
          "hardware": "Jeder Computer, auf dem FrameLy läuft",
          "cost": "Kostenpflichtig pro Nutzung, beim Anbieter",
          "setup": "Einschalten, API-Schlüssel hinzufügen, Anbieter erlauben"
        },
        "apple": {
          "name": "Apple Intelligence",
          "availability": "macOS 26 oder neuer",
          "where": "Auf deinem Mac",
          "connection": "Nicht nötig",
          "platforms": "macOS 26 oder neuer",
          "hardware": "Mac mit Unterstützung für Apple Intelligence",
          "cost": "Kostenlos",
          "setup": "Apple Intelligence in den Systemeinstellungen aktivieren"
        }
      },
      "note": "Die Transkription selbst nutzt immer die lokalen Sprachmodelle von FrameLy – unabhängig vom Modus für Zusammenfassungen und Themen."
    },
    "figure": "Die Modellverwaltung in den FrameLy-Einstellungen: jedes Modell installieren, prüfen oder entfernen. Größen und Lizenzen stammen aus der App.",
    "figureAlt": "FrameLy-Modellverwaltung mit Transkriptions- und Sprachmodellen samt Größe, Speicherbedarf und Lizenz"
  },
  "how": {
    "eyebrow": "So funktioniert’s",
    "title": "Vom ersten Kurzbefehl zur Zusammenfassung.",
    "steps": [
      {
        "title": "Installieren",
        "body": "Lade den Installer für dein System und öffne FrameLy. Es lebt in der Menüleiste oder im Infobereich."
      },
      {
        "title": "Zugriff erlauben",
        "body": "Erlaube die Bildschirmaufnahme und, wenn du möchtest, das Mikrofon. FrameLy erklärt, wofür jede Berechtigung dient."
      },
      {
        "title": "Aufnehmen",
        "body": "Nutze einen Kurzbefehl oder das Schnellzugriff-Panel für einen Screenshot oder eine Aufnahme von Bildschirm, Fenster oder Bereich."
      },
      {
        "title": "KI wählen",
        "body": "Zusammenfassungen und Themen nutzen FrameLy AI Cloud, sobald du es erlaubst. Lieber auf deinem Computer? Lade die lokalen Modelle in einem Schritt oder wähle Apple Intelligence oder deinen Schlüssel."
      },
      {
        "title": "Transkribieren",
        "body": "Schicke eine Aufnahme oder eine beliebige Audio- oder Videodatei an Transkriptionen. Lange Dateien machen dort weiter, wo sie aufgehört haben."
      },
      {
        "title": "Zusammenfassen und teilen",
        "body": "Erstelle Zusammenfassung und Themen, bearbeite sie und exportiere das Transkript als TXT, SRT, VTT, Markdown oder PDF."
      }
    ]
  },
  "uses": {
    "eyebrow": "Anwendungsfälle",
    "title": "Für die tägliche Arbeit gemacht.",
    "items": [
      {
        "title": "Fehlerberichte",
        "body": "Ein Screenshot mit Pfeil und unkenntlicher E-Mail-Adresse oder eine kurze Aufnahme der Schritte."
      },
      {
        "title": "Besprechungsnotizen",
        "body": "Nimm ein Gespräch mit Systemton und Mikrofon auf und erhalte Zusammenfassung und nächste Schritte."
      },
      {
        "title": "Anleitungen und Demos",
        "body": "Nimm eine Vorführung auf und exportiere Untertitel als SRT oder VTT."
      },
      {
        "title": "Kurse und Vorträge",
        "body": "Mach aus einer langen Aufnahme Themen, zwischen denen du springen kannst."
      },
      {
        "title": "Design-Reviews",
        "body": "Zeige Details mit Formen und Text direkt auf der Aufnahme."
      },
      {
        "title": "Sensible Inhalte",
        "body": "Behalte alles auf deinem Computer – mit lokaler KI und dem Modus „Nur lokal“."
      }
    ]
  },
  "demo": {
    "eyebrow": "In Aktion",
    "title": "Eine echte Aufnahme, von FrameLy transkribiert und zusammengefasst.",
    "caption": "Aufgenommen mit FrameLy unter macOS. Der Ton der Besprechung ist synthetische Sprache."
  },
  "download": {
    "eyebrow": "Download",
    "title": "Hol dir FrameLy.",
    "subtitle": "Kostenlos für macOS und Windows. Wähle dein System – deine Wahl wird gespeichert.",
    "choose": "Betriebssystem",
    "detectedHint": "Wir haben {os} erkannt.",
    "notDetectedHint": "Wir konnten dein System nicht erkennen. Wähle eines:",
    "button": "Für {os} herunterladen",
    "version": "Version",
    "released": "Veröffentlicht",
    "arch": "Architektur",
    "format": "Format",
    "size": "Größe",
    "checksum": "SHA-256",
    "checksums": "Alle Prüfsummen",
    "requirements": "Voraussetzungen",
    "install": "Installation",
    "previous": "Frühere Versionen",
    "noPrevious": "Noch keine früheren Versionen.",
    "changelog": "Neuerungen",
    "allReleases": "Alle Versionen auf GitHub",
    "portable": "Portabel (zip)",
    "none": "Für {os} gibt es noch keinen veröffentlichten Installer. Folge dem Repository, um bei Erscheinen informiert zu werden.",
    "unavailableArch": "Noch nicht verfügbar",
    "archNames": {
      "arm64": "Apple Silicon (arm64)",
      "x64": "x64 (Intel/AMD 64-Bit)",
      "universal": "Universal"
    },
    "formats": {
      "dmg": "Disk-Image (.dmg)",
      "exe": "Installer (.exe, NSIS)",
      "msi": "Installer (.msi)",
      "zip": "Portabel (.zip)"
    },
    "req": {
      "macos": [
        "macOS 14 Sonoma oder neuer",
        "Mac mit Apple Chip (M1 oder neuer)",
        "Intel-Macs: noch keine Version",
        "Apple-Intelligence-Funktionen: macOS 26 oder neuer"
      ],
      "windows": [
        "Windows 10 22H2 oder Windows 11",
        "x64 (Intel oder AMD 64-Bit) – ARM64: noch keine Version",
        "Microsoft Edge WebView2 (in Windows 11 enthalten; der Installer kann es einrichten)",
        "Lokale KI: CPU mit AVX2"
      ]
    },
    "steps": {
      "macos": [
        "Öffne die .dmg und ziehe FrameLy in den Ordner „Programme“.",
        "Diese Version ist noch nicht von Apple notariell beglaubigt. Öffne FrameLy beim ersten Mal, gehe zu Systemeinstellungen › Datenschutz & Sicherheit und klicke auf „Dennoch öffnen“.",
        "Erlaube die Bildschirmaufnahme (und das Mikrofon, falls genutzt), wenn FrameLy danach fragt."
      ],
      "windows": [
        "Starte den Installer -setup.exe. Er installiert nur für deinen Benutzer – ohne Administratorrechte.",
        "Der Installer ist noch nicht signiert, daher kann Windows SmartScreen warnen. Klicke auf „Weitere Informationen“ und dann auf „Trotzdem ausführen“.",
        "FrameLy startet im Infobereich. Nutze das Schnellzugriff-Panel oder Strg+Alt+Umschalt+Leertaste."
      ]
    },
    "verify": "Vergleiche den SHA-256-Wert der Datei mit dem obigen Wert: shasum -a 256 <Datei> unter macOS oder Get-FileHash <Datei> in PowerShell.",
    "windowsNote": "Windows: Die Vollbildaufnahme wurde auf echter Hardware getestet; Fenster- und Bereichsaufnahme, Pause, Ton und lokale KI werden noch auf weiteren Windows-PCs geprüft."
  },
  "faq": {
    "eyebrow": "Häufige Fragen",
    "title": "Fragen und Antworten.",
    "more": "Alle Fragen",
    "items": [
      {
        "q": "Ist FrameLy kostenlos?",
        "a": "Ja. FrameLy ist kostenlos und Open Source unter der MIT-Lizenz. FrameLy AI Cloud ist mit Tageslimit kostenlos, lokale KI und Apple Intelligence ebenfalls; Cloud-Anbieter, die du mit deinem eigenen Schlüssel verbindest, rechnen direkt mit dir ab."
      },
      {
        "q": "Verlassen meine Audio- oder Videodaten den Computer?",
        "a": "Nicht für die Transkription: Sie läuft immer auf deinem Computer. Zusammenfassungen und Themen nutzen standardmäßig FrameLy AI Cloud – dafür wird nach deiner Zustimmung der Text des Transkripts gesendet, nie Audio oder Video. Wähle FrameLy AI Local oder Apple Intelligence, um alles auf dem Computer zu behalten, oder schalte „Nur lokal“ ein."
      },
      {
        "q": "Was ist FrameLy AI Cloud?",
        "a": "Der kostenlose Cloud-Dienst von FrameLy für Zusammenfassungen und Themen und die Standardwahl. Kein Konto, kein Schlüssel; jeder Computer hat ein Tageslimit. Der Text des Transkripts wird an ihn gesendet, und ein KI-Partneranbieter erzeugt das Ergebnis; dieser Anbieter kann die Inhalte zur Verbesserung seiner Dienste nutzen, deshalb fragt die App vor der ersten Nutzung, und du kannst jederzeit zur lokalen KI wechseln. Im EWR, in der Schweiz und im Vereinigten Königreich ist er nicht verfügbar."
      },
      {
        "q": "Funktioniert FrameLy offline?",
        "a": "Screenshots, Aufnahmen und Transkription funktionieren ohne Internet, sobald das Sprachmodell geladen ist. Für Zusammenfassungen und Themen offline wähle FrameLy AI Local (nach dem Laden des Modells) oder Apple Intelligence; FrameLy AI Cloud und andere Cloud-Anbieter brauchen eine Verbindung."
      },
      {
        "q": "Wie viel Speicherplatz brauchen die KI-Modelle?",
        "a": "Das Sprache-zu-Text-Modell belegt je nach Genauigkeit 181 MB bis 1 GB, das Sprachmodell je nach Profil 2,6 bis 5,3 GB. FrameLy schlägt ein Profil passend zum Arbeitsspeicher vor, und du kannst Modelle jederzeit entfernen."
      },
      {
        "q": "Welche Macs unterstützen Apple Intelligence in FrameLy?",
        "a": "Macs, die Apple Intelligence unterstützen, mit macOS 26 oder neuer und aktiviertem Apple Intelligence in den Systemeinstellungen. FrameLy selbst läuft ab macOS 14."
      },
      {
        "q": "Läuft FrameLy auf Intel-Macs oder Windows on ARM?",
        "a": "Noch nicht. Die aktuellen Versionen sind für Macs mit Apple Chip und 64-Bit-Windows-PCs mit Intel/AMD."
      },
      {
        "q": "Kann FrameLy Sprecher unterscheiden?",
        "a": "Noch nicht. Die lokale Spracherkennung von FrameLy erkennt keine einzelnen Sprecher."
      },
      {
        "q": "Warum warnen macOS oder Windows bei der Installation?",
        "a": "Die Installer sind noch nicht mit einer Apple Developer ID oder einem Windows-Codesignaturzertifikat signiert. Du kannst jede Datei anhand ihrer SHA-256-Prüfsumme im Download-Bereich und auf GitHub prüfen."
      },
      {
        "q": "Wie aktualisiere ich FrameLy?",
        "a": "Ab Version 0.3.0 sucht FrameLy im Hintergrund nach neuen Versionen und bietet das Update mit einem Klick an – nie während einer Aufnahme; du kannst auch unter Einstellungen › Über suchen oder die Funktion abschalten. Frühere Versionen (0.1.0 und 0.2.0) aktualisieren sich nicht selbst: Lade die neueste Version einmal auf dieser Seite herunter. Unter macOS kann das System nach einem Update erneut nach der Berechtigung zur Bildschirmaufnahme fragen, da die App noch nicht notariell beglaubigt ist."
      },
      {
        "q": "In welchen Sprachen gibt es die App?",
        "a": "Die Oberfläche gibt es auf Deutsch, Englisch, Portugiesisch (Brasilien), Spanisch und Französisch. Die Transkription erkennt die gesprochene Sprache automatisch."
      }
    ]
  },
  "footer": {
    "tagline": "Bildschirmaufnahme, Screenshots und Transkription auf deinem Computer – für macOS und Windows.",
    "product": "Produkt",
    "resources": "Ressourcen",
    "legal": "Rechtliches",
    "github": "Quellcode",
    "releases": "Versionen",
    "issues": "Problem melden",
    "changelog": "Neuerungen",
    "faq": "Häufige Fragen",
    "privacy": "Datenschutz",
    "terms": "Nutzungsbedingungen",
    "license": "Lizenz",
    "rights": "© {year} FrameLy. Veröffentlicht unter der MIT-Lizenz.",
    "trademarks": "Apple, macOS und Apple Intelligence sind Marken der Apple Inc. Windows ist eine Marke von Microsoft. Claude und Anthropic sind Marken von Anthropic. OpenAI ist eine Marke von OpenAI."
  },
  "pages": {
    "pending": "Entwurf – rechtliche Prüfung ausstehend. Dieser Text beschreibt, wie FrameLy heute funktioniert; er ist keine Rechtsberatung.",
    "updated": "Zuletzt aktualisiert: {date}",
    "back": "Zur Startseite",
    "toc": "Auf dieser Seite",
    "notFound": {
      "title": "Seite nicht gefunden",
      "body": "Die gesuchte Seite existiert nicht oder wurde verschoben."
    },
    "changelog": {
      "title": "Neuerungen",
      "description": "Alle Versionen von FrameLy mit Neuerungen, Verbesserungen und Fehlerbehebungen.",
      "intro": "Alle wichtigen Änderungen an FrameLy. Die Versionen folgen der semantischen Versionierung.",
      "added": "Neu",
      "changed": "Verbesserungen",
      "fixed": "Fehlerbehebungen",
      "known": "Bekannte Einschränkungen",
      "assets": "Downloads",
      "release": "Version auf GitHub",
      "empty": "Noch keine veröffentlichten Versionen."
    },
    "faq": {
      "title": "Häufige Fragen",
      "description": "Antworten zu FrameLy: Preis, Datenschutz, KI-Modelle, Kompatibilität und Installation."
    },
    "license": {
      "title": "Lizenz",
      "description": "FrameLy ist Open Source unter der MIT-Lizenz.",
      "intro": "Der Quellcode von FrameLy steht unter der MIT-Lizenz. Die KI-Modelle, die FrameLy herunterlädt, haben eigene Lizenzen, die unten aufgeführt sind.",
      "models": "Von FrameLy geladene Modelle",
      "modelsBody": "Die Spracherkennung nutzt Whisper-Modelle (MIT) über whisper.cpp. Zusammenfassungen und Themen mit lokaler KI nutzen offene Sprachmodelle der Familien Gemma und Qwen unter Apache 2.0. Die Lizenz jedes Modells wird auch in der Modellverwaltung der App angezeigt.",
      "thirdParty": "Software von Drittanbietern",
      "thirdPartyBody": "FrameLy basiert unter anderem auf Tauri, Rust, Svelte, whisper.cpp und llama.cpp – jeweils unter eigener Lizenz.",
      "full": "Vollständiger Lizenztext"
    },
    "privacy": {
      "title": "Datenschutzerklärung",
      "description": "Wie FrameLy mit Screenshots, Aufnahmen, Transkripten und an KI-Anbieter gesendeten Daten umgeht.",
      "sections": [
        {
          "h": "Kurz gesagt",
          "p": [
            "FrameLy ist eine Desktop-App. Es gibt kein Konto, keine Registrierung und keine Analyse- oder Werbetracker. Deine Screenshots, Aufnahmen und Transkripte werden auf deinem Computer in Ordnern deiner Wahl gespeichert.",
            "Daten verlassen deinen Computer nur, wenn du eine Funktion nutzt, die Internet braucht: das Laden von KI-Modellen oder einen Cloud-KI-Dienst, den du erlaubt hast – FrameLy AI Cloud (Standard für Zusammenfassungen und Themen), Anthropic oder OpenAI."
          ]
        },
        {
          "h": "Screenshots und Aufnahmen",
          "p": [
            "Aufnahmen werden als Dateien in den in den Einstellungen festgelegten Ordnern gespeichert (standardmäßig Bilder und Filme/Videos). FrameLy führt einen lokalen Index für die Mediathek. FrameLy lädt diese Dateien nirgendwohin hoch."
          ]
        },
        {
          "h": "Transkription (immer lokal)",
          "p": [
            "Die Spracherkennung läuft mit heruntergeladenen Modellen auf deinem Computer. Audio und Video werden für die Transkription nie an einen Server gesendet. Transkripte, Zusammenfassungen und Themen liegen in einer lokalen Datenbank auf deinem Computer und lassen sich in der App löschen."
          ]
        },
        {
          "h": "Lokale KI und Apple Intelligence",
          "p": [
            "Mit der lokalen KI von FrameLy entstehen Zusammenfassungen und Themen auf deinem Computer. Mit Apple Intelligence erzeugt sie Apples Modell auf dem Gerät über macOS; FrameLy sendet dafür nichts an Apple-Server. Für Apple Intelligence gelten Apples eigene Richtlinien."
          ]
        },
        {
          "h": "Cloud-KI (optional)",
          "p": [
            "FrameLy AI Cloud ist der eigene Cloud-Dienst von FrameLy und der Standard für Zusammenfassungen und Themen. Erst nach deiner Zustimmung sendet FrameLy ihm den Text des Transkripts und die gewünschte Aufgabe, und ein KI-Partneranbieter erzeugt das Ergebnis. Dein Computer wird über einen dort erzeugten anonymen Schlüssel erkannt – ohne Konto. Der Dienst speichert keine Inhalte, sondern nur die anonymen Schlüssel und die tägliche Nutzungszahl. Der Partneranbieter kann die Inhalte zur Verbesserung seiner Dienste nutzen; die App weist vor der ersten Nutzung darauf hin – für sensible oder vertrauliche Aufnahmen lieber die lokale KI. Der Dienst wird im Europäischen Wirtschaftsraum, in der Schweiz und im Vereinigten Königreich nicht angeboten.",
            "Wenn du externe Anbieter einschaltest, deinen eigenen API-Schlüssel hinzufügst und einen Anbieter erlaubst, sendet FrameLy den Text des Transkripts (nicht Audio oder Video) und die Anweisungen für Zusammenfassung oder Themen direkt an diesen Anbieter – Anthropic oder OpenAI. Es gelten dessen Bedingungen und Datenschutzerklärung, auch zur Speicherdauer. Die Nutzung wird über dein Konto beim Anbieter abgerechnet.",
            "API-Schlüssel und der anonyme Schlüssel von FrameLy AI Cloud werden im macOS-Schlüsselbund oder in der Windows-Anmeldeinformationsverwaltung gespeichert, nicht in normalen Dateien. Die Einstellung „Nur lokal“ blockiert alle diese Verbindungen."
          ]
        },
        {
          "h": "Laden von Modellen",
          "p": [
            "Beim Laden von KI-Modellen verbindet sich dein Computer mit den Servern, die sie bereitstellen (etwa Hugging Face). Diese Server sehen eine normale Download-Anfrage einschließlich deiner IP-Adresse. Jede Datei wird mit einem bekannten SHA-256-Hash geprüft."
          ]
        },
        {
          "h": "Diese Website",
          "p": [
            "Diese Website ist statisch und wird über GitHub Pages bereitgestellt. Sie verwendet keine Cookies und keine Analyse. Deine Wahl von Sprache und Betriebssystem wird nur im lokalen Speicher deines Browsers abgelegt. GitHub kann technische Daten wie IP-Adressen verarbeiten, um Website und Downloads auszuliefern – siehe die Datenschutzerklärung von GitHub."
          ]
        },
        {
          "h": "Deine Kontrolle",
          "p": [
            "Du kannst Aufnahmen, Transkripte und Modelle jederzeit in der App oder im Dateisystem löschen. Beim Deinstallieren von FrameLy wird die App entfernt; Dateien in deinen Aufnahmeordnern bleiben, bis du sie löschst."
          ]
        },
        {
          "h": "Kontakt",
          "p": [
            "Fragen zum Datenschutz: Eröffne ein Issue im FrameLy-Repository auf GitHub."
          ]
        }
      ]
    },
    "terms": {
      "title": "Nutzungsbedingungen",
      "description": "Die Bedingungen für die Nutzung von FrameLy und dieser Website.",
      "sections": [
        {
          "h": "Die Software",
          "p": [
            "FrameLy wird kostenlos unter der MIT-Lizenz bereitgestellt. Du darfst es gemäß dieser Lizenz nutzen, kopieren, ändern und weitergeben."
          ]
        },
        {
          "h": "Keine Gewährleistung",
          "p": [
            "FrameLy wird „wie besehen“ ohne jede Gewährleistung bereitgestellt. Transkripte, Zusammenfassungen und Themen werden automatisch erzeugt und können Fehler enthalten; prüfe wichtige Informationen, bevor du dich darauf verlässt."
          ]
        },
        {
          "h": "Deine Inhalte und deine Verantwortung",
          "p": [
            "Du bist verantwortlich für das, was du aufnimmst und verarbeitest. Stelle sicher, dass du das Recht und, wo nötig, die Einwilligung der Beteiligten hast, Bildschirme, Ton und Gespräche aufzunehmen und Transkripte an KI-Anbieter von Drittanbietern zu senden."
          ]
        },
        {
          "h": "Dienste Dritter",
          "p": [
            "Cloud-KI-Anbieter (Anthropic, OpenAI), Apple Intelligence und Dienste, die die Modelle bereitstellen, werden von Dritten nach eigenen Bedingungen betrieben. Ihre Nutzung mit FrameLy ist optional, und ihre Gebühren betreffen nur dich und sie."
          ]
        },
        {
          "h": "Downloads",
          "p": [
            "Lade FrameLy nur aus dem offiziellen GitHub-Repository oder von dieser Website und prüfe die mit jeder Version veröffentlichte SHA-256-Prüfsumme."
          ]
        },
        {
          "h": "Änderungen",
          "p": [
            "Diese Bedingungen können sich mit der Weiterentwicklung von FrameLy ändern. Das Datum oben zeigt die aktuelle Fassung."
          ]
        }
      ]
    }
  }
};
