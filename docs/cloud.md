# FrameLy AI Cloud (`cloud/`)

FrameLy's own service for **summaries and topics** in the cloud, so people don't have to set up
models, keys or accounts. A Cloudflare Worker in TypeScript with no runtime dependencies
(`cloud/src/`, Durable Object `Limiter` for counters). The desktop app sees it as one more provider
(`ProviderId::Cloud`, client in `src-tauri/src/intelligence/cloud.rs`, see [ai.md](ai.md)).

```
FrameLy desktop ──signed HTTPS──▶ FrameLy AI Cloud (Worker) ──▶ configured AI provider
                ◀── validated JSON ──                        ◀──
```

> **Rule:** which AI provider and model process the text is configuration of this service only.
> No API response names them, and the desktop app, its UI strings and the website must never name
> them either. Only this service's code, config and server logs know.

## What it does (and doesn't)

- Receives transcript text (`[mm:ss] text` per line), language, detail level and, for long
  transcripts, the part or the partial summaries to combine. **Builds the prompts itself**
  (`cloud/src/prompts.ts`) — it is not a generic LLM proxy.
- Authenticates the device, applies limits, calls the provider, validates and sanitizes the answer
  (`cloud/src/validate.ts`) and returns the same JSON the app's other providers produce.
- Does **not** receive audio or video, store transcripts or results, or log content (only a short
  device id, task, sizes, latency and status). Stored: device public keys and usage counters.

## Code

| File | Owns |
|---|---|
| `cloud/src/index.ts` | Routing, request flow, error responses |
| `cloud/src/auth.ts`, `cloud/src/crypto.ts` | Proof-of-work registration, Ed25519 request signatures, replay protection |
| `cloud/src/limiter.ts` | Per-device, per-IP and global limits (Durable Object) |
| `cloud/src/config.ts` | Reads `[vars]` and secrets |
| `cloud/src/prompts.ts`, `cloud/src/validate.ts` | Prompts and output validation |
| `cloud/src/providers/` | One file per provider + `mock.ts`; `index.ts::providerFor` picks the active one |
| `cloud/src/errors.ts` | Error codes |
| `cloud/test/` | Vitest (providers simulated; nothing calls a real provider) |

## Endpoints

| Method and route | Use |
|---|---|
| `GET /v1/status` | Availability, data-use policy (`may_train`/`no_training`), public limits. No provider, model, auth or content. |
| `GET /v1/devices/challenge` | Proof-of-work challenge (signed, expires in 10 min). |
| `POST /v1/devices` | `{challenge, publicKey, nonce}` → `{deviceId}`. Registers the device's Ed25519 public key. |
| `POST /v1/ai/summarize` | `{language, detail, transcript, part?}` or `{language, detail, partials}` → `{result: {overview, points[{text,time}]}}` |
| `POST /v1/ai/topics` | `{language, detail, transcript, part?}` → `{result: {topics[…], decisions, actions, dates, numbers, keywords}}` |
| `GET /v1/usage` | Signed: `{usedToday, limitPerDay, resetsIn}` for this device. |

Errors: `{"error": {"code", "message", "retryAfter"?}}` (+ `retry-after` header). Codes:
`invalid_request` 400, `unauthorized` 401, `too_large` 413, `rate_limited`/`daily_limit` 429,
`region_blocked` 451, `provider_error` 502, `service_busy`/`quota_exhausted`/`unavailable` 503. The
app maps them to `AppError::CloudLimit`, `CloudRegion`, `CloudUnavailable`.

### Authentication (no user accounts)

1. The app creates an Ed25519 key pair (private key in the Keychain / Credential Manager).
2. It requests a challenge and finds a `nonce` such that `SHA-256(challenge:publicKey:nonce)` starts
   with `REGISTRATION_POW_BITS` zero bits — cheap once per install, expensive for mass identities.
3. Every AI request is signed over `method\npath\ntimestamp\nrequestId\nsha256(body)` in headers
   `x-framely-device`, `x-framely-timestamp`, `x-framely-request`, `x-framely-signature`. Clock
   off by more than ±5 min or a repeated `requestId` → 401.

Without accounts there is no perfect abuse protection; global ceilings make the worst case
"service busy", never an unbounded bill.

## Configuration

- `cloud/wrangler.toml › [vars]`: active provider (`PROVIDER`), per provider `<P>_MODEL`,
  `<P>_DATA_USE`, `<P>_BLOCKED_COUNTRIES`; timeout, input/output limits, proof-of-work bits,
  per-device / per-IP / global limits.
- Secrets (never in code, app or installer; set with `npx wrangler secret put <NAME>`): the active
  provider's API key (`<P>_API_KEY`) and `TOKEN_SECRET`. Locally they go in `cloud/.dev.vars`
  (git-ignored).
- The app only knows the service URL (`FRAMELY_CLOUD_URL` at build time; repo variable in CI).

| Variable | Default | Effect |
|---|---|---|
| `DEVICE_REQUESTS_PER_MINUTE` / `_PER_DAY` | 6 / 60 | per device (`rate_limited` / `daily_limit`) |
| `IP_REGISTRATIONS_PER_DAY` | 5 | device registrations per IP |
| `GLOBAL_REQUESTS_PER_MINUTE` / `_PER_DAY` | 12 / 900 | service ceiling (`service_busy`) |
| `GLOBAL_TOKENS_PER_MINUTE` | 200 000 | estimated up front, corrected with real usage |
| `MAX_INPUT_CHARS` | 120 000 | per request (the app splits longer transcripts) |
| `MAX_OUTPUT_TOKENS` | 6 000 | model answer |
| `PROVIDER_TIMEOUT_MS` | 60 000 | per request, including up to 2 retries |

When the provider's quota or balance runs out the service returns `quota_exhausted` — it never
switches providers on its own. The consent text in the app follows `dataUse` from `/v1/status`;
when a provider's terms change, update `<P>_DATA_USE` / `<P>_BLOCKED_COUNTRIES` (and get a legal
review before offering it in new regions).

## Run, test, deploy

```bash
cd cloud
pnpm install
printf 'TOKEN_SECRET="local-dev-token-secret-123456"\n' > .dev.vars
pnpm dev:mock          # simulated provider on http://127.0.0.1:8787: no quota or credit used
pnpm test              # vitest: auth, limits, validation, simulated providers (429, quota, 5xx, timeout)
pnpm check             # tsc
```

Desktop against the local service: `FRAMELY_CLOUD_URL=http://127.0.0.1:8787 pnpm tauri dev`.
End-to-end tests from Rust: [development.md](development.md#live-and-end-to-end-tests-ignored-by-default).

Deploy:

```bash
cd cloud
npx wrangler login
npx wrangler secret put <P>_API_KEY
npx wrangler secret put TOKEN_SECRET
pnpm run deploy
```

Then build the desktop app with the published URL (`FRAMELY_CLOUD_URL=… pnpm app:mac` /
`pnpm build:windows`; releases use the repository variable).

Monitoring: `npx wrangler tail` (JSON without content) and the Cloudflare dashboard's Observability.
Keep the global limits within the active provider's quota/budget.

## Change provider or model

- **Model:** edit `<P>_MODEL` in `wrangler.toml` and deploy. No app update needed.
- **Provider:** set `PROVIDER`, make sure its key secret exists, deploy. Nothing changes in the app.
  Try first with `npx wrangler dev --var PROVIDER:<name>`.
- **Add a provider:** `cloud/src/providers/<name>.ts` exporting a `Provider` (takes `system`, `user`,
  `schema`, `maxOutputTokens`; returns text and token counts; throws `ProviderFailure` with
  `quota`, `busy`, `unavailable`, `invalid`, `refused`, `too_long` or `timeout`); add it to
  `providerFor` in `cloud/src/providers/index.ts` and its variables/secret to `cloud/src/config.ts`; set its
  `_DATA_USE` and `_BLOCKED_COUNTRIES`; tests with a simulated provider. The app doesn't change.
