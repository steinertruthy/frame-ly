# FrameLy

Gravação de tela e screenshots para macOS e Windows — simples, rápido e leve.

> Estado atual: **Etapa 8 + redesign**. Nova identidade visual escura, screenshot com edição
> direto na seleção (setas, formas, texto, destaque, desfoque, recorte, desfazer/refazer), gravação
> com áudio no macOS e no Windows, painel rápido, biblioteca, ajustes por categoria e interface em
> cinco idiomas (pt-BR, en, es, fr, de). O gravador do Windows ainda não foi validado em hardware
> real — veja [docs/roadmap.md](docs/roadmap.md).

## Stack

Tauri 2 · Rust · Svelte 5 + TypeScript · APIs nativas (ScreenCaptureKit/AVFoundation no macOS,
Windows.Graphics.Capture/WASAPI/Media Foundation no Windows). Sem FFmpeg. Detalhes e justificativas
em [docs/architecture.md](docs/architecture.md).

## Requisitos

| | Versão |
|---|---|
| macOS | 14 (Sonoma) ou superior |
| Windows | 10 22H2 ou superior (11 recomendado) |
| Rust | stable (1.80+) |
| Node | 22 LTS (`.nvmrc`) + pnpm 10 |
| macOS dev | Xcode 15+ |
| Windows dev | Visual Studio Build Tools (C++), WebView2 |

## Desenvolvimento

```bash
pnpm install
pnpm tauri dev        # app com hot reload
pnpm check            # svelte-check + tsc
pnpm test             # testes da UI (Vitest)
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
pnpm tauri build      # pacote de release
```

Mais em [docs/development.md](docs/development.md).

## Atalhos padrão

| Ação | macOS | Windows |
|---|---|---|
| Abrir painel rápido | ⌥⇧⌘Space | Ctrl+Alt+Shift+Space |
| Capturar tela inteira / região / janela | ⌥⇧⌘3 / 4 / 5 | Ctrl+Alt+Shift+3 / 4 / 5 |
| Gravar região | ⌥⇧⌘R | Ctrl+Alt+Shift+R |
| Parar gravação | ⌥⇧⌘S | Ctrl+Alt+Shift+S |
| Pausar / retomar | ⌥⇧⌘P | Ctrl+Alt+Shift+P |

Todos configuráveis em Ajustes › Atalhos de teclado (gravar janela/tela inteira começam sem atalho).

## Linha de comando

Com o FrameLy aberto, o executável repassa ações para a instância em execução — útil para Raycast,
Alfred, Atalhos do macOS ou AutoHotkey:

```bash
/Applications/FrameLy.app/Contents/MacOS/framely --capture region   # region | window | screen | cancel
/Applications/FrameLy.app/Contents/MacOS/framely --record screen     # region | window | screen | pause | stop
/Applications/FrameLy.app/Contents/MacOS/framely --panel             # abre/fecha o painel rápido
/Applications/FrameLy.app/Contents/MacOS/framely --show library      # home | library | settings
/Applications/FrameLy.app/Contents/MacOS/framely --edit ~/Pictures/FrameLy/shot.png
```
