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
