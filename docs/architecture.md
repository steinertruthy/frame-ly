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
