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

Instalador próprio do FrameLy, do início ao fim, sobre o NSIS do Tauri:

- `src-tauri/installer/installer.nsi` — o template do Tauri (tauri-cli 2.12.4) com as páginas
  trocadas pelas telas do FrameLy; toda a lógica de instalação, atualização (`/UPDATE`), modo
  passivo (`/P`), silencioso (`/S`) e desinstalação é a do Tauri (mudanças marcadas `FrameLy:`).
  Ao atualizar o `@tauri-apps/cli`, compare com o template novo.
- `src-tauri/installer/ui.nsh` — as telas, em janela inteira, nas cores do app: boas-vindas (com
  local de instalação, atalho na área de trabalho e o estado: nova instalação, atualização,
  reinstalação ou versão anterior), progresso com barra própria, conclusão com “Abrir o FrameLy
  agora”; no desinstalador, confirmação (com a opção de apagar dados), progresso e conclusão.
  Botões planos e arredondados na cor primária do app; os botões nativos ficam ocultos, mas Enter
  e Esc continuam funcionando. Barra de título escura no Windows 10 2004+/11.
- `installer/lang/*.nsh` — textos do Tauri + do FrameLy nos 5 idiomas do app.
- `installer/hooks.nsh` — encerra o `framely-llm.exe` antes de instalar/desinstalar e, com “apagar
  dados”, remove as credenciais `FrameLy/ai/*`.
- `installer/logo.bmp` — gerado por `node scripts/windows-installer-art.mjs`.

Instalar por cima de uma versão anterior atualiza no lugar (como o updater): ajustes, biblioteca,
transcrições e modelos em `%APPDATA%\app.framely.desktop` são mantidos.

**Telas:** *Actions › Installer screens* compila `installer/preview/preview.nsi` (as mesmas telas
com uma instalação simulada; `/PAGE=` e `/STATE=` abrem cada tela) num runner Windows e publica
screenshots. *Updater end-to-end* também fotografa a tela inicial do instalador real.

**Build local no macOS:** o makensis do Homebrew quebra no macOS 26; use o do Docker:
`ln -s "$PWD/scripts/makensis-docker.sh" ~/bin/makensis` (com `~/bin` primeiro no `PATH`) e
`pnpm build:windows`.

## Atualização automática

O app usa o `tauri-plugin-updater` (`src-tauri/src/updates.rs`):

- lê `https://github.com/steinertruthy/frame-ly/releases/latest/download/latest.json`;
- verifica 45 s depois de abrir e a cada 6 h (desligável em Ajustes › Sobre; nunca com o modo
  “Exclusivamente local”); `framely --update` verifica e instala na hora;
- só baixa quando a pessoa escolhe “Atualizar e reiniciar”; a assinatura do pacote é conferida
  com a chave pública em `tauri.conf.json` (`plugins.updater.pubkey`);
- nunca interrompe uma gravação: com uma gravação em andamento, instala quando ela termina;
- macOS: substitui o `.app` e reinicia; Windows: roda o instalador NSIS em modo passivo
  (`/UPDATE`), que atualiza no lugar, mantém os dados e reabre o FrameLy.

### Chaves de assinatura

- Pública: `src-tauri/tauri.conf.json` › `plugins.updater.pubkey`.
- Privada e senha: **nunca no git**. Ficam nos secrets do repositório `TAURI_SIGNING_PRIVATE_KEY`
  e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, e numa cópia local fora do repositório
  (`~/.tauri/framely-updater.key` e `~/.tauri/framely-updater.password`). Sem a chave privada não é
  possível publicar atualizações que as versões instaladas aceitem — guarde um backup seguro.
- O workflow de release falha se os secrets não existirem, e só publica se `latest.json` tiver o
  pacote e a assinatura das duas plataformas (`scripts/release/latest-json.mjs`).

### Arquivos extras de cada release

`FrameLy_X.Y.Z_aarch64.app.tar.gz` + `.sig` (macOS), `FrameLy_X.Y.Z_x64-setup.exe.sig` (Windows) e
`latest.json`.

### Teste de ponta a ponta

*Actions › Updater end-to-end* (também roda em PRs que mexem no updater): gera uma chave
descartável, compila uma versão 0.0.1 e uma 0.0.2 assinada, serve o `latest.json` em
`127.0.0.1`, roda `framely --update` na versão antiga e confere que ela baixou, verificou,
instalou e reabriu já na 0.0.2. No Windows também confere que os dados do usuário sobrevivem à
atualização e que a desinstalação é limpa.

### Limitações

- Quem tem uma versão sem o updater (0.1.0 e 0.2.0) precisa baixar manualmente, uma vez, a
  primeira versão que o inclui.
- macOS sem Developer ID/notarização: a assinatura *ad hoc* muda a cada versão, então o macOS
  pode pedir de novo a permissão de Gravação de Tela (e Microfone) depois de atualizar.
