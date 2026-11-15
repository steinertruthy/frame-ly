# Arquitetura

## Visão geral

```
UI (Svelte 5, WebView do sistema)  ──IPC tipado──  Core Rust (compartilhado)  ──trait──  Engines nativas
 janela principal, painel da bandeja,               settings, biblioteca, estado,           macOS: Swift (SCK, AVFoundation)
 overlay de seleção, editor                          recuperação, logs, i18n do tray        Windows: Rust (WGC, WASAPI, MF)
```

**Princípio:** o pipeline de mídia (captura → codificação → mux) é 100% nativo por plataforma — é onde
estão desempenho e qualidade. Tudo que não é mídia é compartilhado. Existe uma única abstração de
plataforma (`CaptureBackend`, Etapa 1); não há outras camadas.

## Decisões de stack

| Decisão | Motivo |
|---|---|
| Tauri 2 em vez de Electron | WebView do sistema: ~30–60 MB de RAM em repouso, binário ~10 MB, inicialização rápida. |
| Svelte 5 | Sem virtual DOM, runtime mínimo; o bundle da janela principal tem ~30 KB gzip. |
| Engine macOS em Swift | ScreenCaptureKit/AVFoundation usam delegates, async e blocks; em Swift o código é idiomático e seguro. Ligado ao Rust por uma fronteira C pequena. |
| Engine Windows em Rust (`windows` crate) | Acesso direto a WinRT/COM sem outra linguagem. |
| Sem FFmpeg na v1 | Codificadores de hardware nativos (VideoToolbox, Media Foundation) já produzem MP4 H.264/HEVC + AAC. Evita ~60 MB, licenças GPL/LGPL e processos extras. |
| SQLite para a biblioteca, JSON para ajustes | Consultas rápidas de busca/ordenação; ajustes legíveis e migráveis. |

## Código

| Caminho | Responsabilidade |
|---|---|
| `src-tauri/src/core/` | Lógica sem SO: `errors` (AppError → `{code,message}`), `settings` (versionado, migração, escrita atômica, backup de arquivo corrompido), `paths` (nomes de arquivo), `locale`, `logging`. |
| `src-tauri/src/commands/` | Camada IPC fina: valida, delega ao core, emite eventos. |
| `src-tauri/src/platform/` | Trait `CaptureBackend` e tipos compartilhados; `macos/` (FFI para FrameLyKit) e `windows/` (Win32/WinRT). Ver [platform.md](platform.md). |
| `native/macos/FrameLyKit/` | Engine macOS em Swift (biblioteca estática). |
| `src-tauri/src/windows.rs` | Criação das janelas nativas e aparência por SO (vibrancy no macOS). |
| `src-tauri/src/tray.rs` | Ícone da bandeja/menu bar com ações de captura. |
| `src-tauri/src/screenshot.rs` | Sessão de captura, overlays, entrega (PNG, pasta, clipboard) e prévia. |
| `src-tauri/src/recording.rs` | Fluxo de gravação: alvo, contagem, controle, entrega, recuperação. |
| `src-tauri/src/core/recorder.rs`, `core/video.rs` | Estado/relógio da gravação; tamanho de saída e bitrate (puros, testados). |
| `src-tauri/src/library.rs`, `core/library.rs` | Serviço da biblioteca; índice SQLite puro e testado. |
| `src-tauri/src/editor.rs`, `src/windows/editor/` | Janela do editor (sessão, salvar/copiar) e editor em canvas. |
| `src-tauri/src/intelligence/`, `llm-worker/` | FrameLy AI: transcrição, resumo, tópicos, modelos e provedores (ver abaixo). |
| `src-tauri/src/core/transcript*.rs`, `core/pdf.rs`, `core/resample.rs` | Modelo da transcrição, banco, exportações e reamostragem (puros, testados). |
| `src-tauri/src/protocol.rs` | Protocolo `framely://` para imagens em memória. |
| `src-tauri/src/core/image.rs` | Quadros BGRA, recorte, PNG e BMP (puro, testado). |
| `src/lib/design/tokens.css` | Design tokens (cores, tipografia, raio, movimento) com temas claro/escuro. |
| `src/lib/i18n/` | Dicionários tipados pt-BR/en; uma chave faltando quebra o type-check. |
| `src/lib/ipc/` | Wrappers tipados de comandos e eventos; encaminhamento de erros da UI para o log. |
| `src/lib/stores/` | Estado reativo (runes). A UI só reflete o que o backend salvou de fato. |
| `src/windows/<janela>/` | Um entry HTML por tipo de janela; cada janela carrega só o que usa. |

## Fluxo de screenshot (`src-tauri/src/screenshot.rs`)

```
Tela inteira:  capture_display(monitor sob o cursor) ─────────────────────────────┐
Região/Janela: capture_display(todos, em paralelo) → quadros congelados          │
               → 1 overlay por monitor (overlay.html)                             │
               → overlay carrega o quadro via framely://localhost/frame/<id>     │
               → região: recorte do quadro congelado                              │
                 janela: capture_window nativo (fallback: recorte do quadro) ─────┤
                                                                                   ▼
                     encode PNG → salvar (opcional) → clipboard (opcional) → prévia
```

- **Quadro congelado:** o usuário seleciona sobre a imagem exata que será salva (menus abertos e
  estados de hover são preservados) e a lupa mostra pixels reais.
- **`framely://`:** protocolo próprio que serve imagens em memória (BMP sem compressão para os
  overlays, PNG da última captura para a prévia). Nada vai para arquivos temporários e o asset
  protocol continua desligado.
- **Conversão de coordenadas:** o overlay envia a seleção em pixels CSS + o tamanho do viewport; o
  Rust converte usando o tamanho real do quadro. Funciona igual com pontos (macOS) e DPI (Windows).
- **Janelas do FrameLy nunca aparecem nas capturas:** o filtro do ScreenCaptureKit exclui o próprio
  app (consultado com janelas fora da tela incluídas); no Windows, `WDA_EXCLUDEFROMCAPTURE` em cada
  janela flutuante (overlay, prévia, painel, pílula, contagem, moldura) e a janela principal é
  escondida antes do primeiro quadro de uma gravação.
- **Desempenho (M-series, 3024×1964):** captura ~85 ms; PNG ~95 ms em release; clipboard ~0 ms
  (o PNG já codificado vai direto para o NSPasteboard).

## Fluxo de gravação (`src-tauri/src/recording.rs`)

```
alvo: tela (monitor sob o cursor) | janela/região (mesmo overlay das screenshots, em vermelho)
  → contagem regressiva (janela própria; Esc/clique cancela)
  → FrameLyKit: SCStream → AVAssetWriter (VideoToolbox) → MP4 fragmentado em cache/recordings/<id>.mp4
     + manifesto <id>.json + miniatura <id>.jpg
  → status a cada 500 ms → ícone/tempo na menu bar + banner na tela inicial
  → parar (UI, menu, clique no ícone, CLI, ou o próprio sistema) → arquivo finalizado
  → movido para a pasta de gravações → prévia do vídeo
```

- **Máquina de estados** (`core/recorder.rs`): `Idle → Countdown → Recording ⇄ Paused → Finalizing`,
  com transições inválidas rejeitadas e relógio que desconta pausas (testado).
- **Pausa** não para o stream: as amostras do intervalo pausado são descartadas e as seguintes são
  re-temporizadas (`PauseTimeline.swift`, testado). Retomar é instantâneo.
- **Tamanho e bitrate** (`core/video.rs`): resolução nativa/1080p/720p mantendo proporção, limite
  do codec (H.264 ≤ 4096×2304), dimensões pares, bitrate por bits/pixel/quadro conforme a qualidade.
- **Recuperação:** o MP4 é fragmentado a cada 2 s. Se o FrameLy encerrar inesperadamente, o próximo
  início encontra o manifesto e move o arquivo para a pasta de gravações como "(recovered)" — testado
  com `kill -9` durante a gravação.
- **Parada pelo sistema** (janela fechada, monitor desconectado, botão "Parar" do macOS, permissão
  revogada): o arquivo é finalizado e entregue normalmente.

## Áudio (macOS)

```
ScreenCaptureKit (áudio do sistema, sem o som do próprio FrameLy) ─┐
AVCaptureSession (microfone escolhido, relógio convertido p/ host) ─┤→ AudioMixer → AAC 192 kbps
                                                                     │  (48 kHz estéreo, ganhos,
                     vídeo: primeiro quadro = tempo zero do mix ─────┘   soft-clip, pausa)
```

- **Uma única faixa AAC** com tudo mixado: toca igual em qualquer player (players que só leem a
  primeira faixa não perdem o microfone).
- **Sincronia:** cada buffer é posicionado pelo seu horário no relógio do host — o mesmo do vídeo.
  Jitter de até 10 ms é colado de forma contínua (sem cliques); desvios maiores (deriva do relógio
  do dispositivo) ressincronizam. O offset A/V fica limitado a ~10 ms em gravações de qualquer duração.
- **Fontes lentas ou removidas** nunca travam as outras: a saída anda no máximo 250 ms atrás da fonte
  mais adiantada; um microfone desconectado vira silêncio e a gravação continua.
- **Medição real** (`AVSyncTests`): um tom agendado num horário exato do host aparece no arquivo a
  +6–8 ms do esperado, já considerando a latência de saída do alto-falante (28 ms no MacBook Pro).
- **Permissão e dispositivo:** com o microfone ligado, a permissão é pedida no início da gravação; se
  negada, ou sem microfone, grava sem ele e avisa. Dispositivo escolhido ausente → padrão do sistema.

## Painel rápido e atalhos

- **Painel** (`panel.rs`, `windows/panel/`): janela criada oculta no início (abre instantaneamente),
  ancorada ao ícone da bandeja — abaixo dele na barra de menus, acima numa barra de tarefas inferior —
  e mantida dentro da área útil do monitor. Some ao perder o foco. No macOS é um `NSPanel` não
  ativante com material nativo de popover: abrir o painel não traz a janela principal para frente.
  Clique esquerdo no ícone abre o painel; clique direito abre o menu nativo (alternativa acessível).
- **Atalhos** (`core/shortcuts.rs` + `shortcuts.rs`): modelo puro com normalização, exigência de
  ⌘/Ctrl/Alt (para não sequestrar a digitação), lista de combinações reservadas por SO e detecção de
  duplicatas; registro no SO via `tauri-plugin-global-shortcut`. Se outro app já usa a combinação, a
  falha é reportada por ação e aparece nos Ajustes. Enquanto o usuário grava um novo atalho, os
  atuais ficam suspensos.

## Biblioteca

- **Fonte da verdade são as pastas.** O índice SQLite (`core/library.rs`, em
  `app_data_dir/library.sqlite3`) guarda metadados para busca, filtros e ordenação instantâneos.
  `reconcile` adiciona, atualiza (tamanho/data mudaram) e remove itens comparando com o disco; só
  arquivos novos ou alterados são sondados. Roda no início, ao abrir a biblioteca e ao mudar as pastas.
- **Novas capturas entram na hora:** screenshots e gravações são registradas com os metadados que o
  FrameLy já conhece, sem sondagem.
- **Metadados e miniaturas:** vídeos via AVFoundation (duração, tamanho com rotação; quadro a ~1 s);
  imagens via ImageIO no macOS e `image` no Windows. Miniaturas JPEG (≤ 480 px) geradas sob demanda
  em `framely://localhost/thumb/<id>` e guardadas em cache (chave inclui o tamanho do arquivo).
- **Reprodução:** o visualizador usa o asset protocol do Tauri (suporta *range requests*, necessário
  para `<video>`), com escopo limitado às duas pastas de mídia.
- **Excluir = mover para a Lixeira** (`trash`), sempre com confirmação. Renomear valida nomes que
  funcionam em macOS e Windows e preserva a extensão.

## Edição inline (overlay)

- Depois de selecionar uma região ou janela, o overlay entra em modo de edição sobre o quadro
  congelado: a barra contextual (seta, texto, retângulo, elipse, desfoque, destaque, recorte,
  desfazer/refazer, copiar, salvar, cancelar) aparece abaixo da seleção, acima dela ou dentro dela,
  conforme o espaço. Não há janela de editor separada nesse fluxo.
- O modelo, o histórico e o renderizador são os mesmos do editor (`windows/editor/*`); as formas
  ficam em pixels do quadro, então a exportação sai na resolução real.
- Exportar: `exportPng` gera o PNG e `finish_annotated` o envia como corpo IPC bruto com o
  cabeçalho `x-framely-intent` (`copy` | `save`). O Rust (`screenshot::finish_annotated`) decodifica
  e entrega por `deliver`, que aplica as preferências: `Intent::Copy` também salva se "manter uma
  cópia" estiver ligado; `Save` também copia se configurado; formato PNG ou JPEG (qualidade 50–100).
- Teclas: Enter/⌘C copia, ⌘S salva, Esc cancela. Enquanto um monitor edita, os outros overlays
  ficam bloqueados (`overlay://editing`).
- Gravação de região/janela continua usando apenas a seleção (sem edição).

## Interface e aparência

- `lib/design/tokens.css`: paleta escura (#080D18 → #3978D7), tema claro opcional, tokens de
  movimento zerados quando `data-motion="reduced"` ou quando o sistema pede menos movimento.
- `Aurora.svelte`: luz ambiente com animação só de `transform` (barata para o compositor),
  desligável em Ajustes › Aparência (`data-ambient="off"`).
- Atributos `data-theme`, `data-motion` e `data-ambient` no `<html>` (via `applyAppearance`).
- i18n: dicionários tipados em `lib/i18n/` (pt-BR, en, es, fr, de), `en.ts` define as chaves;
  o teste de completude falha se qualquer idioma ficar sem uma chave. O menu da bandeja usa
  `core/locale.rs` com os mesmos cinco idiomas. Idioma do sistema detectado e persistido.

## Ajustes (schema v2)

- `general`: idioma, iniciar com o sistema (`tauri-plugin-autostart`, abre com `--background`),
  mostrar a janela ao abrir, continuar na bandeja ao fechar, notificações
  (`tauri-plugin-notification`, usadas quando a prévia está desligada).
- Sons (`general.sounds`, ligado por padrão): `core/sounds.rs` sintetiza três sinais curtos (captura,
  início e fim da gravação) em WAV na memória — sem arquivos de áudio no pacote. Tocados via NSSound
  (macOS) e `PlaySoundW` assíncrono (Windows). O áudio do próprio FrameLy é excluído do som do
  sistema, mas o **microfone** capta o que sai dos alto-falantes; por isso nenhum sinal toca com a
  captura ativa: o de início termina antes da gravação começar (é curto, ~0,35 s, e termina junto
  com a contagem; sem contagem, a gravação espera essa duração — o início nativo leva mais ~0,2 s), o de fim toca depois que o arquivo foi fechado, e o de
  screenshot fica mudo durante uma gravação. Verificado medindo o áudio do início dos vídeos.
- Controles flutuantes (`recording.show_controls`, janela `recbar`): tempo, níveis de som, pausar
  e parar, no centro inferior do monitor gravado, arrastáveis, sem foco e fora do vídeo.
- Bandeja durante a gravação: só o ícone muda (vermelho/âmbar) e o tempo vai para a dica. Um
  título de texto alargava o item e, em MacBooks com notch e barra cheia, ele era empurrado para
  trás do notch e sumia.
- `appearance`: tema (padrão escuro), movimento, luz ambiente. `capture`: formato, qualidade JPEG,
  copiar/salvar/prévia. `recording`: alvo padrão, prévia ao terminar, além dos campos anteriores.
- Migração v1 → v2 preserva o tema `system` de quem já tinha o app.
- Armazenamento: `storage_cleanup` remove miniaturas em cache e sobras temporárias, nunca durante
  uma gravação.

## Editor de screenshots

- **Modelo** (`windows/editor/model.ts`, testado): formas em coordenadas da imagem original (seta,
  retângulo, elipse, texto, destaque, desfoque), recorte e tamanho de saída; histórico por snapshots
  para desfazer/refazer; *hit testing* (contorno vs. preenchimento, distância à seta).
- **Renderização única** (`render.ts`): a mesma função desenha a tela e o arquivo exportado, então o
  que se vê é o que se salva. Desfoque = pixelização em blocos (≈ 1/140 da imagem) aplicada na
  exportação — os pixels originais não ficam no arquivo.
- **Segurança:** o editor só abre imagens dentro das pastas do FrameLy; a imagem chega por
  `framely://localhost/edit-source` com CORS (o canvas continua exportável — verificado na abertura).
- **Exportar:** o PNG vai para o Rust como bytes brutos no corpo do IPC (não JSON). "Salvar" nunca
  sobrescreve o original: cria "nome (edited).png" ao lado e o registra na biblioteca. "Copiar" põe
  a imagem na área de transferência.
- Entradas: botão de editar na prévia pós-captura, no visualizador e no menu de contexto da biblioteca.

## FrameLy AI: transcrição, resumo e tópicos (`src-tauri/src/intelligence/`)

```
mídia (gravação, vídeo ou áudio importado)
  → 1 decodificação nativa → PCM 16 kHz mono num temporário privado (AVAssetReader | IMFSourceReader + core/resample)
  → 2 whisper.cpp em blocos de ~5 min cortados no ponto mais silencioso → SQLite a cada bloco (retomável)
  → 7 transcripts.sqlite3 (transcrição ligada ao arquivo por caminho + tamanho + data)
  → 6 geração (prompts fiéis + JSON schema estrito; map-reduce quando não cabe no contexto)
      → 5 provedor: FrameLy AI (3: processo framely-llm, llama.cpp) | Apple Intelligence | Anthropic | OpenAI
  → 8 exportação: TXT, SRT, VTT, Markdown, PDF
4 gerenciador de modelos: catálogo versionado, download retomável (HTTP Range), SHA-256, espaço, reparo
```

- **Dois runtimes, dois processos.** whisper.cpp (`whisper-rs`) roda no app; llama.cpp roda no
  `framely-llm` (`src-tauri/llm-worker`, sidecar do Tauri, JSON por stdin/stdout). Os dois embutem
  versões diferentes do `ggml` (0.9 e 0.26) e, ligados no mesmo binário, o linker mistura os
  símbolos sem erro — comportamento indefinido. O processo separado também devolve a memória do modelo
  ao terminar, cancela na hora (o app encerra o processo) e isola falhas.
- **Modelos** (avaliados numa reunião pt-BR real; detalhes em `intelligence/catalog.rs`): transcrição
  whisper small (Rápida, 190 MB), large-v3-turbo (Equilibrada, 574 MB) e large-v3 com busca em feixe
  (Máxima, 1,08 GB), licença MIT; resumos/tópicos Gemma 4 E2B (Básico, 2,8 GB), Qwen3.5-4B
  (Equilibrado, 3,0 GB) e Qwen3.5-9B (Avançado, 5,7 GB), Apache 2.0. Qwen3.5-2B e Granite 4.2 3B foram
  descartados por inventarem decisões e responsáveis. Arquivos em `<app data>/models`, nunca no instalador.
- **Perfil por hardware** (`hardware.rs`): RAM, Apple Silicon (Metal), AVX2, GPU/VRAM (DXGI, informativo).
  Só a GPU da Apple é usada; no Windows tudo roda na CPU (AVX2 obrigatório, verificado em runtime).
- **Progresso real**: etapas (preparar áudio, carregar modelo, transcrever, gerar, juntar partes),
  percentual vindo do decodificador, do callback do whisper e da avaliação do prompt; tempo restante
  pela vazão medida da própria execução. Carregar o modelo mostra só atividade (não é mensurável).
- **Estimativas** (`estimate.rs`): mediana das últimas execuções nesta máquina (tabela `perf`); sem
  histórico, taxas de referência medidas, marcadas como aproximadas na UI.
- **Retomada e cache**: blocos persistidos; ao reabrir, o que estava rodando vira "interrompida" e
  continua do próximo bloco (o PCM temporário é reaproveitado se completo). O mesmo arquivo com os
  mesmos ajustes nunca é processado de novo.
- **Provedores** (`providers.rs`): cada um declara local/externo, internet, chave, contexto, custo e
  disponibilidade (com o motivo). Anthropic usa a Messages API (`anthropic-version: 2026-10-21`,
  `output_config.format`), OpenAI usa Chat Completions com `json_schema` estrito. Chaves só no Keychain
  (Swift `Keychain.swift`) ou no Gerenciador de Credenciais (`CredWriteW`). Envio externo exige
  provedores externos ligados, chave e consentimento explícito por provedor; o modo exclusivamente local
  (`net.rs`) bloqueia qualquer requisição, inclusive downloads. Não há fallback silencioso.
- **Apple Intelligence** (`Intelligence.swift`): Foundation Models com geração guiada (`@Generable`) no
  mesmo formato dos outros provedores; contexto de 4096 tokens → partes menores. `FoundationModels` é
  weak-linked, então o app segue abrindo no macOS 14/15. Não aparece no Windows.
- **Diarização**: não há separação de falantes local (whisper.cpp só tem `tinydiarize`, inglês) e as
  gravações do FrameLy têm uma faixa mixada; o ajuste aparece indisponível, com explicação.

## Comportamento de ciclo de vida

- Fechar a janela principal a **esconde** (padrão) e o app continua na bandeja; com a opção
  desligada, fechar encerra o app — exceto durante uma gravação, que sempre continua.
- Esconder a janela principal (fechar, iniciar gravação) usa `windows::hide_main`: em tela cheia no
  macOS ela fica invisível (alpha 0, sem animação de janela), sai da tela cheia e é escondida ao fim
  da transição. O usuário vê só o Space deslizando, como ao fechar um app em tela cheia; esconder
  direto deixava o Space vazio e preto.
- Uma segunda instância apenas foca a existente (`tauri-plugin-single-instance`).
- A janela é criada oculta e exibida após o primeiro render (sem flash em branco).

## Erros e logs

- Comandos retornam `AppResult<T>`; a UI traduz `error.<code>`.
- Logs JSON com rotação diária (7 arquivos) em `~/Library/Logs/app.framely.desktop/` ou
  `%LOCALAPPDATA%\app.framely.desktop\logs\`. Nível via `FRAMELY_LOG` (ex.: `FRAMELY_LOG=debug`).
- Erros não tratados de qualquer janela da UI também vão para esse log.

## Segurança

- CSP restritiva; o asset protocol só alcança as pastas de gravações e screenshots.
- A UI não tem permissão genérica de abrir caminhos: `open_folder` só aceita as pastas do FrameLy, e
  as ações da prévia só operam sobre a última captura feita pelo próprio app.
