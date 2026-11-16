# Versionamento e releases

## SemVer

O FrameLy segue o [Versionamento Semântico](https://semver.org/lang/pt-BR/): `MAJOR.MINOR.PATCH`.
Enquanto a versão for `0.y.z`, mudanças de comportamento podem acontecer em versões `MINOR`.
Pré-lançamentos usam sufixo (`0.2.0-beta.1`) e saem como *pre-release* no GitHub.

A versão fica em três arquivos, que precisam concordar (o CI confere):

- `package.json`
- `src-tauri/Cargo.toml` (`[package] version`)
- `src-tauri/tauri.conf.json` (a versão exibida em Ajustes › Sobre)

## Publicar uma versão

1. Atualize a versão nos três arquivos.
2. Em `CHANGELOG.md`, mova o que estiver em `[Unreleased]` para `## [X.Y.Z] - AAAA-MM-DD` e
   atualize os links do rodapé. Traduza a seção em `website/src/content/changelog/<idioma>.md`
   (opcional: sem tradução, o site mostra o texto em inglês).
3. Confira localmente: `node scripts/release/check-version.mjs vX.Y.Z`.
4. Commit, `git tag -a vX.Y.Z -m "FrameLy X.Y.Z"` e `git push origin main vX.Y.Z`.

O workflow **Release** (`.github/workflows/release.yml`) então:

1. valida a tag, os três arquivos e o changelog, e falha se já existir um release com essa tag
   (releases nunca são sobrescritos — corrija com uma nova versão);
2. compila os instaladores em runners próprios: `macos-15` → `FrameLy_X.Y.Z_aarch64.dmg`,
   `windows-latest` → `FrameLy_X.Y.Z_x64-setup.exe` (NSIS);
3. só se **todos** os builds passarem, gera `SHA256SUMS.txt` e cria o release com as notas do
   changelog;
4. chama o workflow **Website**, que lê a API de Releases e republica a área de download.

Para testar os builds sem publicar nada: *Actions › Release › Run workflow* (os instaladores ficam
como artefatos da execução).

## Site

`website/` é publicado no GitHub Pages pelo workflow **Website** a cada push em `main` que toque o
site ou o changelog, e depois de cada release. Os dados de download (`website/src/data/releases.json`)
são gerados no build a partir da API do GitHub, então o site nunca aponta para arquivos que não
existem. Veja [website/README.md](../website/README.md).

## Instalador do Windows (NSIS)

Usa o template oficial do Tauri com a identidade do FrameLy (`src-tauri/tauri.conf.json` ›
`bundle.windows.nsis`), sem template próprio, para manter o fluxo de atualização do Tauri:

- `src-tauri/installer/sidebar.bmp` (164×314) e `header.bmp` (150×57), gerados por
  `node scripts/windows-installer-art.mjs` a partir de `assets/logo.svg`;
- ícone do app no instalador e no desinstalador;
- os 5 idiomas do app, escolhidos pelo idioma do Windows (`installer/lang/*.nsh`: textos do Tauri
  + textos do FrameLy nas telas de boas-vindas e de conclusão);
- `installer/hooks.nsh`: encerra o `framely-llm.exe` antes de instalar/desinstalar e, se a pessoa
  marcar “apagar dados” ao desinstalar, remove também as credenciais `FrameLy/ai/*`.

Instalar por cima de uma versão anterior atualiza no lugar: o desinstalador antigo roda em modo
silencioso e nunca apaga `%APPDATA%\app.framely.desktop` (ajustes, biblioteca, transcrições e
modelos baixados). Atalho no Menu Iniciar sempre; atalho na área de trabalho e “Abrir o FrameLy
agora” como opções na última tela.

Ao atualizar o `@tauri-apps/cli`, compare `installer/lang/*.nsh` com os arquivos de idioma do
Tauri (novas chaves precisam ser copiadas).
