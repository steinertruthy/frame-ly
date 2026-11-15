## [0.1.0] - 2026-11-15

Primeira versão pública.

### Added

- Screenshots da tela inteira, de uma janela ou de uma região, com edição direto na seleção: setas, formas, texto, destaque, desfoque e recorte, com desfazer e refazer. Copie para a área de transferência ou salve.
- Gravação da tela inteira, de uma janela ou de uma região, com áudio do sistema e microfone, pausa e retomada, contagem regressiva, H.264 ou HEVC e tamanho estimado do arquivo antes de começar.
- Painel rápido na barra de menus ou na bandeja do sistema, atalhos globais para todos os modos de captura e linha de comando (`--capture`, `--record`, `--panel`, `--show`, `--edit`, `--transcribe`).
- Biblioteca de screenshots e gravações, e ajustes por categoria.
- Interface em português (Brasil), inglês, espanhol, francês e alemão.
- Transcrição de gravações e de áudios ou vídeos importados no seu computador (whisper.cpp), com detecção automática de idioma, tempo de cada palavra, edição e exportação em TXT, SRT, VTT, Markdown e PDF.
- Resumos e tópicos com a FrameLy AI no seu computador (llama.cpp), o Apple Intelligence no macOS 26 ou posterior, ou Anthropic e OpenAI com a sua chave de API, após sua autorização.
- Gerenciador de modelos com downloads verificados por SHA-256, pausa e retomada, e perfil sugerido pela memória do computador.
- Modo “Exclusivamente local”, que bloqueia qualquer conexão.

### Known limitations

- O gravador do Windows foi validado em hardware real para a tela inteira; gravação de janela e região, pausa, áudio e IA local ainda precisam de mais testes em PCs com Windows.
- Os instaladores ainda não são assinados com um Developer ID da Apple nem com certificado de assinatura de código do Windows.
- Há versões apenas para Macs com Apple Silicon e Windows 64 bits Intel/AMD.
- As transcrições não identificam quem está falando.
- Uma gravação captura um monitor por vez.
