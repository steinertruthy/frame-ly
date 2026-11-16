import type { Messages } from './index';

export const fr: Messages = {
  "meta": {
    "title": "FrameLy — Enregistrement d’écran, captures et transcription sur votre ordinateur",
    "description": "FrameLy réalise des captures et enregistre l’écran sur macOS et Windows, puis transcrit, résume et organise vos enregistrements en sujets avec une IA qui peut fonctionner entièrement sur votre ordinateur.",
    "ogAlt": "Fenêtre de FrameLy affichant un enregistrement avec sa transcription, son résumé et ses sujets"
  },
  "a11y": {
    "skip": "Aller au contenu",
    "menu": "Ouvrir le menu",
    "close": "Fermer le menu",
    "language": "Langue",
    "home": "Accueil FrameLy",
    "screenshotOf": "Capture de FrameLy :",
    "playPause": "Lire ou mettre en pause la vidéo de démonstration"
  },
  "nav": {
    "overview": "Présentation",
    "features": "Fonctionnalités",
    "ai": "Intelligence artificielle",
    "how": "Fonctionnement",
    "download": "Télécharger",
    "cta": "Télécharger"
  },
  "os": {
    "macos": "macOS",
    "windows": "Windows"
  },
  "hero": {
    "eyebrow": "Enregistrement d’écran · Captures · Transcription",
    "title": "Capturez. Enregistrez.\nComprenez.",
    "subtitle": "FrameLy est un enregistreur d’écran et un outil de capture rapide et discret pour macOS et Windows, qui transforme vos enregistrements en transcriptions, résumés et sujets — avec une IA qui peut fonctionner entièrement sur votre ordinateur.",
    "ctaFor": "Télécharger pour {os}",
    "ctaGeneric": "Télécharger FrameLy",
    "ctaSecondary": "Voir le fonctionnement",
    "detected": "Système détecté : {os}",
    "notDetected": "Choisissez votre système ci-dessous",
    "compat": "macOS 14+ sur Apple Silicon · Windows 10 22H2+ (x64)",
    "version": "Version {version}",
    "noRelease": "Installeurs en préparation",
    "free": "Gratuit et open source (MIT)"
  },
  "intro": {
    "eyebrow": "Présentation",
    "title": "Une seule app, de la capture au sens.",
    "body": "FrameLy vit dans la barre des menus ou la zone de notification. Faites une capture et annotez-la directement sur la sélection, enregistrez l’écran avec le son du système et le micro, puis laissez FrameLy transcrire l’enregistrement et en extraire l’essentiel. Vos fichiers restent dans les dossiers de votre choix.",
    "points": [
      {
        "title": "Natif et léger",
        "body": "Conçu en Rust avec les API de capture de chaque système — ScreenCaptureKit sur macOS, Windows.Graphics.Capture sur Windows. Sans FFmpeg ni extension de navigateur."
      },
      {
        "title": "Pensé pour le clavier",
        "body": "Raccourcis globaux pour chaque mode de capture, un panneau rapide et une ligne de commande pour Raycast, Alfred, Raccourcis ou AutoHotkey."
      },
      {
        "title": "L’IA de votre choix",
        "body": "La transcription se fait toujours sur votre ordinateur. Les résumés et les sujets utilisent FrameLy AI Cloud par défaut, ou les modèles locaux de FrameLy, Apple Intelligence ou votre propre clé d’API."
      }
    ]
  },
  "features": {
    "eyebrow": "Fonctionnalités",
    "title": "Cinq outils, un flux de travail serein.",
    "benefit": "Pourquoi c’est utile",
    "items": {
      "screenshot": {
        "kicker": "Captures",
        "title": "Sélectionnez, annotez, partagez — sans quitter la sélection.",
        "body": "Capturez l’écran entier, une fenêtre ou n’importe quelle zone. Flèches, formes, texte, surlignage, flou et recadrage apparaissent directement sur la sélection, avec annuler et rétablir. Copiez dans le presse-papiers ou enregistrez dans votre dossier.",
        "benefit": "Expliquez un bug ou un détail de design en quelques secondes, en floutant les parties sensibles.",
        "alt": "Éditeur de captures de FrameLy avec les outils d’annotation autour d’une zone sélectionnée"
      },
      "recording": {
        "kicker": "Enregistrement d’écran",
        "title": "Enregistrez l’écran, une fenêtre ou une zone — avec le son.",
        "body": "Mélangez le son du système et le micro, mettez en pause et reprenez, et arrêtez depuis une petite barre d’enregistrement ou un raccourci. H.264 ou HEVC, avec compte à rebours et taille de fichier estimée avant de commencer.",
        "benefit": "Tutoriels, démos et comptes rendus de réunion sans monter un studio.",
        "alt": "Écran d’accueil de FrameLy avec les raccourcis de capture et d’enregistrement"
      },
      "transcription": {
        "kicker": "Transcription",
        "title": "Chaque mot, sur votre ordinateur.",
        "body": "FrameLy transcrit les enregistrements et les fichiers audio ou vidéo importés avec des modèles vocaux ouverts qui fonctionnent localement, détecte automatiquement la langue et conserve le minutage de chaque mot. Cliquez sur une ligne pour aller à ce moment, modifiez le texte et exportez en TXT, SRT, VTT, Markdown ou PDF.",
        "benefit": "Des enregistrements consultables et des sous-titres prêts, sans envoyer l’audio.",
        "alt": "Transcription dans FrameLy avec lecteur vidéo et texte horodaté"
      },
      "summaries": {
        "kicker": "Résumés",
        "title": "La réunion en un paragraphe.",
        "body": "Générez un résumé bref, équilibré ou détaillé à partir de la transcription — avec FrameLy AI Cloud (par défaut), les modèles locaux de FrameLy, Apple Intelligence sur les Mac compatibles, ou Anthropic ou OpenAI avec votre propre clé.",
        "benefit": "Partagez les décisions et les prochaines étapes plutôt qu’une vidéo de 40 minutes.",
        "alt": "Onglet résumé de FrameLy avec les points clés et les prochaines étapes"
      },
      "topics": {
        "kicker": "Sujets",
        "title": "Une carte de ce qui a été dit.",
        "body": "FrameLy organise la transcription en sujets et sous-sujets, chacun avec son moment dans l’enregistrement. Ouvrez un sujet pour lire ses points, ou cliquez sur son horodatage pour écouter ce passage.",
        "benefit": "Retrouvez d’un coup d’œil le passage utile dans un long enregistrement.",
        "alt": "Onglet sujets de FrameLy avec des sujets et sous-sujets horodatés"
      }
    }
  },
  "ai": {
    "eyebrow": "Intelligence artificielle",
    "title": "Trois façons de réfléchir. Vous choisissez où.",
    "subtitle": "La transcription a toujours lieu sur votre ordinateur. Pour les résumés et les sujets, choisissez le moteur adapté à l’enregistrement — et changez quand vous voulez dans les Réglages.",
    "modes": {
      "local": {
        "name": "FrameLy AI Local",
        "label": "Sur votre ordinateur",
        "tag": "Sur cet ordinateur · fonctionne hors ligne",
        "body": "Des modèles ouverts téléchargés par l’app et exécutés sur votre ordinateur. Sans compte, sans clé et sans coût à l’usage. Une fois les modèles téléchargés, transcription, résumés et sujets fonctionnent sans internet.",
        "points": [
          "Voix : 3 niveaux de précision (de 181 Mo à 1 Go)",
          "Langage : 3 profils (de 2,6 à 5,3 Go), suggérés selon votre RAM",
          "Téléchargements vérifiés par SHA-256 ; pause, reprise ou suppression à tout moment",
          "Le mode « Local uniquement » bloque toute connexion"
        ]
      },
      "cloud": {
        "name": "Dans le cloud",
        "label": "FrameLy AI Cloud · Anthropic · OpenAI",
        "tag": "Par défaut pour les résumés et les sujets",
        "body": "Résumés et sujets générés sur des serveurs plutôt que sur votre ordinateur. Seul le texte de la transcription est envoyé — jamais l’audio ni la vidéo — et seulement après votre autorisation.",
        "options": [
          {
            "name": "FrameLy AI Cloud",
            "status": "Disponible · gratuit",
            "body": "Le service cloud de FrameLy et l’option par défaut pour les résumés et les sujets : gratuit, sans compte ni clé, avec une limite quotidienne par ordinateur. Un fournisseur d’IA partenaire génère le résultat et peut utiliser le contenu pour améliorer ses services — préférez l’IA locale pour les enregistrements sensibles. Non disponible dans l’EEE, en Suisse ni au Royaume-Uni."
          },
          {
            "name": "Anthropic (Claude) ou OpenAI",
            "status": "Disponible · payant, avec votre clé",
            "body": "Utilisez votre propre clé d’API. Désactivé par défaut : activez les fournisseurs externes, ajoutez la clé et autorisez chaque fournisseur. L’utilisation est facturée sur votre compte chez eux."
          }
        ],
        "points": [
          "Clés stockées dans le trousseau macOS ou le Gestionnaire d’identification Windows",
          "Le mode « Local uniquement » bloque toutes les options cloud"
        ]
      },
      "apple": {
        "name": "Apple Intelligence",
        "label": "Sur votre Mac",
        "tag": "macOS 26 ou version ultérieure",
        "body": "Sur un Mac doté d’Apple Intelligence, FrameLy peut rédiger résumés et sujets avec le modèle d’Apple qui fonctionne sur l’appareil. Rien à télécharger dans FrameLy, aucune clé.",
        "points": [
          "Nécessite macOS 26 ou version ultérieure",
          "Un Mac compatible avec Apple Intelligence, activé dans les Réglages Système",
          "Dans les langues prises en charge par Apple Intelligence",
          "Non disponible sur Windows"
        ]
      }
    },
    "table": {
      "caption": "Comparez les options d’IA pour les résumés et les sujets",
      "feature": "Option",
      "rows": {
        "availability": "Disponibilité",
        "where": "Où a lieu le traitement",
        "connection": "Internet",
        "platforms": "Plateformes",
        "hardware": "Matériel",
        "cost": "Coût",
        "setup": "Configuration"
      },
      "columns": {
        "local": {
          "name": "FrameLy AI Local",
          "availability": "Disponible",
          "where": "Sur votre ordinateur",
          "connection": "Seulement pour télécharger les modèles",
          "platforms": "macOS et Windows",
          "hardware": "Apple Silicon, ou processeur x64 avec AVX2 ; environ 3 à 6 Go de mémoire libre selon le profil",
          "cost": "Gratuit",
          "setup": "Télécharger les modèles dans l’app (3 à 6,5 Go au total)"
        },
        "cloud": {
          "name": "FrameLy AI Cloud",
          "availability": "Disponible, sauf dans l’EEE, en Suisse et au Royaume-Uni",
          "where": "Service FrameLy et un fournisseur d’IA partenaire",
          "connection": "Obligatoire",
          "platforms": "macOS et Windows",
          "hardware": "Tout ordinateur qui exécute FrameLy",
          "cost": "Gratuit, avec une limite quotidienne",
          "setup": "Par défaut ; autoriser à la première utilisation, sans clé"
        },
        "external": {
          "name": "Anthropic / OpenAI",
          "availability": "Disponible",
          "where": "Serveurs d’Anthropic ou d’OpenAI",
          "connection": "Obligatoire",
          "platforms": "macOS et Windows",
          "hardware": "Tout ordinateur qui exécute FrameLy",
          "cost": "Payant à l’usage, auprès du fournisseur",
          "setup": "Activer, ajouter votre clé d’API, autoriser le fournisseur"
        },
        "apple": {
          "name": "Apple Intelligence",
          "availability": "macOS 26 ou ultérieur",
          "where": "Sur votre Mac",
          "connection": "Pas nécessaire",
          "platforms": "macOS 26 ou ultérieur",
          "hardware": "Mac compatible avec Apple Intelligence",
          "cost": "Gratuit",
          "setup": "Activer Apple Intelligence dans les Réglages Système"
        }
      },
      "note": "La transcription elle-même utilise toujours les modèles vocaux locaux de FrameLy, quel que soit le mode choisi pour les résumés et les sujets."
    },
    "figure": "Le gestionnaire de modèles dans les Réglages de FrameLy : installez, vérifiez ou supprimez chaque modèle. Tailles et licences proviennent de l’app.",
    "figureAlt": "Gestionnaire de modèles de FrameLy listant les modèles de transcription et de langage avec taille, mémoire et licence"
  },
  "how": {
    "eyebrow": "Fonctionnement",
    "title": "Du premier raccourci au résumé.",
    "steps": [
      {
        "title": "Installez",
        "body": "Téléchargez l’installeur de votre système et ouvrez FrameLy. Il vit dans la barre des menus ou la zone de notification."
      },
      {
        "title": "Autorisez l’accès",
        "body": "Autorisez l’enregistrement de l’écran et, si vous le souhaitez, le micro. FrameLy explique à quoi sert chaque autorisation."
      },
      {
        "title": "Capturez ou enregistrez",
        "body": "Utilisez un raccourci ou le panneau rapide pour faire une capture ou enregistrer l’écran, une fenêtre ou une zone."
      },
      {
        "title": "Choisissez l’IA",
        "body": "Les résumés et les sujets utilisent FrameLy AI Cloud après votre autorisation. Vous préférez votre ordinateur ? Téléchargez les modèles locaux en une étape, ou choisissez Apple Intelligence ou votre clé."
      },
      {
        "title": "Transcrivez",
        "body": "Envoyez un enregistrement ou n’importe quel fichier audio ou vidéo dans Transcriptions. Les longs fichiers reprennent là où ils s’étaient arrêtés."
      },
      {
        "title": "Résumez et partagez",
        "body": "Générez le résumé et les sujets, modifiez-les et exportez la transcription en TXT, SRT, VTT, Markdown ou PDF."
      }
    ]
  },
  "uses": {
    "eyebrow": "Cas d’usage",
    "title": "Conçu pour le travail au quotidien.",
    "items": [
      {
        "title": "Signaler des bugs",
        "body": "Une capture avec une flèche et une adresse e-mail floutée, ou un court enregistrement des étapes."
      },
      {
        "title": "Comptes rendus de réunion",
        "body": "Enregistrez un appel avec le son du système et le micro, puis obtenez le résumé et les prochaines étapes."
      },
      {
        "title": "Tutoriels et démos",
        "body": "Enregistrez une démonstration et exportez les sous-titres en SRT ou VTT."
      },
      {
        "title": "Cours et conférences",
        "body": "Transformez un long enregistrement en sujets entre lesquels naviguer."
      },
      {
        "title": "Revue de design",
        "body": "Pointez les détails avec des formes et du texte, directement sur la capture."
      },
      {
        "title": "Contenu sensible",
        "body": "Gardez tout sur votre ordinateur avec l’IA locale et le mode « Local uniquement »."
      }
    ]
  },
  "demo": {
    "eyebrow": "En action",
    "title": "Un vrai enregistrement, transcrit et résumé par FrameLy.",
    "caption": "Capturé depuis FrameLy sur macOS. L’audio de la réunion est une voix de synthèse."
  },
  "download": {
    "eyebrow": "Télécharger",
    "title": "Obtenez FrameLy.",
    "subtitle": "Gratuit pour macOS et Windows. Choisissez votre système — votre choix est mémorisé.",
    "choose": "Système d’exploitation",
    "detectedHint": "Nous avons détecté {os}.",
    "notDetectedHint": "Nous n’avons pas pu identifier votre système. Choisissez-en un :",
    "button": "Télécharger pour {os}",
    "version": "Version",
    "released": "Publication",
    "arch": "Architecture",
    "format": "Format",
    "size": "Taille",
    "checksum": "SHA-256",
    "checksums": "Toutes les sommes de contrôle",
    "requirements": "Configuration requise",
    "install": "Installation",
    "previous": "Versions précédentes",
    "noPrevious": "Pas encore de versions précédentes.",
    "changelog": "Nouveautés",
    "allReleases": "Toutes les versions sur GitHub",
    "portable": "Portable (zip)",
    "none": "Aucun installeur n’est encore publié pour {os}. Suivez le dépôt pour être averti de sa sortie.",
    "unavailableArch": "Pas encore disponible",
    "archNames": {
      "arm64": "Apple Silicon (arm64)",
      "x64": "x64 (Intel/AMD 64 bits)",
      "universal": "Universel"
    },
    "formats": {
      "dmg": "Image disque (.dmg)",
      "exe": "Installeur (.exe, NSIS)",
      "msi": "Installeur (.msi)",
      "zip": "Portable (.zip)"
    },
    "req": {
      "macos": [
        "macOS 14 Sonoma ou version ultérieure",
        "Mac avec puce Apple (M1 ou ultérieure)",
        "Mac Intel : pas encore de version",
        "Fonctions Apple Intelligence : macOS 26 ou ultérieur"
      ],
      "windows": [
        "Windows 10 22H2 ou Windows 11",
        "x64 (Intel ou AMD 64 bits) — ARM64 : pas encore de version",
        "Microsoft Edge WebView2 (inclus dans Windows 11 ; l’installeur peut le configurer)",
        "IA locale : processeur avec AVX2"
      ]
    },
    "steps": {
      "macos": [
        "Ouvrez le .dmg et faites glisser FrameLy dans Applications.",
        "Cette version n’est pas encore notariée par Apple. La première fois, ouvrez FrameLy, puis allez dans Réglages Système › Confidentialité et sécurité et cliquez sur « Ouvrir quand même ».",
        "Autorisez l’Enregistrement de l’écran (et le Micro, si vous l’utilisez) quand FrameLy le demande."
      ],
      "windows": [
        "Lancez l’installeur -setup.exe. Il s’installe pour votre utilisateur uniquement — sans droits d’administrateur.",
        "L’installeur n’est pas encore signé, donc Windows SmartScreen peut afficher un avertissement. Cliquez sur « Informations complémentaires », puis sur « Exécuter quand même ».",
        "FrameLy démarre dans la zone de notification. Utilisez le panneau rapide ou Ctrl+Alt+Maj+Espace."
      ]
    },
    "verify": "Comparez le SHA-256 du fichier avec la valeur ci-dessus : shasum -a 256 <fichier> sur macOS, ou Get-FileHash <fichier> dans PowerShell.",
    "windowsNote": "Windows : l’enregistrement plein écran a été testé sur du matériel réel ; l’enregistrement de fenêtre et de zone, la pause, l’audio et l’IA locale sont encore en cours de vérification sur d’autres PC Windows."
  },
  "faq": {
    "eyebrow": "Questions fréquentes",
    "title": "Vos questions, nos réponses.",
    "more": "Toutes les questions",
    "items": [
      {
        "q": "FrameLy est-il gratuit ?",
        "a": "Oui. FrameLy est gratuit et open source sous licence MIT. FrameLy AI Cloud est gratuit avec une limite quotidienne, et l’IA locale et Apple Intelligence sont gratuites aussi ; les fournisseurs cloud que vous connectez avec votre clé vous facturent directement."
      },
      {
        "q": "Mon audio ou ma vidéo quitte-t-il mon ordinateur ?",
        "a": "Pas pour la transcription : elle se fait toujours sur votre ordinateur. Les résumés et les sujets utilisent FrameLy AI Cloud par défaut : le texte de la transcription — jamais l’audio ni la vidéo — lui est envoyé après votre autorisation. Choisissez FrameLy AI Local ou Apple Intelligence pour tout garder sur votre ordinateur, ou activez « Local uniquement »."
      },
      {
        "q": "Qu’est-ce que FrameLy AI Cloud ?",
        "a": "Le service cloud gratuit de FrameLy pour les résumés et les sujets, et l’option par défaut. Ni compte ni clé ; chaque ordinateur a une limite quotidienne. Le texte de la transcription lui est envoyé et un fournisseur d’IA partenaire génère le résultat ; ce fournisseur peut utiliser le contenu pour améliorer ses services, c’est pourquoi l’app demande votre autorisation avant la première utilisation et vous pouvez passer à l’IA locale à tout moment. Il n’est pas disponible dans l’EEE, en Suisse ni au Royaume-Uni."
      },
      {
        "q": "FrameLy fonctionne-t-il hors ligne ?",
        "a": "Les captures, l’enregistrement et la transcription fonctionnent sans internet une fois le modèle vocal téléchargé. Pour des résumés et sujets hors ligne, choisissez FrameLy AI Local (après avoir téléchargé son modèle) ou Apple Intelligence ; FrameLy AI Cloud et les autres fournisseurs cloud nécessitent une connexion."
      },
      {
        "q": "Quel espace disque occupent les modèles d’IA ?",
        "a": "Le modèle vocal occupe de 181 Mo à 1 Go selon la précision, et le modèle de langage de 2,6 à 5,3 Go selon le profil. FrameLy suggère un profil d’après la mémoire de votre ordinateur, et vous pouvez supprimer les modèles à tout moment."
      },
      {
        "q": "Quels Mac prennent en charge Apple Intelligence dans FrameLy ?",
        "a": "Les Mac compatibles avec Apple Intelligence sous macOS 26 ou ultérieur, avec Apple Intelligence activé dans les Réglages Système. FrameLy lui-même fonctionne sur macOS 14 ou ultérieur."
      },
      {
        "q": "FrameLy fonctionne-t-il sur les Mac Intel ou Windows ARM ?",
        "a": "Pas encore. Les versions actuelles sont destinées aux Mac avec puce Apple et aux PC Windows 64 bits Intel/AMD."
      },
      {
        "q": "Distingue-t-il les intervenants ?",
        "a": "Pas encore. La reconnaissance vocale locale de FrameLy n’identifie pas les intervenants."
      },
      {
        "q": "Pourquoi macOS ou Windows affichent-ils un avertissement à l’installation ?",
        "a": "Les installeurs ne sont pas encore signés avec un Developer ID Apple ni avec un certificat de signature de code Windows. Vous pouvez vérifier chaque fichier grâce à sa somme SHA-256 dans la section de téléchargement et sur GitHub."
      },
      {
        "q": "Comment mettre FrameLy à jour ?",
        "a": "Depuis la version 0.3.0, FrameLy recherche les nouvelles versions en arrière-plan et propose la mise à jour en un clic, jamais pendant un enregistrement ; vous pouvez aussi vérifier dans Réglages › À propos ou désactiver cette option. Les versions précédentes (0.1.0 et 0.2.0) ne se mettent pas à jour seules : téléchargez une fois la dernière version depuis cette page. Sur macOS, le système peut redemander l’autorisation d’Enregistrement de l’écran après une mise à jour, car l’app n’est pas encore notariée."
      },
      {
        "q": "En quelles langues l’app est-elle disponible ?",
        "a": "L’interface est disponible en français, anglais, portugais (Brésil), espagnol et allemand. La transcription détecte automatiquement la langue parlée."
      }
    ]
  },
  "footer": {
    "tagline": "Enregistrement d’écran, captures et transcription sur votre ordinateur, pour macOS et Windows.",
    "product": "Produit",
    "resources": "Ressources",
    "legal": "Mentions",
    "github": "Code source",
    "releases": "Versions",
    "issues": "Signaler un problème",
    "changelog": "Nouveautés",
    "faq": "Questions fréquentes",
    "privacy": "Confidentialité",
    "terms": "Conditions d’utilisation",
    "license": "Licence",
    "rights": "© {year} FrameLy. Publié sous licence MIT.",
    "trademarks": "Apple, macOS et Apple Intelligence sont des marques d’Apple Inc. Windows est une marque de Microsoft. Claude et Anthropic sont des marques d’Anthropic. OpenAI est une marque d’OpenAI."
  },
  "pages": {
    "pending": "Brouillon — en attente de relecture juridique. Ce texte décrit le fonctionnement actuel de FrameLy ; il ne constitue pas un conseil juridique.",
    "updated": "Dernière mise à jour : {date}",
    "back": "Retour à l’accueil",
    "toc": "Sur cette page",
    "notFound": {
      "title": "Page introuvable",
      "body": "La page que vous cherchez n’existe pas ou a été déplacée."
    },
    "changelog": {
      "title": "Nouveautés",
      "description": "Toutes les versions de FrameLy, avec les nouveautés, améliorations et corrections.",
      "intro": "Toutes les modifications notables de FrameLy. Les versions suivent le versionnage sémantique.",
      "added": "Nouveautés",
      "changed": "Améliorations",
      "fixed": "Corrections",
      "known": "Limites connues",
      "assets": "Téléchargements",
      "release": "Version sur GitHub",
      "empty": "Aucune version publiée pour l’instant."
    },
    "faq": {
      "title": "Questions fréquentes",
      "description": "Réponses sur FrameLy : prix, confidentialité, modèles d’IA, compatibilité et installation."
    },
    "license": {
      "title": "Licence",
      "description": "FrameLy est open source sous licence MIT.",
      "intro": "Le code source de FrameLy est publié sous licence MIT. Les modèles d’IA téléchargés par FrameLy ont leurs propres licences, indiquées ci-dessous.",
      "models": "Modèles téléchargés par FrameLy",
      "modelsBody": "La reconnaissance vocale utilise les modèles Whisper (MIT) via whisper.cpp. Les résumés et sujets avec l’IA locale utilisent des modèles de langage ouverts des familles Gemma et Qwen, distribués sous licence Apache 2.0. La licence de chaque modèle est aussi affichée dans le gestionnaire de modèles de l’app.",
      "thirdParty": "Logiciels tiers",
      "thirdPartyBody": "FrameLy est construit avec Tauri, Rust, Svelte, whisper.cpp et llama.cpp, entre autres projets open source, chacun sous sa propre licence.",
      "full": "Texte complet de la licence"
    },
    "privacy": {
      "title": "Politique de confidentialité",
      "description": "Comment FrameLy traite les captures, enregistrements, transcriptions et les données envoyées aux fournisseurs d’IA.",
      "sections": [
        {
          "h": "En bref",
          "p": [
            "FrameLy est une application de bureau. Pas de compte, pas d’inscription, aucun traceur d’analyse ou de publicité. Vos captures, enregistrements et transcriptions sont stockés sur votre ordinateur, dans les dossiers de votre choix.",
            "Les données ne quittent votre ordinateur que lorsque vous utilisez une fonction qui nécessite internet : le téléchargement de modèles d’IA, ou un service d’IA cloud que vous avez autorisé — FrameLy AI Cloud (par défaut pour les résumés et les sujets), Anthropic ou OpenAI."
          ]
        },
        {
          "h": "Captures et enregistrements",
          "p": [
            "Les captures sont enregistrées comme fichiers dans les dossiers définis dans les Réglages (par défaut, Images et Vidéos). FrameLy en conserve un index local pour la bibliothèque. FrameLy ne les envoie nulle part."
          ]
        },
        {
          "h": "Transcription (toujours locale)",
          "p": [
            "La reconnaissance vocale s’exécute sur votre ordinateur avec des modèles téléchargés. L’audio et la vidéo ne sont jamais envoyés à un serveur pour être transcrits. Les transcriptions, résumés et sujets sont stockés dans une base de données locale sur votre ordinateur, et vous pouvez les supprimer dans l’app."
          ]
        },
        {
          "h": "IA locale et Apple Intelligence",
          "p": [
            "Avec l’IA locale de FrameLy, résumés et sujets sont générés sur votre ordinateur. Avec Apple Intelligence, ils sont générés par le modèle d’Apple sur l’appareil, via macOS ; FrameLy n’envoie rien aux serveurs d’Apple pour cela. Les politiques d’Apple s’appliquent à Apple Intelligence."
          ]
        },
        {
          "h": "IA dans le cloud (facultatif)",
          "p": [
            "FrameLy AI Cloud est le service cloud de FrameLy et l’option par défaut pour les résumés et les sujets. Seulement après votre autorisation, FrameLy lui envoie le texte de la transcription et la tâche demandée, et un fournisseur d’IA partenaire génère le résultat. Votre ordinateur est identifié par une clé anonyme créée sur celui-ci — sans compte. Le service ne conserve pas le contenu : il stocke les clés anonymes et les compteurs d’utilisation quotidienne. Le fournisseur partenaire peut utiliser le contenu pour améliorer ses services ; l’app le signale avant la première utilisation, préférez donc l’IA locale pour les enregistrements sensibles ou confidentiels. Le service n’est pas proposé dans l’Espace économique européen, en Suisse ni au Royaume-Uni.",
            "Si vous activez les fournisseurs externes, ajoutez votre propre clé d’API et autorisez un fournisseur, FrameLy envoie le texte de la transcription (ni l’audio ni la vidéo) et les instructions du résumé ou des sujets directement à ce fournisseur — Anthropic ou OpenAI. Les conditions et la politique de confidentialité du fournisseur s’appliquent, y compris la durée de conservation des données. L’utilisation est facturée sur votre compte chez lui.",
            "Les clés d’API et la clé anonyme de FrameLy AI Cloud sont stockées dans le trousseau macOS ou le Gestionnaire d’identification Windows, pas dans des fichiers ordinaires. Le réglage « Local uniquement » bloque toutes ces connexions."
          ]
        },
        {
          "h": "Téléchargement des modèles",
          "p": [
            "Lorsque vous téléchargez des modèles d’IA, votre ordinateur se connecte aux serveurs qui les hébergent (comme Hugging Face). Ces serveurs voient une requête de téléchargement ordinaire, y compris votre adresse IP. Chaque fichier est vérifié avec une empreinte SHA-256 connue."
          ]
        },
        {
          "h": "Ce site",
          "p": [
            "Ce site est statique et hébergé sur GitHub Pages. Il n’utilise ni cookies ni outils d’analyse. Vos choix de langue et de système sont enregistrés uniquement dans le stockage local de votre navigateur. GitHub peut traiter des données techniques, comme les adresses IP, pour servir le site et les téléchargements — voir la déclaration de confidentialité de GitHub."
          ]
        },
        {
          "h": "Vous gardez le contrôle",
          "p": [
            "Vous pouvez supprimer captures, transcriptions et modèles depuis l’app ou votre système de fichiers à tout moment. Désinstaller FrameLy supprime l’app ; les fichiers de vos dossiers de capture restent jusqu’à ce que vous les supprimiez."
          ]
        },
        {
          "h": "Contact",
          "p": [
            "Questions sur la confidentialité : ouvrez une issue dans le dépôt FrameLy sur GitHub."
          ]
        }
      ]
    },
    "terms": {
      "title": "Conditions d’utilisation",
      "description": "Les conditions d’utilisation de FrameLy et de ce site.",
      "sections": [
        {
          "h": "Le logiciel",
          "p": [
            "FrameLy est fourni gratuitement sous licence MIT. Vous pouvez l’utiliser, le copier, le modifier et le distribuer selon les termes de cette licence."
          ]
        },
        {
          "h": "Absence de garantie",
          "p": [
            "FrameLy est fourni « en l’état », sans garantie d’aucune sorte. Les transcriptions, résumés et sujets sont générés automatiquement et peuvent contenir des erreurs ; vérifiez les informations importantes avant de vous y fier."
          ]
        },
        {
          "h": "Votre contenu et votre responsabilité",
          "p": [
            "Vous êtes responsable de ce que vous capturez, enregistrez et traitez. Assurez-vous d’avoir le droit et, lorsque c’est nécessaire, le consentement des personnes concernées pour enregistrer des écrans, de l’audio et des conversations, et pour envoyer des transcriptions à des fournisseurs d’IA tiers."
          ]
        },
        {
          "h": "Services tiers",
          "p": [
            "Les fournisseurs d’IA cloud (Anthropic, OpenAI), Apple Intelligence et les services qui hébergent les modèles sont exploités par des tiers selon leurs propres conditions. Leur utilisation avec FrameLy est facultative, et leurs frais vous concernent directement avec eux."
          ]
        },
        {
          "h": "Téléchargements",
          "p": [
            "Téléchargez FrameLy uniquement depuis le dépôt GitHub officiel ou ce site, et vérifiez la somme SHA-256 publiée avec chaque version."
          ]
        },
        {
          "h": "Modifications",
          "p": [
            "Ces conditions peuvent évoluer avec FrameLy. La date en haut de page indique la version la plus récente."
          ]
        }
      ]
    }
  }
};
