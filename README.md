<p align="center"><img src="assets/logo.svg" width="96" alt="FrameLy"></p>

# FrameLy

Gravação de tela e screenshots para macOS e Windows, com transcrição, resumos e tópicos por IA que
pode rodar inteiramente no seu computador.

**[Site](https://steinertruthy.github.io/frame-ly/)** ·
**[Download](https://github.com/steinertruthy/frame-ly/releases/latest)** ·
**[Novidades](CHANGELOG.md)** · Licença [MIT](LICENSE)

> *English:* FrameLy is a screen recorder and screenshot tool for macOS 14+ (Apple Silicon) and
> Windows 10 22H2+ (x64) that transcribes recordings on-device and writes summaries and topics with
> local models, Apple Intelligence or your own Anthropic/OpenAI key. See the
> [website](https://steinertruthy.github.io/frame-ly/en/).

## O que faz

- **Screenshots** da tela, de uma janela ou de uma região, com edição direto na seleção (setas,
  formas, texto, destaque, desfoque, recorte, desfazer/refazer).
- **Gravação de tela** com áudio do sistema e microfone, pausa, contagem regressiva, H.264/HEVC.
- **Transcrição** local (whisper.cpp) com detecção de idioma, tempo por palavra e exportação em
  TXT, SRT, VTT, Markdown e PDF.
- **Resumos e tópicos** com a FrameLy AI local (llama.cpp), o Apple Intelligence (macOS 26+) ou
  Anthropic/OpenAI com a sua chave — sempre com autorização explícita para provedores externos.
- Painel rápido, biblioteca, atalhos globais, linha de comando e interface em cinco idiomas
  (pt-BR, en, es, fr, de).

Limitações conhecidas e o estado de cada plataforma estão no [CHANGELOG](CHANGELOG.md) e em
[docs/roadmap.md](docs/roadmap.md). Os instaladores ainda não têm assinatura Apple/Microsoft —
veja [docs/signing.md](docs/signing.md).

## Repositório

| Pasta | Conteúdo |
|---|---|
| `src/`, `src-tauri/`, `native/` | O app (Svelte 5 + TypeScript, Rust/Tauri 2, Swift no macOS) |
| `cloud/` | Serviço FrameLy Cloud (Cloudflare Worker) — ainda não publicado |
| `website/` | Site oficial (Astro, GitHub Pages) |
| `docs/` | Arquitetura, desenvolvimento, plataformas, [releases](docs/release.md), [assinatura](docs/signing.md) |
| `.github/workflows/` | CI do app, site e release |

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

Versões e releases: [docs/release.md](docs/release.md). Site: [website/README.md](website/README.md).

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
/Applications/FrameLy.app/Contents/MacOS/framely --show library      # home | library | transcripts | settings[:<categoria>]
/Applications/FrameLy.app/Contents/MacOS/framely --show transcript:3:summary   # transcrição 3, aba resumo/tópicos
/Applications/FrameLy.app/Contents/MacOS/framely --edit ~/Pictures/FrameLy/shot.png
```
