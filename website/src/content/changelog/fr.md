## [0.3.0] - 2026-11-16

FrameLy se met désormais à jour tout seul, et l’installeur Windows adopte l’apparence de FrameLy.

### Added

- Mises à jour automatiques : FrameLy consulte les versions publiées sur GitHub en arrière-plan (désactivable dans Réglages › À propos, et jamais en mode « Local uniquement »), affiche un avis discret avec les nouveautés et se met à jour en un clic — progression du téléchargement, vérification de la signature, installation et redémarrage. Un enregistrement en cours n’est jamais interrompu : la mise à jour s’installe à la fin. `framely --update` recherche et installe depuis la ligne de commande.

### Changed

- L’installeur Windows adopte l’apparence de FrameLy, les cinq langues de l’app, des textes propres sur les écrans d’accueil et de fin, et une option de désinstallation qui supprime aussi les clés d’API enregistrées.

### Known limitations

- Les versions 0.1.0 et 0.2.0 n’ont pas le module de mise à jour : téléchargez une fois, depuis le site, la première version qui l’inclut.
- Les versions macOS ne sont pas encore notariées : macOS peut redemander l’autorisation d’Enregistrement de l’écran après une mise à jour.

## [0.2.0] - 2026-11-16

FrameLy AI Cloud est en ligne et devient l’option par défaut pour les résumés et les sujets.

### Added

- FrameLy AI Cloud dans les versions publiées : résumés et sujets dans le cloud, gratuits, sans compte ni clé, avec une limite quotidienne par ordinateur. Un fournisseur d’IA partenaire génère le résultat et peut utiliser le contenu pour améliorer ses services — l’app le signale avant la première utilisation.

### Changed

- Les nouvelles installations et les réglages encore sur l’ancienne option par défaut utilisent FrameLy AI Cloud pour les résumés et les sujets ; tout autre choix est conservé. FrameLy demande votre autorisation avant tout envoi, et le mode « Local uniquement » bloque toujours le cloud.
- La configuration initiale ne télécharge le modèle local de résumé que si FrameLy AI Local génère les résumés ou les sujets.

### Fixed

- Les boîtes de confirmation (comme la demande d’autorisation) s’ouvrent désormais au-dessus de la fenêtre Nouvelle transcription, et Échap ferme d’abord la boîte de dialogue.

### Known limitations

- FrameLy AI Cloud n’est pas disponible dans l’Espace économique européen, en Suisse ni au Royaume-Uni ; FrameLy AI Local, Apple Intelligence et votre propre clé Anthropic ou OpenAI y fonctionnent.

## [0.1.0] - 2026-11-15

Première version publique.

### Added

- Captures de l’écran entier, d’une fenêtre ou d’une zone, avec annotation directement sur la sélection : flèches, formes, texte, surlignage, flou et recadrage, avec annuler et rétablir. Copie dans le presse-papiers ou enregistrement.
- Enregistrement de l’écran entier, d’une fenêtre ou d’une zone, avec le son du système et le micro, pause et reprise, compte à rebours, H.264 ou HEVC et taille de fichier estimée avant de commencer.
- Panneau rapide dans la barre des menus ou la zone de notification, raccourcis globaux pour chaque mode de capture et ligne de commande (`--capture`, `--record`, `--panel`, `--show`, `--edit`, `--transcribe`).
- Bibliothèque des captures et enregistrements, et réglages par catégorie.
- Interface en français, anglais, portugais (Brésil), espagnol et allemand.
- Transcription des enregistrements et des fichiers audio ou vidéo importés sur votre ordinateur (whisper.cpp), avec détection automatique de la langue, minutage de chaque mot, modification et export en TXT, SRT, VTT, Markdown et PDF.
- Résumés et sujets avec FrameLy AI sur votre ordinateur (llama.cpp), Apple Intelligence sous macOS 26 ou ultérieur, ou Anthropic et OpenAI avec votre propre clé d’API, après votre autorisation.
- Gestionnaire de modèles avec téléchargements vérifiés par SHA-256, pause et reprise, et profil suggéré selon la mémoire de l’ordinateur.
- Mode « Local uniquement », qui bloque toute connexion.

### Known limitations

- L’enregistreur Windows a été validé sur du matériel réel pour le plein écran ; l’enregistrement de fenêtre et de zone, la pause, l’audio et l’IA locale demandent encore des tests sur des PC Windows.
- Les installeurs ne sont pas encore signés avec un Developer ID Apple ni avec un certificat de signature de code Windows.
- Les versions ne concernent que les Mac avec puce Apple et Windows 64 bits Intel/AMD.
- Les transcriptions n’identifient pas les intervenants.
- Un enregistrement capture un seul écran à la fois.
