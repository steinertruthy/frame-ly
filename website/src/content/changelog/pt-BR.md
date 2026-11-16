## [0.3.0] - 2026-11-16

O FrameLy agora se atualiza sozinho, e o instalador do Windows ganhou a cara do FrameLy.

### Added

- Atualização automática: o FrameLy verifica os releases do GitHub em segundo plano (dá para desligar em Ajustes › Sobre, e nunca no modo “Exclusivamente local”), mostra um aviso discreto com as novidades e atualiza com um clique — progresso do download, verificação da assinatura, instalação e reinício. Uma gravação em andamento nunca é interrompida: a atualização é instalada quando ela termina. `framely --update` verifica e instala pela linha de comando.

### Changed

- O instalador do Windows agora tem o visual do FrameLy, os cinco idiomas do app, textos próprios nas telas de boas-vindas e conclusão, e uma opção na desinstalação que também remove as chaves de API salvas.

### Known limitations

- As versões 0.1.0 e 0.2.0 não têm o atualizador: baixe uma vez, pelo site, a primeira versão que o inclui.
- As versões para macOS ainda não são notarizadas, então o macOS pode pedir de novo a permissão de Gravação de Tela depois de uma atualização.

## [0.2.0] - 2026-11-16

A FrameLy AI Cloud está no ar e agora é o padrão para resumos e tópicos.

### Added

- FrameLy AI Cloud nas versões publicadas: resumos e tópicos na nuvem, gratuitos, sem conta nem chave, com limite diário por computador. Um provedor de IA parceiro gera o resultado e pode usar o conteúdo para melhorar os serviços dele — o app avisa antes do primeiro uso.

### Changed

- Instalações novas e ajustes que ainda estavam no padrão anterior usam a FrameLy AI Cloud para resumos e tópicos; qualquer outra escolha é mantida. O FrameLy pede autorização antes de enviar qualquer coisa, e o modo “Exclusivamente local” continua bloqueando a nuvem.
- A configuração inicial só baixa o modelo local de resumos quando a FrameLy AI Local é quem gera resumos ou tópicos.

### Fixed

- Os diálogos de confirmação (como o pedido de autorização) agora abrem acima da janela de Nova transcrição, e o Esc fecha primeiro o diálogo.

### Known limitations

- A FrameLy AI Cloud não está disponível no Espaço Econômico Europeu, na Suíça nem no Reino Unido; lá funcionam a FrameLy AI Local, o Apple Intelligence e a sua chave da Anthropic ou da OpenAI.

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
