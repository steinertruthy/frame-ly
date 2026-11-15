# Desenvolvimento

## Toolchain

- Rust stable via rustup (`rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc` para Windows).
- Node 22 (`.nvmrc`) e pnpm 10 (`corepack enable`).
- macOS: Xcode 15+ (26 para Apple Intelligence). Windows: VS Build Tools com C++ e o runtime WebView2.
- `cmake` e `ninja` (`brew install cmake ninja`): compilam whisper.cpp e llama.cpp.
- O runtime local de IA (`framely-llm`) é um sidecar: `pnpm llm:build` gera
  `src-tauri/binaries/framely-llm-<alvo>` (o `pnpm tauri dev/build` já chama). Sem ele o build do app
  para com uma mensagem explicando.

## Comandos

| Comando | O que faz |
|---|---|
| `pnpm tauri dev` | App com hot reload (UI e Rust). |
| `pnpm check` | `svelte-check` (falha em warnings) + type-check do config. |
| `pnpm test` | Testes Vitest da UI. |
| `cargo test` (em `src-tauri/`) | Testes unitários do core. |
| `cargo clippy --all-targets -- -D warnings` | Lint. |
| `pnpm tauri build` | Pacote de release (.app/.dmg, NSIS). |
| `pnpm check:windows` | Clippy do código Windows a partir do macOS (alvo `x86_64-pc-windows-gnu`; requer `brew install mingw-w64` e `rustup target add x86_64-pc-windows-gnu`). |
| `pnpm app:mac` | Build de debug assinado com a identidade de dev e abre o app (testes de permissão). |
| `swift test` (em `native/macos/FrameLyKit/`) | Testes da engine Swift. |
| `cargo test -- --ignored` | Testes "ao vivo" contra as APIs reais do SO (lista, captura). |
| `cargo test --release -- --ignored --nocapture png_encode_speed` | Mede a codificação PNG de uma tela 3024×1964. |
| `<app>/Contents/MacOS/framely --capture region` | Dispara ações no app aberto (`region`, `window`, `screen`, `cancel`). |
| `<app>/Contents/MacOS/framely --record screen` | Controla gravações (`region`, `window`, `screen`, `pause`, `stop`). |
| `<app>/Contents/MacOS/framely --panel` | Abre/fecha o painel rápido. |
| `<app>/Contents/MacOS/framely --show library` | Abre a janela principal numa página (`home`, `library`, `settings`). |
| `<app>/Contents/MacOS/framely --edit <imagem>` | Abre uma imagem das pastas do FrameLy no editor. |
| `<app>/Contents/MacOS/framely --transcribe <arquivo>` | Abre a transcrição do arquivo (ou a tela de nova transcrição). |
| `cargo test -- --ignored --nocapture end_to_end_with_real_models` | FrameLy AI de ponta a ponta com modelos reais: `FRAMELY_TEST_MEDIA=<áudio>` e `FRAMELY_LLM_WORKER=src-tauri/binaries/framely-llm-<alvo>` (baixa ~3,5 GB na pasta de dados do FrameLy). |
| `cd cloud && pnpm test` | Testes do FrameLy Cloud API (Worker; Gemini simulado). |
| `cd cloud && pnpm dev:mock` | FrameLy Cloud local em `http://127.0.0.1:8787` com provedor simulado. |
| `FRAMELY_CLOUD_URL=http://127.0.0.1:8787 cargo test -- --ignored cloud_round_trip` | Registro do aparelho e pedidos assinados contra o serviço local. |
| `FRAMELY_CLOUD_URL=… FRAMELY_TEST_MEDIA=<arquivo já transcrito> cargo test -- --ignored cloud_generation_end_to_end` | Resumo e tópicos pelo FrameLy Cloud através da fila de jobs do app. |
| `cargo test -- --ignored round_trip` | Grava e apaga uma chave de teste no Keychain/Gerenciador de Credenciais. |

## Logs e depuração

- `FRAMELY_LOG=debug pnpm tauri dev` — no modo debug os logs também saem no terminal.
- `FRAMELY_DEVTOOLS=1 pnpm tauri dev` — mostra a janela imediatamente e abre o Web Inspector.
- Erros não tratados da UI aparecem no log com `"message":"ui"`.
- O servidor de dev escuta em `127.0.0.1:1420` (IPv4 explícito). Com `localhost`, o Node pode escutar
  só em `::1` e o WebKit não carrega a página — a janela fica vazia.
- O WebKit não pinta janelas totalmente encobertas; uma captura de tela de uma janela atrás de outras
  sai vazia mesmo com a UI funcionando.

## Windows

No macOS, `pnpm check:windows` verifica o código Windows pelo alvo GNU (mingw-w64), que traz os
headers C necessários para compilar o SQLite embutido. O build real (MSVC) roda no CI e na VM
Windows 11 ARM (Parallels), onde o app é executado e testado.

### Build de teste do Windows a partir do macOS

```bash
brew install nsis llvm lld && cargo install --locked cargo-xwin
LANG=en_US.UTF-8 XWIN_ACCEPT_LICENSE=1 PATH="/opt/homebrew/opt/llvm/bin:$PATH" pnpm build:windows
```

O build compila whisper.cpp e o `framely-llm` (llama.cpp) com AVX2/FMA/F16C; o script
`src-tauri/cmake/ggml-windows.cmake` contorna o build do `whisper-rs-sys`, que decide pelo sistema
que compila e não pelo alvo. O `cargo-xwin` baixa os headers do Windows SDK/CRT da Microsoft (aceitando a licença deles) e gera
`src-tauri/target/x86_64-pc-windows-msvc/release/framely.exe`; o NSIS empacota o instalador em
`…/release/bundle/nsis/`. O instalador sai sem assinatura (SmartScreen avisa). O `.exe` sozinho também
roda (não há recursos externos), útil como versão portátil — exige o WebView2 Runtime.
`LANG` precisa estar definido: com o locale vazio o `makensis` 3.13 do Homebrew aborta com
`std::bad_alloc` (bug do NSIS no macOS).

## Assinatura no macOS (sem Developer ID)

O macOS vincula as permissões de Gravação de Tela e Microfone à assinatura do app. Para que elas
persistam entre builds, assine sempre com a mesma identidade local:

```bash
./scripts/macos-dev-cert.sh   # uma vez; pede sua senha para confiar no certificado
pnpm app:mac                  # build de debug assinado + abre o FrameLy.app
```

O script cria a identidade autoassinada "FrameLy Development" no seu Keychain de login. Para remover:
`security delete-identity -c "FrameLy Development"`. A distribuição pública exige Developer ID +
notarização (Etapa 9).
