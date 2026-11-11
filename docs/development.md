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

## Logs

`FRAMELY_LOG=debug pnpm tauri dev` — no modo debug os logs também saem no terminal.

## Windows

O código Windows não pode ser compilado por completo no macOS (`tauri-build` precisa do compilador de
recursos do Windows). Use a VM Windows 11 ARM (Parallels) para rodar e testar, e o CI para x64.

## Assinatura no macOS (sem Developer ID)

A permissão de Gravação de Tela fica vinculada à assinatura do app. Builds sem assinatura estável podem
perder a permissão a cada recompilação. A partir da Etapa 1 (quando a captura começa), o build de dev
será assinado com um certificado local autoassinado fixo; o passo a passo estará aqui. A distribuição
pública exige Developer ID + notarização (Etapa 9).
