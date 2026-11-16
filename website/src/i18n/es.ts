import type { Messages } from './index';

export const es: Messages = {
  meta: {
    title: 'FrameLy — Grabación de pantalla, capturas y transcripción en tu ordenador',
    description:
      'FrameLy hace capturas y graba la pantalla en macOS y Windows, y luego transcribe, resume y organiza las grabaciones en temas con una IA que puede ejecutarse por completo en tu ordenador.',
    ogAlt: 'Ventana de FrameLy con una grabación, su transcripción, resumen y temas',
  },
  a11y: {
    skip: 'Saltar al contenido',
    menu: 'Abrir menú',
    close: 'Cerrar menú',
    language: 'Idioma',
    home: 'Inicio de FrameLy',
    screenshotOf: 'Captura de FrameLy:',
    playPause: 'Reproducir o pausar el vídeo de demostración',
  },
  nav: {
    overview: 'Visión general',
    features: 'Funciones',
    ai: 'Inteligencia artificial',
    how: 'Cómo funciona',
    download: 'Descargar',
    cta: 'Descargar',
  },
  os: { macos: 'macOS', windows: 'Windows' },
  hero: {
    eyebrow: 'Grabación de pantalla · Capturas · Transcripción',
    title: 'Captura. Graba.\nEntiende.',
    subtitle:
      'FrameLy es un grabador de pantalla y una herramienta de capturas rápida y discreta para macOS y Windows que convierte tus grabaciones en transcripciones, resúmenes y temas, con una IA que puede ejecutarse por completo en tu ordenador.',
    ctaFor: 'Descargar para {os}',
    ctaGeneric: 'Descargar FrameLy',
    ctaSecondary: 'Mira cómo funciona',
    detected: 'Sistema detectado: {os}',
    notDetected: 'Elige tu sistema más abajo',
    compat: 'macOS 14+ con Apple Silicon · Windows 10 22H2+ (x64)',
    version: 'Versión {version}',
    noRelease: 'Instaladores en preparación',
    free: 'Gratis y de código abierto (MIT)',
  },
  intro: {
    eyebrow: 'Visión general',
    title: 'Una app de la captura al significado.',
    body:
      'FrameLy vive en la barra de menús o en la bandeja del sistema. Haz una captura y anótala sobre la propia selección, graba la pantalla con el audio del sistema y el micrófono, y deja que FrameLy transcriba la grabación y saque lo importante. Tus archivos se quedan en las carpetas que elijas.',
    points: [
      {
        title: 'Nativo y ligero',
        body: 'Hecho con Rust y las API de captura de cada sistema: ScreenCaptureKit en macOS y Windows.Graphics.Capture en Windows. Sin FFmpeg ni extensiones del navegador.',
      },
      {
        title: 'Pensado para el teclado',
        body: 'Atajos globales para cada modo de captura, un panel rápido y línea de comandos para Raycast, Alfred, Atajos o AutoHotkey.',
      },
      {
        title: 'La IA que prefieras',
        body: 'La transcripción siempre se hace en tu ordenador. Los resúmenes y temas pueden usar modelos locales, Apple Intelligence o tu propia clave de API en la nube.',
      },
    ],
  },
  features: {
    eyebrow: 'Funciones',
    title: 'Cinco herramientas, un flujo tranquilo.',
    benefit: 'Por qué ayuda',
    items: {
      screenshot: {
        kicker: 'Capturas',
        title: 'Selecciona, anota y comparte sin salir de la selección.',
        body: 'Captura la pantalla completa, una ventana o cualquier región. Flechas, formas, texto, resaltado, desenfoque y recorte aparecen sobre la selección, con deshacer y rehacer. Copia al portapapeles o guarda en tu carpeta.',
        benefit: 'Explica un error o un detalle de diseño en segundos, con las partes sensibles desenfocadas.',
        alt: 'Editor de capturas de FrameLy con herramientas de anotación alrededor de una región',
      },
      recording: {
        kicker: 'Grabación de pantalla',
        title: 'Graba la pantalla, una ventana o una región, con sonido.',
        body: 'Mezcla el audio del sistema y el micrófono, pausa y reanuda, y detén desde una pequeña barra de grabación o con un atajo. H.264 o HEVC, con cuenta atrás y tamaño estimado del archivo antes de empezar.',
        benefit: 'Tutoriales, demos y notas de reuniones sin montar un estudio.',
        alt: 'Pantalla de inicio de FrameLy con atajos de captura y grabación',
      },
      transcription: {
        kicker: 'Transcripción',
        title: 'Cada palabra, en tu ordenador.',
        body: 'FrameLy transcribe grabaciones y audios o vídeos importados con modelos abiertos de voz que se ejecutan localmente, detecta el idioma automáticamente y guarda el tiempo de cada palabra. Haz clic en cualquier línea para ir a ese momento, edita el texto y exporta en TXT, SRT, VTT, Markdown o PDF.',
        benefit: 'Grabaciones con búsqueda y subtítulos listos, sin subir el audio.',
        alt: 'Transcripción en FrameLy con reproductor de vídeo y texto con marcas de tiempo',
      },
      summaries: {
        kicker: 'Resúmenes',
        title: 'La reunión en un párrafo.',
        body: 'Genera un resumen breve, equilibrado o detallado a partir de la transcripción: con los modelos locales de FrameLy, con Apple Intelligence en Macs compatibles, o con Anthropic u OpenAI usando tu propia clave.',
        benefit: 'Comparte las decisiones y los próximos pasos en lugar de un vídeo de 40 minutos.',
        alt: 'Pestaña de resumen de FrameLy con puntos clave y próximos pasos',
      },
      topics: {
        kicker: 'Temas',
        title: 'Un mapa de lo que se dijo.',
        body: 'FrameLy organiza la transcripción en temas y subtemas, cada uno con su momento en la grabación. Abre un tema para leer sus puntos o haz clic en su tiempo para reproducir esa parte.',
        benefit: 'Encuentra de un vistazo la parte que necesitas en una grabación larga.',
        alt: 'Pestaña de temas de FrameLy con temas y subtemas marcados en el tiempo',
      },
    },
  },
  ai: {
    eyebrow: 'Inteligencia artificial',
    title: 'Tres formas de pensar. Tú decides dónde.',
    subtitle:
      'La transcripción siempre ocurre en tu ordenador. Para resúmenes y temas, elige el motor que mejor encaje con la grabación y cámbialo cuando quieras en Ajustes.',
    modes: {
      local: {
        name: 'FrameLy AI Local',
        label: 'En tu ordenador',
        tag: 'Predeterminada · en este ordenador',
        body: 'Modelos abiertos que la app descarga y ejecuta en tu ordenador. Sin cuenta, sin clave y sin coste por uso. Una vez descargados los modelos, la transcripción, los resúmenes y los temas funcionan sin internet.',
        points: [
          'Voz: 3 niveles de precisión (de 181 MB a 1 GB)',
          'Lenguaje: 3 perfiles (de 2,6 a 5,3 GB), sugeridos según tu RAM',
          'Descargas verificadas con SHA-256; pausa, reanuda o elimina cuando quieras',
          'El modo «Solo local» bloquea cualquier conexión',
        ],
      },
      cloud: {
        name: 'En la nube',
        label: 'FrameLy AI Cloud · Anthropic · OpenAI',
        tag: 'Opcional · requiere internet',
        body: 'Resúmenes y temas generados en servidores en lugar de en tu ordenador. Solo se envía el texto de la transcripción —nunca el audio ni el vídeo— y solo después de que lo autorices.',
        options: [
          { name: 'FrameLy AI Cloud', status: 'Aún no disponible en esta versión', body: 'El servicio en la nube del propio FrameLy: gratis, sin cuenta ni clave, con un límite diario por ordenador. Un proveedor de IA asociado genera el resultado. Ya está en la app, pero el servicio aún no está en marcha, por eso las versiones publicadas lo muestran como no disponible.' },
          { name: 'Anthropic (Claude) u OpenAI', status: 'Disponible · de pago, con tu clave', body: 'Usa tu propia clave de API. Viene desactivado: activa los proveedores externos, añade la clave y autoriza cada proveedor. El uso se factura a tu cuenta con ellos.' },
        ],
        points: [
          'Claves guardadas en el Llavero de macOS o en el Administrador de credenciales de Windows',
          'El modo «Solo local» bloquea todas las opciones en la nube',
        ],
      },
      apple: {
        name: 'Apple Intelligence',
        label: 'En tu Mac',
        tag: 'macOS 26 o posterior',
        body: 'En un Mac con Apple Intelligence, FrameLy puede escribir resúmenes y temas con el modelo de Apple que funciona en el propio dispositivo. Nada que descargar en FrameLy y sin clave.',
        points: [
          'Requiere macOS 26 o posterior',
          'Un Mac compatible con Apple Intelligence, activado en Ajustes del Sistema',
          'En los idiomas que admite Apple Intelligence',
          'No disponible en Windows',
        ],
      },
    },
    table: {
      caption: 'Compara las opciones de IA para resúmenes y temas',
      feature: 'Opción',
      rows: {
          availability: 'Disponibilidad',
          where: 'Dónde se procesa',
          connection: 'Internet',
          platforms: 'Plataformas',
          hardware: 'Hardware',
          cost: 'Coste',
          setup: 'Configuración',
      },
      columns: {
        local: {
          name: 'FrameLy AI Local',
          availability: 'Disponible',
          where: 'En tu ordenador',
          connection: 'Solo para descargar los modelos',
          platforms: 'macOS y Windows',
          hardware: 'Apple Silicon o CPU x64 con AVX2; entre 3 y 6 GB de memoria libre, según el perfil',
          cost: 'Gratis',
          setup: 'Descargar los modelos en la app (de 3 a 6,5 GB en total)',
        },
        cloud: {
          name: 'FrameLy AI Cloud',
          availability: 'Aún no en las versiones publicadas',
          where: 'Servicio de FrameLy y un proveedor de IA asociado',
          connection: 'Obligatoria',
          platforms: 'macOS y Windows',
          hardware: 'Cualquier ordenador que ejecute FrameLy',
          cost: 'Gratis, con límite diario',
          setup: 'Autorizar en el primer uso; sin clave',
        },
        external: {
          name: 'Anthropic / OpenAI',
          availability: 'Disponible',
          where: 'Servidores de Anthropic u OpenAI',
          connection: 'Obligatoria',
          platforms: 'macOS y Windows',
          hardware: 'Cualquier ordenador que ejecute FrameLy',
          cost: 'De pago por uso, al proveedor',
          setup: 'Activar, añadir tu clave de API y autorizar el proveedor',
        },
        apple: {
          name: 'Apple Intelligence',
          availability: 'macOS 26 o posterior',
          where: 'En tu Mac',
          connection: 'No hace falta',
          platforms: 'macOS 26 o posterior',
          hardware: 'Mac compatible con Apple Intelligence',
          cost: 'Gratis',
          setup: 'Activar Apple Intelligence en Ajustes del Sistema',
        },
      },
      note: 'La transcripción siempre usa los modelos de voz locales de FrameLy, sea cual sea el modo elegido para resúmenes y temas.',
    },
    figure: 'El gestor de modelos en los Ajustes de FrameLy: instala, comprueba o elimina cada modelo. Los tamaños y licencias vienen de la propia app.',
    figureAlt: 'Gestor de modelos de FrameLy con modelos de transcripción y de lenguaje, tamaño, memoria y licencia',
  },
  how: {
    eyebrow: 'Cómo funciona',
    title: 'Del primer atajo al resumen.',
    steps: [
      { title: 'Instala', body: 'Descarga el instalador para tu sistema y abre FrameLy. Vive en la barra de menús o en la bandeja del sistema.' },
      { title: 'Da acceso', body: 'Permite la grabación de pantalla y, si quieres, el micrófono. FrameLy explica para qué sirve cada permiso.' },
      { title: 'Captura o graba', body: 'Usa un atajo o el panel rápido para hacer una captura o grabar la pantalla, una ventana o una región.' },
      { title: 'Elige la IA', body: 'Descarga los modelos locales en un solo paso, o elige Apple Intelligence o tu clave en la nube.' },
      { title: 'Transcribe', body: 'Envía una grabación o cualquier audio o vídeo a Transcripciones. Los archivos largos continúan donde se quedaron.' },
      { title: 'Resume y comparte', body: 'Genera el resumen y los temas, edítalos y exporta la transcripción en TXT, SRT, VTT, Markdown o PDF.' },
    ],
  },
  uses: {
    eyebrow: 'Casos de uso',
    title: 'Pensado para el trabajo diario.',
    items: [
      { title: 'Informes de errores', body: 'Una captura con una flecha y un correo desenfocado, o una grabación corta de los pasos.' },
      { title: 'Notas de reuniones', body: 'Graba una llamada con audio del sistema y micrófono, y obtén el resumen y los próximos pasos.' },
      { title: 'Tutoriales y demos', body: 'Graba una guía y exporta subtítulos en SRT o VTT.' },
      { title: 'Clases y charlas', body: 'Convierte una grabación larga en temas entre los que saltar.' },
      { title: 'Revisión de diseño', body: 'Señala detalles con formas y texto sobre la propia captura.' },
      { title: 'Contenido sensible', body: 'Mantén todo en tu ordenador con la IA local y el modo «Solo local».' },
    ],
  },
  demo: {
    eyebrow: 'En acción',
    title: 'Una grabación real, transcrita y resumida por FrameLy.',
    caption: 'Capturado de FrameLy funcionando en macOS. El audio de la reunión es voz sintetizada.',
  },
  download: {
    eyebrow: 'Descargar',
    title: 'Consigue FrameLy.',
    subtitle: 'Gratis para macOS y Windows. Elige tu sistema: recordaremos tu elección.',
    choose: 'Sistema operativo',
    detectedHint: 'Hemos detectado {os}.',
    notDetectedHint: 'No hemos podido identificar tu sistema. Elige uno:',
    button: 'Descargar para {os}',
    version: 'Versión',
    released: 'Publicación',
    arch: 'Arquitectura',
    format: 'Formato',
    size: 'Tamaño',
    checksum: 'SHA-256',
    checksums: 'Todas las sumas de verificación',
    requirements: 'Requisitos',
    install: 'Cómo instalar',
    previous: 'Versiones anteriores',
    noPrevious: 'Todavía no hay versiones anteriores.',
    changelog: 'Novedades',
    allReleases: 'Todas las versiones en GitHub',
    portable: 'Portátil (zip)',
    none: 'Todavía no hay un instalador publicado para {os}. Sigue el repositorio para enterarte cuando salga.',
    unavailableArch: 'Aún no disponible',
    archNames: { arm64: 'Apple Silicon (arm64)', x64: 'x64 (Intel/AMD de 64 bits)', universal: 'Universal' },
    formats: { dmg: 'Imagen de disco (.dmg)', exe: 'Instalador (.exe, NSIS)', msi: 'Instalador (.msi)', zip: 'Portátil (.zip)' },
    req: {
      macos: ['macOS 14 Sonoma o posterior', 'Mac con Apple Silicon (M1 o posterior)', 'Macs Intel: aún sin versión', 'Funciones de Apple Intelligence: macOS 26 o posterior'],
      windows: ['Windows 10 22H2 o Windows 11', 'x64 (Intel o AMD de 64 bits) — ARM64: aún sin versión', 'Microsoft Edge WebView2 (incluido en Windows 11; el instalador puede configurarlo)', 'IA local: CPU con AVX2'],
    },
    steps: {
      macos: [
        'Abre el .dmg y arrastra FrameLy a Aplicaciones.',
        'Esta versión aún no está notarizada por Apple. La primera vez, abre FrameLy, ve a Ajustes del Sistema › Privacidad y seguridad y haz clic en «Abrir igualmente».',
        'Permite la Grabación de pantalla (y el Micrófono, si lo usas) cuando FrameLy lo pida.',
      ],
      windows: [
        'Ejecuta el instalador -setup.exe. Se instala solo para tu usuario, sin necesidad de administrador.',
        'El instalador aún no tiene firma de código, así que SmartScreen de Windows puede avisarte. Haz clic en «Más información» y luego en «Ejecutar de todas formas».',
        'FrameLy se abre en la bandeja del sistema. Usa el panel rápido o Ctrl+Alt+Mayús+Espacio.',
      ],
    },
    verify: 'Compara el SHA-256 del archivo con el valor de arriba: shasum -a 256 <archivo> en macOS, o Get-FileHash <archivo> en PowerShell.',
    windowsNote: 'Windows: la grabación de pantalla completa se ha probado en hardware real; la grabación de ventana y región, la pausa, el audio y la IA local aún se están verificando en más PC con Windows.',
  },
  faq: {
    eyebrow: 'Preguntas frecuentes',
    title: 'Dudas, resueltas.',
    more: 'Todas las preguntas',
    items: [
      {
        q: '¿FrameLy es gratis?',
        a: 'Sí. FrameLy es gratis y de código abierto con licencia MIT. La IA local y Apple Intelligence son gratuitas; los proveedores en la nube que conectes con tu clave te cobran directamente.',
      },
      {
        q: '¿Mi audio o vídeo sale de mi ordenador?',
        a: 'No para la transcripción: siempre se hace en tu ordenador. Si eliges un proveedor en la nube (Anthropic u OpenAI) para resúmenes o temas, solo se envía el texto de la transcripción a ese proveedor, después de que lo autorices.',
      },
      {
        q: '¿Qué es FrameLy AI Cloud?',
        a: 'El servicio gratuito en la nube de FrameLy para resúmenes y temas, sin cuenta ni clave. Se le envía el texto de la transcripción y un proveedor de IA asociado genera el resultado; según ese proveedor, el contenido puede usarse para mejorar sus servicios, y la app te lo indica antes de usarlo. Ya forma parte de la app, pero aún no está en marcha, por eso las versiones actuales lo muestran como no disponible.',
      },
      {
        q: '¿FrameLy funciona sin conexión?',
        a: 'Las capturas, la grabación y, una vez descargados los modelos, la transcripción, los resúmenes y los temas con IA local funcionan sin internet. Descargar modelos y usar proveedores en la nube requiere conexión.',
      },
      {
        q: '¿Cuánto espacio ocupan los modelos de IA?',
        a: 'El modelo de voz ocupa de 181 MB a 1 GB según la precisión, y el modelo de lenguaje de 2,6 a 5,3 GB según el perfil. FrameLy sugiere un perfil según la memoria de tu ordenador, y puedes eliminar los modelos cuando quieras.',
      },
      {
        q: '¿Qué Macs admiten Apple Intelligence en FrameLy?',
        a: 'Los Macs compatibles con Apple Intelligence con macOS 26 o posterior y Apple Intelligence activado en Ajustes del Sistema. FrameLy en sí funciona con macOS 14 o posterior.',
      },
      {
        q: '¿FrameLy funciona en Macs Intel o Windows ARM?',
        a: 'Todavía no. Las versiones actuales son para Macs con Apple Silicon y PC con Windows de 64 bits Intel/AMD.',
      },
      {
        q: '¿Distingue a quién habla?',
        a: 'Todavía no. El reconocimiento de voz local de FrameLy no identifica a los hablantes.',
      },
      {
        q: '¿Por qué macOS o Windows me avisan al instalar?',
        a: 'Los instaladores aún no están firmados con un Developer ID de Apple ni con un certificado de firma de código de Windows. Puedes comprobar cada archivo con su suma SHA-256 en la sección de descarga y en GitHub.',
      },
      {
        q: '¿En qué idiomas está la app?',
        a: 'La interfaz está en español, inglés, portugués (Brasil), francés y alemán. La transcripción detecta automáticamente el idioma hablado.',
      },
    ],
  },
  footer: {
    tagline: 'Grabación de pantalla, capturas y transcripción en tu ordenador, para macOS y Windows.',
    product: 'Producto',
    resources: 'Recursos',
    legal: 'Legal',
    github: 'Código fuente',
    releases: 'Versiones',
    issues: 'Informar de un problema',
    changelog: 'Novedades',
    faq: 'Preguntas frecuentes',
    privacy: 'Privacidad',
    terms: 'Términos de uso',
    license: 'Licencia',
    rights: '© {year} FrameLy. Publicado con licencia MIT.',
    trademarks: 'Apple, macOS y Apple Intelligence son marcas de Apple Inc. Windows es una marca de Microsoft. Claude y Anthropic son marcas de Anthropic. OpenAI es una marca de OpenAI.',
  },
  pages: {
    pending: 'Borrador, pendiente de revisión legal. Este texto describe cómo funciona FrameLy hoy; no es asesoramiento jurídico.',
    updated: 'Última actualización: {date}',
    back: 'Volver al inicio',
    toc: 'En esta página',
    notFound: { title: 'Página no encontrada', body: 'La página que buscas no existe o se ha movido.' },
    changelog: {
      title: 'Novedades',
      description: 'Todas las versiones de FrameLy, con novedades, mejoras y correcciones.',
      intro: 'Todos los cambios relevantes de FrameLy. Las versiones siguen el Versionado Semántico.',
      added: 'Novedades',
      changed: 'Mejoras',
      fixed: 'Correcciones',
      known: 'Limitaciones conocidas',
      assets: 'Descargas',
      release: 'Versión en GitHub',
      empty: 'Todavía no hay versiones publicadas.',
    },
    faq: { title: 'Preguntas frecuentes', description: 'Respuestas sobre FrameLy: precio, privacidad, modelos de IA, compatibilidad e instalación.' },
    license: {
      title: 'Licencia',
      description: 'FrameLy es de código abierto con licencia MIT.',
      intro: 'El código fuente de FrameLy se publica con licencia MIT. Los modelos de IA que descarga FrameLy tienen sus propias licencias, indicadas abajo.',
      models: 'Modelos que descarga FrameLy',
      modelsBody: 'El reconocimiento de voz usa modelos Whisper (MIT) a través de whisper.cpp. Los resúmenes y temas con IA local usan modelos de lenguaje abiertos de las familias Gemma y Qwen, distribuidos con licencia Apache 2.0. La licencia de cada modelo también aparece en el gestor de modelos de la app.',
      thirdParty: 'Software de terceros',
      thirdPartyBody: 'FrameLy está hecho con Tauri, Rust, Svelte, whisper.cpp y llama.cpp, entre otros proyectos de código abierto, cada uno con su propia licencia.',
      full: 'Texto completo de la licencia',
    },
    privacy: {
      title: 'Política de privacidad',
      description: 'Cómo trata FrameLy las capturas, grabaciones, transcripciones y los datos enviados a proveedores de IA.',
      sections: [
        {
          h: 'Resumen',
          p: [
            'FrameLy es una aplicación de escritorio. No tiene cuentas, registro ni rastreadores de analítica o publicidad. Tus capturas, grabaciones y transcripciones se guardan en tu ordenador, en las carpetas que elijas.',
            'Los datos solo salen de tu ordenador cuando usas una función que necesita internet: descargar modelos de IA, o un proveedor de IA en la nube que hayas activado y autorizado.',
          ],
        },
        {
          h: 'Capturas y grabaciones',
          p: [
            'Las capturas se guardan como archivos en las carpetas definidas en Ajustes (por defecto, Imágenes y Películas/Vídeos). FrameLy mantiene un índice local para la biblioteca. FrameLy no sube estos archivos a ningún sitio.',
          ],
        },
        {
          h: 'Transcripción (siempre local)',
          p: [
            'El reconocimiento de voz se ejecuta en tu ordenador con modelos descargados. El audio y el vídeo nunca se envían a un servidor para transcribirlos. Las transcripciones, resúmenes y temas se guardan en una base de datos local en tu ordenador, y puedes borrarlos desde la app.',
          ],
        },
        {
          h: 'IA local y Apple Intelligence',
          p: [
            'Con la IA local de FrameLy, los resúmenes y temas se generan en tu ordenador. Con Apple Intelligence, los genera el modelo de Apple en el propio dispositivo a través de macOS; FrameLy no envía nada a servidores de Apple para ello. A Apple Intelligence se aplican las políticas de Apple.',
          ],
        },
        {
          h: 'IA en la nube (opcional)',
          p: [
            'FrameLy AI Cloud es el servicio opcional en la nube del propio FrameLy. Aún no está en marcha, así que las versiones publicadas no pueden usarlo. Cuando lo esté, y solo después de que lo autorices, FrameLy le enviará el texto de la transcripción y la tarea solicitada, y un proveedor de IA asociado generará el resultado. Tu ordenador se identifica con una clave anónima creada en él, sin cuenta. El servicio no guarda el contenido: almacena las claves anónimas y los recuentos de uso diario. Según el plan del proveedor, el contenido puede usarse para mejorar sus servicios; la app lo muestra antes de usarlo. Esta política se actualizará cuando el servicio esté en marcha.',
            'Si activas los proveedores externos, añades tu clave de API y autorizas un proveedor, FrameLy envía el texto de la transcripción (no el audio ni el vídeo) y las instrucciones del resumen o de los temas directamente a ese proveedor: Anthropic u OpenAI. Se aplican los términos y la política de privacidad del proveedor, incluido cuánto tiempo conserva los datos. El uso se factura a tu cuenta con él.',
            'Las claves de API y la clave anónima de FrameLy AI Cloud se guardan en el Llavero de macOS o en el Administrador de credenciales de Windows, no en archivos normales. El ajuste «Solo local» bloquea todas estas conexiones.',
          ],
        },
        {
          h: 'Descarga de modelos',
          p: [
            'Al descargar modelos de IA, tu ordenador se conecta a los servidores que los alojan (como Hugging Face). Esos servidores ven una solicitud de descarga normal, incluida tu dirección IP. Cada archivo se comprueba con un hash SHA-256 conocido.',
          ],
        },
        {
          h: 'Este sitio web',
          p: [
            'Este sitio es estático y está alojado en GitHub Pages. No usa cookies ni analítica. Tus elecciones de idioma y sistema operativo se guardan solo en el almacenamiento local de tu navegador. GitHub puede tratar datos técnicos, como direcciones IP, para servir el sitio y las descargas; consulta la declaración de privacidad de GitHub.',
          ],
        },
        {
          h: 'Tu control',
          p: [
            'Puedes borrar capturas, transcripciones y modelos desde la app o tu sistema de archivos cuando quieras. Desinstalar FrameLy elimina la app; los archivos de tus carpetas de captura se quedan hasta que los borres.',
          ],
        },
        {
          h: 'Contacto',
          p: ['Preguntas sobre privacidad: abre una incidencia (issue) en el repositorio de FrameLy en GitHub.'],
        },
      ],
    },
    terms: {
      title: 'Términos de uso',
      description: 'Los términos para usar FrameLy y este sitio web.',
      sections: [
        {
          h: 'El software',
          p: ['FrameLy se ofrece gratis con licencia MIT. Puedes usarlo, copiarlo, modificarlo y distribuirlo según los términos de esa licencia.'],
        },
        {
          h: 'Sin garantía',
          p: [
            'FrameLy se proporciona «tal cual», sin garantía de ningún tipo. Las transcripciones, resúmenes y temas se generan automáticamente y pueden contener errores; comprueba la información importante antes de confiar en ella.',
          ],
        },
        {
          h: 'Tu contenido y tu responsabilidad',
          p: [
            'Eres responsable de lo que capturas, grabas y procesas. Asegúrate de tener el derecho y, cuando sea necesario, el consentimiento de las personas implicadas para grabar pantallas, audio y conversaciones, y para enviar transcripciones a proveedores de IA de terceros.',
          ],
        },
        {
          h: 'Servicios de terceros',
          p: [
            'Los proveedores de IA en la nube (Anthropic, OpenAI), Apple Intelligence y los servicios que alojan los modelos los gestionan terceros con sus propios términos. Usarlos con FrameLy es opcional, y sus cargos son entre tú y ellos.',
          ],
        },
        {
          h: 'Descargas',
          p: ['Descarga FrameLy solo desde el repositorio oficial de GitHub o este sitio, y comprueba la suma SHA-256 publicada con cada versión.'],
        },
        {
          h: 'Cambios',
          p: ['Estos términos pueden cambiar a medida que FrameLy evoluciona. La fecha de arriba indica la versión más reciente.'],
        },
      ],
    },
  },
};
