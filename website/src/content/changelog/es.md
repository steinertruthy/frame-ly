## [0.2.0] - 2026-11-16

FrameLy AI Cloud ya está en marcha y ahora es la opción predeterminada para resúmenes y temas.

### Added

- FrameLy AI Cloud en las versiones publicadas: resúmenes y temas en la nube, gratis, sin cuenta ni clave y con un límite diario por ordenador. Un proveedor de IA asociado genera el resultado y puede usar el contenido para mejorar sus servicios; la app lo indica antes del primer uso.

### Changed

- Las instalaciones nuevas y los ajustes que seguían en la opción predeterminada anterior usan FrameLy AI Cloud para resúmenes y temas; cualquier otra elección se mantiene. FrameLy pide autorización antes de enviar nada, y el modo «Solo local» sigue bloqueando la nube.
- La configuración inicial solo descarga el modelo local de resúmenes cuando FrameLy AI Local es quien genera los resúmenes o temas.

### Fixed

- Los diálogos de confirmación (como la solicitud de autorización) ahora se abren encima de la hoja de Nueva transcripción, y Esc cierra primero el diálogo.

### Known limitations

- FrameLy AI Cloud no está disponible en el Espacio Económico Europeo, Suiza ni el Reino Unido; allí funcionan FrameLy AI Local, Apple Intelligence y tu propia clave de Anthropic u OpenAI.

## [0.1.0] - 2026-11-15

Primera versión pública.

### Added

- Capturas de la pantalla completa, una ventana o una región, con edición sobre la propia selección: flechas, formas, texto, resaltado, desenfoque y recorte, con deshacer y rehacer. Copia al portapapeles o guarda.
- Grabación de la pantalla completa, una ventana o una región, con audio del sistema y micrófono, pausa y reanudación, cuenta atrás, H.264 o HEVC y tamaño estimado del archivo antes de empezar.
- Panel rápido en la barra de menús o la bandeja del sistema, atajos globales para cada modo de captura y línea de comandos (`--capture`, `--record`, `--panel`, `--show`, `--edit`, `--transcribe`).
- Biblioteca de capturas y grabaciones, y ajustes por categoría.
- Interfaz en español, inglés, portugués (Brasil), francés y alemán.
- Transcripción de grabaciones y de audios o vídeos importados en tu ordenador (whisper.cpp), con detección automática del idioma, tiempo de cada palabra, edición y exportación a TXT, SRT, VTT, Markdown y PDF.
- Resúmenes y temas con FrameLy AI en tu ordenador (llama.cpp), Apple Intelligence en macOS 26 o posterior, o Anthropic y OpenAI con tu propia clave de API, tras tu autorización.
- Gestor de modelos con descargas verificadas por SHA-256, pausa y reanudación, y un perfil sugerido según la memoria del ordenador.
- Modo «Solo local», que bloquea cualquier conexión.

### Known limitations

- El grabador de Windows se ha validado en hardware real para la pantalla completa; la grabación de ventana y región, la pausa, el audio y la IA local aún necesitan más pruebas en PC con Windows.
- Los instaladores aún no están firmados con un Developer ID de Apple ni con un certificado de firma de código de Windows.
- Solo hay versiones para Macs con Apple Silicon y Windows de 64 bits Intel/AMD.
- Las transcripciones no identifican a los hablantes.
- Una grabación captura una pantalla cada vez.
