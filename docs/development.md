# Desenvolvimento

## Toolchain

- Rust stable via rustup (`rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc` para Windows).
- Node 22 (`.nvmrc`) e pnpm 10 (`corepack enable`).
- macOS: Xcode 15+. Windows: VS Build Tools com C++ e o runtime WebView2.

## Comandos

| Comando | O que faz |
|---|---|
| `pnpm tauri dev` | App com hot reload (UI e Rust). |
| `pnpm check` | `svelte-check` (falha em warnings) + type-check do config. |
| `pnpm test` | Testes Vitest da UI. |
| `cargo test` (em `src-tauri/`) | Testes unitários do core. |
| `cargo clippy --all-targets -- -D warnings` | Lint. |
| `pnpm tauri build` | Pacote de release (.app/.dmg, NSIS). |
| `pnpm check:windows` | Clippy do código Windows a partir do macOS (requer `brew install llvm`). |
| `pnpm app:mac` | Build de debug assinado com a identidade de dev e abre o app (testes de permissão). |
| `swift test` (em `native/macos/FrameLyKit/`) | Testes da engine Swift. |
| `cargo test -- --ignored` | Testes "ao vivo" contra as APIs reais do SO (lista, captura). |
| `cargo test --release -- --ignored --nocapture png_encode_speed` | Mede a codificação PNG de uma tela 3024×1964. |
| `<app>/Contents/MacOS/framely --capture region` | Dispara ações no app aberto (`region`, `window`, `screen`, `cancel`). |

## Logs e depuração

- `FRAMELY_LOG=debug pnpm tauri dev` — no modo debug os logs também saem no terminal.
- `FRAMELY_DEVTOOLS=1 pnpm tauri dev` — mostra a janela imediatamente e abre o Web Inspector.
- Erros não tratados da UI aparecem no log com `"message":"ui"`.
- O servidor de dev escuta em `127.0.0.1:1420` (IPv4 explícito). Com `localhost`, o Node pode escutar
  só em `::1` e o WebKit não carrega a página — a janela fica vazia.
- O WebKit não pinta janelas totalmente encobertas; uma captura de tela de uma janela atrás de outras
  sai vazia mesmo com a UI funcionando.

## Windows

O código Windows não pode ser compilado por completo no macOS (`tauri-build` precisa do compilador de
recursos do Windows). Use a VM Windows 11 ARM (Parallels) para rodar e testar, e o CI para x64.

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
