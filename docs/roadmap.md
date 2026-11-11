# Roadmap

| Etapa | Escopo | Estado |
|---|---|---|
| 0 | Toolchain, esqueleto Tauri + Svelte, design tokens, janela principal, bandeja, ajustes, logs, CI | ✅ |
| 1 | `CaptureBackend`, FrameLyKit (Swift), listagem de monitores/janelas/áudio, permissões | ✅ |
| 2 | Screenshots ponta a ponta (tela, monitor, janela, região), clipboard, preview | ⏳ |
| 3 | Gravação de vídeo no macOS (SCK + AVAssetWriter), pausa, recuperação | |
| 4 | Gravação de vídeo no Windows (WGC + Media Foundation) | |
| 5 | Áudio: microfone, sistema, mix, sincronização | |
| 6 | Painel rápido da bandeja + atalhos globais | |
| 7 | Biblioteca + Dashboard | |
| 8 | Editor de screenshots | |
| 9 | Ajustes completos, polimento, instaladores, assinatura | |

## Limitações conhecidas (v1)

- Uma gravação captura um monitor por vez; regiões não atravessam monitores.
- Windows 10 mostra uma borda amarela durante a captura (limitação do sistema; ausente no Windows 11).
- Conteúdo protegido por DRM aparece preto / sem áudio — comportamento do SO, não contornado.
