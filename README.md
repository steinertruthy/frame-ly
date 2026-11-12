# FrameLy

Gravação de tela e screenshots para macOS e Windows — simples, rápido e leve.

> Estado atual: **Etapa 3**. Screenshots (região com lupa, janela, tela inteira) e gravação de
> tela no macOS: tela inteira, janela ou região, H.264/HEVC por hardware, contagem regressiva,
> pausar/retomar, prévia do vídeo e recuperação de gravações interrompidas. Gravação no Windows e
> áudio chegam nas próximas etapas — veja [docs/roadmap.md](docs/roadmap.md).

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

## Linha de comando

Com o FrameLy aberto, o executável repassa ações para a instância em execução — útil para Raycast,
Alfred, Atalhos do macOS ou AutoHotkey:

```bash
/Applications/FrameLy.app/Contents/MacOS/framely --capture region   # region | window | screen | cancel
/Applications/FrameLy.app/Contents/MacOS/framely --record screen     # region | window | screen | pause | stop
```
