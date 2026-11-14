# Roadmap

| Etapa | Escopo | Estado |
|---|---|---|
| 0 | Toolchain, esqueleto Tauri + Svelte, design tokens, janela principal, bandeja, ajustes, logs, CI | ✅ |
| 1 | `CaptureBackend`, FrameLyKit (Swift), listagem de monitores/janelas/áudio, permissões | ✅ |
| 2 | Screenshots ponta a ponta (tela, monitor, janela, região), clipboard, preview | ✅ |
| 3 | Gravação de vídeo no macOS (SCK + AVAssetWriter), pausa, recuperação | ✅ |
| 4 | Gravação de vídeo no Windows (WGC + Media Foundation) | ✅ (compila e passa no lint; falta validar em hardware Windows) |
| 5 | Áudio: microfone, sistema, mix, sincronização (macOS e Windows) | ✅ |
| 6 | Painel rápido da bandeja + atalhos globais | ✅ |
| 7 | Biblioteca + Dashboard | ✅ |
| 8 | Editor de screenshots | ✅ |
| 8.5 | Redesign: identidade escura, edição inline na seleção, ajustes por categoria, 5 idiomas, iniciar com o sistema, notificações | ✅ |
| 9 | Ajustes completos, polimento, instaladores, assinatura | ⏳ |

## Limitações conhecidas (v1)

- Uma gravação captura um monitor por vez; regiões não atravessam monitores.
- Windows 10 mostra uma borda amarela durante a captura (limitação do sistema; ausente no Windows 11).
- Conteúdo protegido por DRM aparece preto / sem áudio — comportamento do SO, não contornado.
- Vídeos com taxa de quadros variável: o ScreenCaptureKit só entrega quadros quando a tela muda, e
  cada quadro é gravado com seu horário real (telas paradas quase não ocupam espaço). Todos os players
  comuns lidam bem com isso; alguns editores de vídeo podem preferir converter para taxa constante.
- H.264 é limitado a 4096×2304 pelo hardware: gravações nativas de telas 5K em H.264 são reduzidas
  proporcionalmente (use HEVC para manter a resolução total).
- Windows: HEVC depende da extensão "HEVC Video Extensions" da Microsoft Store; sem ela o FrameLy
  grava em H.264 (telas acima de 4096 px de largura com HEVC escolhido podem falhar nesse caso).
- Windows: o som do computador vem do loopback do dispositivo de saída padrão (inclui tudo o que toca
  nele); os sons do FrameLy tocam fora da janela de captura.
- A biblioteca lista os arquivos diretamente nas pastas de gravações e screenshots (sem subpastas) e
  se atualiza ao abrir a biblioteca; não há monitoramento contínuo do sistema de arquivos.
