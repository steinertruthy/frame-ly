# FrameLy Cloud API

Serviço próprio do FrameLy para gerar **resumos e tópicos** na nuvem, sem que o usuário precise
configurar modelos, chaves ou contas. É um Cloudflare Worker em TypeScript, sem dependências de
runtime. O Desktop vê o FrameLy Cloud como mais um provedor (`ProviderId::Cloud`); quem processa o
texto por trás dele (hoje, Google Gemini) é configuração deste serviço.

```
FrameLy Desktop ──HTTPS assinado──▶ FrameLy Cloud API (Worker) ──▶ Gemini (generateContent)
                ◀── JSON validado ──                            ◀──
```

## O que o serviço faz (e não faz)

- Recebe a transcrição em texto (`[mm:ss] texto` por linha), o idioma, o nível de detalhe e, para
  transcrições longas, a parte ou os resumos parciais a combinar. **Monta os prompts aqui** — o
  serviço não é um proxy genérico de modelo de linguagem.
- Autentica o aparelho, aplica limites, chama o provedor, valida e higieniza a resposta e devolve
  o mesmo JSON que os outros provedores do app produzem.
- **Não** recebe áudio nem vídeo, **não** guarda transcrições nem resultados, **não** registra
  conteúdo nos logs (só aparelho abreviado, tarefa, tamanhos, latência e status). O que é gravado:
  chaves públicas dos aparelhos e contadores de uso (Durable Object `Limiter`).

## Endpoints

| Método e rota | Uso |
|---|---|
| `GET /v1/status` | Disponibilidade, processador (`Google Gemini`), política de uso de dados, limites públicos. Sem autenticação e sem conteúdo. |
| `GET /v1/devices/challenge` | Desafio de prova de trabalho (assinado, expira em 10 min). |
| `POST /v1/devices` | `{challenge, publicKey, nonce}` → `{deviceId}`. Registra a chave pública Ed25519 do aparelho. |
| `POST /v1/ai/summarize` | `{language, detail, transcript, part?}` ou `{language, detail, partials}` → `{result: {overview, points[{text,time}]}}` |
| `POST /v1/ai/topics` | `{language, detail, transcript, part?}` → `{result: {topics[{title,time,subtopics[{title,time,points}]}], decisions, actions, dates, numbers, keywords}}` |

Erros: `{"error": {"code", "message", "retryAfter"?}}` com `retry-after` quando faz sentido.
Códigos: `invalid_request` 400, `unauthorized` 401, `too_large` 413, `rate_limited`/`daily_limit`
429, `region_blocked` 451, `provider_error` 502, `service_busy`/`quota_exhausted`/`unavailable` 503.

### Autenticação (sem contas de usuário)

1. O app cria um par de chaves Ed25519 (privada no Keychain / Gerenciador de Credenciais).
2. Pede um desafio e procura um `nonce` tal que `SHA-256(challenge:publicKey:nonce)` comece com
   `REGISTRATION_POW_BITS` bits zero — barato uma vez por instalação, caro para quem tenta criar
   milhares de identidades.
3. Cada pedido de IA é assinado: `método\ncaminho\ntimestamp\nrequestId\nsha256(body)` nos
   cabeçalhos `x-framely-device`, `x-framely-timestamp`, `x-framely-request`,
   `x-framely-signature`. Relógio fora de ±5 min ou `requestId` repetido → 401.

Sem contas não existe proteção perfeita contra abuso; os tetos globais garantem que o pior caso seja
"serviço ocupado", nunca uma cobrança.

## Configuração (tudo centralizado)

- `wrangler.toml` › `[vars]`: provedor ativo, modelo, timeout, limites de entrada/saída, prova de
  trabalho, limites por aparelho / IP / globais, países bloqueados, política de uso de dados.
- Segredos (nunca no código, no app ou no instalador): `GEMINI_API_KEY`, `TOKEN_SECRET`.
- O app só conhece a URL do serviço (`FRAMELY_CLOUD_URL`, definida no build do Desktop).

## Chave do Gemini

1. Entre em <https://aistudio.google.com> com a conta Google do projeto e crie uma chave de API.
2. `cd cloud && npx wrangler secret put GEMINI_API_KEY` (cole a chave quando pedir).
3. `npx wrangler secret put TOKEN_SECRET` com um valor aleatório longo (ex.: `openssl rand -hex 32`).

**Plano gratuito (atual):** pelos termos da Gemini API (23/03/2026), no uso gratuito o Google pode
usar o conteúdo e as respostas para melhorar seus produtos e revisores humanos podem lê-los; os
termos pedem para não enviar dados sensíveis. Por isso `DATA_USE = "may_train"` e o app avisa isso
antes do primeiro envio. O plano gratuito não pode ser oferecido a usuários do EEE, Suíça e Reino
Unido: `BLOCKED_COUNTRIES` recusa esses países (`cf.country`).

**Migrar para o plano pago:** ative o faturamento no projeto do Google Cloud da chave, troque
`DATA_USE` para `"no_training"`, reveja os limites globais e (opcionalmente) esvazie
`BLOCKED_COUNTRIES`. Nada muda no Desktop: o texto de consentimento vem de `/v1/status`.

## Execução local

```bash
cd cloud
pnpm install
printf 'TOKEN_SECRET="local-dev-token-secret-123456"\n' > .dev.vars
pnpm dev:mock          # provedor simulado: não usa a cota do Gemini
# ou: GEMINI_API_KEY no .dev.vars + `pnpm dev` para chamar o Gemini de verdade
```

Desktop apontando para o serviço local:

```bash
FRAMELY_CLOUD_URL=http://127.0.0.1:8787 pnpm tauri dev
```

## Deploy

```bash
cd cloud
npx wrangler login
npx wrangler secret put GEMINI_API_KEY
npx wrangler secret put TOKEN_SECRET
pnpm run deploy        # publica em https://framely-cloud.<sua-conta>.workers.dev
```

Depois, gere o Desktop com a URL publicada (fica embutida no binário):

```bash
FRAMELY_CLOUD_URL=https://framely-cloud.<sua-conta>.workers.dev pnpm app:mac
FRAMELY_CLOUD_URL=https://framely-cloud.<sua-conta>.workers.dev pnpm build:windows
```

Um domínio próprio (ex.: `cloud.framely.app`) pode ser ligado ao Worker em Workers & Pages ›
framely-cloud › Settings › Domains.

## Monitoramento

- `npx wrangler tail` mostra os logs em tempo real (JSON sem conteúdo: `event`, `device` abreviado,
  `inChars`, `inTokens`, `outTokens`, `ms`, `status`/`kind`).
- Painel Cloudflare › Workers › framely-cloud › Observability: erros, latência, volume.
- Cota do Gemini: <https://aistudio.google.com/rate-limit> (a cota é por projeto, não por chave).
  Mantenha `GLOBAL_REQUESTS_PER_DAY`/`PER_MINUTE` e `GLOBAL_TOKENS_PER_MINUTE` abaixo dela.

## Limites

| Variável | Padrão | Efeito |
|---|---|---|
| `DEVICE_REQUESTS_PER_MINUTE` / `_PER_DAY` | 6 / 60 | por aparelho (`rate_limited` / `daily_limit`) |
| `IP_REGISTRATIONS_PER_DAY` | 5 | registros de aparelho por IP |
| `GLOBAL_REQUESTS_PER_MINUTE` / `_PER_DAY` | 12 / 900 | teto do serviço (`service_busy`) |
| `GLOBAL_TOKENS_PER_MINUTE` | 200 000 | estimado antes, corrigido com o uso real |
| `MAX_INPUT_CHARS` | 120 000 | por pedido (o app divide transcrições maiores) |
| `MAX_OUTPUT_TOKENS` | 6 000 | resposta do modelo |
| `PROVIDER_TIMEOUT_MS` | 60 000 | por pedido, incluindo até 2 novas tentativas |

Quando o provedor esgota a cota diária o serviço devolve `quota_exhausted` — nunca troca para um
provedor pago. Alterar limites: edite `wrangler.toml` e `pnpm deploy`.

## Trocar o modelo

Edite `GEMINI_MODEL` em `wrangler.toml` e publique. Modelos e prazos de descontinuação:
<https://aistudio.google.com/docs/models> (ex.: `gemini-2.5-flash` passa a ser redirecionado para
`gemini-3.5-flash-lite` em 1/12/2026). Teste antes com `pnpm dev` e uma transcrição de exemplo.

## Adicionar um provedor (Groq, OpenRouter, Workers AI, OpenAI, Anthropic…)

1. Crie `src/providers/<nome>.ts` exportando um `Provider` (recebe `system`, `user`, `schema`,
   `maxOutputTokens`; devolve `text` e tokens; lança `ProviderFailure` com `quota`, `busy`,
   `unavailable`, `invalid`, `refused`, `too_long` ou `timeout`).
2. Adicione o caso em `providerFor` (`src/providers/index.ts`) e a leitura das variáveis/segredos em
   `src/config.ts`.
3. Ajuste `DATA_USE` conforme os termos do novo provedor. O Desktop não muda.

## Testes

```bash
pnpm test     # vitest: autenticação, limites, validação, Gemini simulado (429, cota, 5xx, timeout)
pnpm check    # tsc
```

Nenhum teste chama o Gemini de verdade. Teste de ponta a ponta com o Desktop: ver
`docs/development.md` (`cloud_round_trip` e `cloud_generation_end_to_end`).
