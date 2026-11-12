# Integração com o sistema operacional

Toda chamada ao SO passa pela trait `CaptureBackend` (`src-tauri/src/platform/mod.rs`). Os métodos são
bloqueantes e rodam em `spawn_blocking` (camada `commands/capture.rs`).

## macOS — FrameLyKit (Swift)

`native/macos/FrameLyKit` é um Swift Package compilado como biblioteca estática pelo `build.rs`
(`swift-rs`) e linkado ao binário Rust, junto com ScreenCaptureKit, AVFoundation, CoreMedia,
CoreGraphics e AppKit. O runtime Swift vem do próprio macOS (`/usr/lib/swift`).

**Fronteira C:** cada função exportada (`@_cdecl`) devolve uma string JSON
`{"ok": …}` ou `{"err": {"code", "message"}}`, liberada por `fl_free_string`. Os tipos são `Codable` no
Swift e `serde` no Rust, ambos em camelCase. O caminho de mídia (Etapa 3+) terá uma interface própria,
baseada em callbacks.

| Função | API |
|---|---|
| Monitores | `CGGetActiveDisplayList`, `CGDisplayCopyDisplayMode`, nomes via `NSScreen.localizedName` |
| Janelas | `SCShareableContent` (exige permissão de Gravação de Tela; nunca dispara o prompt) |
| Microfones | `AVCaptureDevice.DiscoverySession` (`.microphone`, `.external`) |
| Permissões | `CGPreflight/RequestScreenCaptureAccess`, `AVCaptureDevice.authorizationStatus/requestAccess` |
| Screenshot | `SCScreenshotManager.captureImage` (monitor: filtro excluindo o próprio app; janela: `desktopIndependentWindow`) → `CGContext` sRGB BGRA |
| Clipboard | PNG direto no `NSPasteboard` (`fl_copy_png`) |
| Overlays | `NSWindow.level = .screenSaver`, cobre menu bar e Dock, acompanha todos os Spaces |
| Áudio | Sistema: `SCStreamConfiguration.capturesAudio` (48 kHz, `excludesCurrentProcessAudio`). Microfone: `AVCaptureSession` + `AVCaptureAudioDataOutput` (Float32 48 kHz), timestamps convertidos de `synchronizationClock` para o relógio do host. Teste de microfone: medidor de pico independente. |
| Gravação | `SCStream` (NV12, `minimumFrameInterval`, `sourceRect` para região, filtro de janela única) → `AVAssetWriter` H.264/HEVC por VideoToolbox, MP4 fragmentado (2 s), cores BT.709. Eventos para o Rust por callback C. |

As capturas de pixels usam uma interface própria: o Swift preenche um buffer `malloc` BGRA e o Rust
copia e libera com `fl_free_buffer`.

Coordenadas: pontos globais com origem no canto superior esquerdo (o mesmo espaço do ScreenCaptureKit).

## Windows — `windows` crate

| Função | API |
|---|---|
| Monitores | `EnumDisplayMonitors`, `GetMonitorInfoW`, `GetDpiForMonitor`, `EnumDisplaySettingsW`, nomes via `QueryDisplayConfig`/`DisplayConfigGetDeviceInfo` |
| Janelas | `EnumWindows` com filtro estilo Alt-Tab: visível, não minimizada, sem dono, sem `WS_EX_TOOLWINDOW`, não *cloaked* (DWM); limites via `DWMWA_EXTENDED_FRAME_BOUNDS` |
| Microfones | Core Audio `IMMDeviceEnumerator` (endpoints de captura ativos, padrão via `eConsole`) |
| Screenshot | Windows.Graphics.Capture (`CreateForMonitor` / `CreateForWindow`), primeiro quadro copiado para textura *staging* D3D11 (fallback WARP sem GPU) |
| Clipboard | `arboard` (DIB) |
| Áudio | Etapa 4: WASAPI loopback (process loopback no Windows 11 para excluir o próprio app) + captura de microfone, mesmo modelo de mix |
| Overlays | Janelas sem borda no retângulo físico do monitor, `WDA_EXCLUDEFROMCAPTURE` |
| Permissões | Tela: `GraphicsCaptureSession::IsSupported`. Microfone: chaves de privacidade em `ConsentStore\microphone` (global, usuário e "apps da área de trabalho") |

Coordenadas: pixels físicos (o processo é per-monitor DPI aware); `scaleFactor` = DPI / 96.

## Permissões

| | macOS | Windows |
|---|---|---|
| Tela | Prompt do sistema na primeira vez; depois só em Ajustes do Sistema. **Exige reabrir o app** para valer. O macOS não informa "não definido": o FrameLy guarda se já pediu (`UserDefaults`) para distinguir "não definido" de "negado". | Sempre disponível em apps desktop (Windows 10 1903+). |
| Microfone | Prompt do sistema (`NSMicrophoneUsageDescription` no `Info.plist`). | Sem prompt para apps não empacotados: o FrameLy lê o estado e oferece o atalho para Configurações. |

A UI relê permissões e dispositivos sempre que a janela recebe foco (ex.: ao voltar dos Ajustes).
O FrameLy nunca tenta contornar esses mecanismos.

## Assinatura e entitlements (macOS)

O app é assinado com o Hardened Runtime (exigido para notarização). Sob ele, o microfone só funciona
com o entitlement `com.apple.security.device.audio-input` (`src-tauri/Entitlements.plist`). Sem
ele o macOS nega o acesso em silêncio: nenhum aviso aparece e o app nem surge em Ajustes do Sistema ›
Privacidade › Microfone. Gravação de Tela não precisa de entitlement.

## Limitações conhecidas

- `tauri dev` no macOS: as permissões são atribuídas ao app que lançou o processo (Terminal, IDE). Para
  testar o fluxo real use `pnpm app:mac` (app assinado; ver `development.md`).
- A lista de janelas do macOS mostra apenas janelas do Space atual (limitação do ScreenCaptureKit com
  `onScreenWindowsOnly`).
