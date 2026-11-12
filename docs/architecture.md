# Arquitetura

## Visão geral

```
UI (Svelte 5, WebView do sistema)  ──IPC tipado──  Core Rust (compartilhado)  ──trait──  Engines nativas
 janela principal, painel da bandeja,               settings, biblioteca, estado,           macOS: Swift (SCK, AVFoundation)
 overlay de seleção, editor                          recuperação, logs, i18n do tray        Windows: Rust (WGC, WASAPI, MF)
```

**Princípio:** o pipeline de mídia (captura → codificação → mux) é 100% nativo por plataforma — é onde
estão desempenho e qualidade. Tudo que não é mídia é compartilhado. Existe uma única abstração de
plataforma (`CaptureBackend`, Etapa 1); não há outras camadas.

## Decisões de stack

| Decisão | Motivo |
|---|---|
| Tauri 2 em vez de Electron | WebView do sistema: ~30–60 MB de RAM em repouso, binário ~10 MB, inicialização rápida. |
| Svelte 5 | Sem virtual DOM, runtime mínimo; o bundle da janela principal tem ~30 KB gzip. |
| Engine macOS em Swift | ScreenCaptureKit/AVFoundation usam delegates, async e blocks; em Swift o código é idiomático e seguro. Ligado ao Rust por uma fronteira C pequena. |
| Engine Windows em Rust (`windows` crate) | Acesso direto a WinRT/COM sem outra linguagem. |
| Sem FFmpeg na v1 | Codificadores de hardware nativos (VideoToolbox, Media Foundation) já produzem MP4 H.264/HEVC + AAC. Evita ~60 MB, licenças GPL/LGPL e processos extras. |
| SQLite para a biblioteca, JSON para ajustes | Consultas rápidas de busca/ordenação; ajustes legíveis e migráveis. |

## Código

| Caminho | Responsabilidade |
|---|---|
| `src-tauri/src/core/` | Lógica sem SO: `errors` (AppError → `{code,message}`), `settings` (versionado, migração, escrita atômica, backup de arquivo corrompido), `paths` (nomes de arquivo), `locale`, `logging`. |
| `src-tauri/src/commands/` | Camada IPC fina: valida, delega ao core, emite eventos. |
| `src-tauri/src/platform/` | Trait `CaptureBackend` e tipos compartilhados; `macos/` (FFI para FrameLyKit) e `windows/` (Win32/WinRT). Ver [platform.md](platform.md). |
| `native/macos/FrameLyKit/` | Engine macOS em Swift (biblioteca estática). |
| `src-tauri/src/windows.rs` | Criação das janelas nativas e aparência por SO (vibrancy no macOS). |
| `src-tauri/src/tray.rs` | Ícone da bandeja/menu bar com ações de captura. |
| `src-tauri/src/screenshot.rs` | Sessão de captura, overlays, entrega (PNG, pasta, clipboard) e prévia. |
| `src-tauri/src/recording.rs` | Fluxo de gravação: alvo, contagem, controle, entrega, recuperação. |
| `src-tauri/src/core/recorder.rs`, `core/video.rs` | Estado/relógio da gravação; tamanho de saída e bitrate (puros, testados). |
| `src-tauri/src/protocol.rs` | Protocolo `framely://` para imagens em memória. |
| `src-tauri/src/core/image.rs` | Quadros BGRA, recorte, PNG e BMP (puro, testado). |
| `src/lib/design/tokens.css` | Design tokens (cores, tipografia, raio, movimento) com temas claro/escuro. |
| `src/lib/i18n/` | Dicionários tipados pt-BR/en; uma chave faltando quebra o type-check. |
| `src/lib/ipc/` | Wrappers tipados de comandos e eventos; encaminhamento de erros da UI para o log. |
| `src/lib/stores/` | Estado reativo (runes). A UI só reflete o que o backend salvou de fato. |
| `src/windows/<janela>/` | Um entry HTML por tipo de janela; cada janela carrega só o que usa. |

## Fluxo de screenshot (`src-tauri/src/screenshot.rs`)

```
Tela inteira:  capture_display(monitor sob o cursor) ─────────────────────────────┐
Região/Janela: capture_display(todos, em paralelo) → quadros congelados          │
               → 1 overlay por monitor (overlay.html)                             │
               → overlay carrega o quadro via framely://localhost/frame/<id>     │
               → região: recorte do quadro congelado                              │
                 janela: capture_window nativo (fallback: recorte do quadro) ─────┤
                                                                                   ▼
                     encode PNG → salvar (opcional) → clipboard (opcional) → prévia
```

- **Quadro congelado:** o usuário seleciona sobre a imagem exata que será salva (menus abertos e
  estados de hover são preservados) e a lupa mostra pixels reais.
- **`framely://`:** protocolo próprio que serve imagens em memória (BMP sem compressão para os
  overlays, PNG da última captura para a prévia). Nada vai para arquivos temporários e o asset
  protocol continua desligado.
- **Conversão de coordenadas:** o overlay envia a seleção em pixels CSS + o tamanho do viewport; o
  Rust converte usando o tamanho real do quadro. Funciona igual com pontos (macOS) e DPI (Windows).
- **Janelas do FrameLy nunca aparecem nas capturas:** o filtro do ScreenCaptureKit exclui o próprio
  app; no Windows, `WDA_EXCLUDEFROMCAPTURE` nas janelas de overlay e prévia.
- **Desempenho (M-series, 3024×1964):** captura ~85 ms; PNG ~95 ms em release; clipboard ~0 ms
  (o PNG já codificado vai direto para o NSPasteboard).

## Fluxo de gravação (`src-tauri/src/recording.rs`)

```
alvo: tela (monitor sob o cursor) | janela/região (mesmo overlay das screenshots, em vermelho)
  → contagem regressiva (janela própria; Esc/clique cancela)
  → FrameLyKit: SCStream → AVAssetWriter (VideoToolbox) → MP4 fragmentado em cache/recordings/<id>.mp4
     + manifesto <id>.json + miniatura <id>.jpg
  → status a cada 500 ms → ícone/tempo na menu bar + banner na tela inicial
  → parar (UI, menu, clique no ícone, CLI, ou o próprio sistema) → arquivo finalizado
  → movido para a pasta de gravações → prévia do vídeo
```

- **Máquina de estados** (`core/recorder.rs`): `Idle → Countdown → Recording ⇄ Paused → Finalizing`,
  com transições inválidas rejeitadas e relógio que desconta pausas (testado).
- **Pausa** não para o stream: as amostras do intervalo pausado são descartadas e as seguintes são
  re-temporizadas (`PauseTimeline.swift`, testado). Retomar é instantâneo.
- **Tamanho e bitrate** (`core/video.rs`): resolução nativa/1080p/720p mantendo proporção, limite
  do codec (H.264 ≤ 4096×2304), dimensões pares, bitrate por bits/pixel/quadro conforme a qualidade.
- **Recuperação:** o MP4 é fragmentado a cada 2 s. Se o FrameLy encerrar inesperadamente, o próximo
  início encontra o manifesto e move o arquivo para a pasta de gravações como "(recovered)" — testado
  com `kill -9` durante a gravação.
- **Parada pelo sistema** (janela fechada, monitor desconectado, botão "Parar" do macOS, permissão
  revogada): o arquivo é finalizado e entregue normalmente.

## Áudio (macOS)

```
ScreenCaptureKit (áudio do sistema, sem o som do próprio FrameLy) ─┐
AVCaptureSession (microfone escolhido, relógio convertido p/ host) ─┤→ AudioMixer → AAC 192 kbps
                                                                     │  (48 kHz estéreo, ganhos,
                     vídeo: primeiro quadro = tempo zero do mix ─────┘   soft-clip, pausa)
```

- **Uma única faixa AAC** com tudo mixado: toca igual em qualquer player (players que só leem a
  primeira faixa não perdem o microfone).
- **Sincronia:** cada buffer é posicionado pelo seu horário no relógio do host — o mesmo do vídeo.
  Jitter de até 10 ms é colado de forma contínua (sem cliques); desvios maiores (deriva do relógio
  do dispositivo) ressincronizam. O offset A/V fica limitado a ~10 ms em gravações de qualquer duração.
- **Fontes lentas ou removidas** nunca travam as outras: a saída anda no máximo 250 ms atrás da fonte
  mais adiantada; um microfone desconectado vira silêncio e a gravação continua.
- **Medição real** (`AVSyncTests`): um tom agendado num horário exato do host aparece no arquivo a
  +6–8 ms do esperado, já considerando a latência de saída do alto-falante (28 ms no MacBook Pro).
- **Permissão e dispositivo:** com o microfone ligado, a permissão é pedida no início da gravação; se
  negada, ou sem microfone, grava sem ele e avisa. Dispositivo escolhido ausente → padrão do sistema.

## Painel rápido e atalhos

- **Painel** (`panel.rs`, `windows/panel/`): janela criada oculta no início (abre instantaneamente),
  ancorada ao ícone da bandeja — abaixo dele na barra de menus, acima numa barra de tarefas inferior —
  e mantida dentro da área útil do monitor. Some ao perder o foco. No macOS é um `NSPanel` não
  ativante com material nativo de popover: abrir o painel não traz a janela principal para frente.
  Clique esquerdo no ícone abre o painel; clique direito abre o menu nativo (alternativa acessível).
- **Atalhos** (`core/shortcuts.rs` + `shortcuts.rs`): modelo puro com normalização, exigência de
  ⌘/Ctrl/Alt (para não sequestrar a digitação), lista de combinações reservadas por SO e detecção de
  duplicatas; registro no SO via `tauri-plugin-global-shortcut`. Se outro app já usa a combinação, a
  falha é reportada por ação e aparece nos Ajustes. Enquanto o usuário grava um novo atalho, os
  atuais ficam suspensos.

## Comportamento de ciclo de vida

- Fechar a janela principal a **esconde**; o app continua na bandeja (gravações sobrevivem).
- Uma segunda instância apenas foca a existente (`tauri-plugin-single-instance`).
- A janela é criada oculta e exibida após o primeiro render (sem flash em branco).

## Erros e logs

- Comandos retornam `AppResult<T>`; a UI traduz `error.<code>`.
- Logs JSON com rotação diária (7 arquivos) em `~/Library/Logs/app.framely.desktop/` ou
  `%LOCALAPPDATA%\app.framely.desktop\logs\`. Nível via `FRAMELY_LOG` (ex.: `FRAMELY_LOG=debug`).
- Erros não tratados de qualquer janela da UI também vão para esse log.

## Segurança

- CSP restritiva; o asset protocol está desligado até a biblioteca precisar dele.
- A UI não tem permissão genérica de abrir caminhos: `open_folder` só aceita as pastas do FrameLy, e
  as ações da prévia só operam sobre a última captura feita pelo próprio app.
