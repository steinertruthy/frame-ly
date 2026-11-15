/**
 * FrameLy Cloud API (Cloudflare Worker): summaries and topics for the FrameLy desktop app.
 *
 *   GET  /v1/status              availability, data-use policy, public limits
 *   GET  /v1/devices/challenge   proof-of-work challenge for registration
 *   POST /v1/devices             register an anonymous device (Ed25519 public key)
 *   POST /v1/ai/summarize        summary of a transcript (or of one part; or combine parts)
 *   POST /v1/ai/topics           topics, decisions, tasks, dates, numbers, keywords
 *   GET  /v1/usage               this device's requests today and its daily limit (signed)
 *
 * The app never learns which AI provider or model is behind the service.
 *
 * Privacy: transcripts exist only in memory for the duration of a request. Nothing is stored
 * besides usage counters and device public keys; logs carry no content.
 */
import { decodePublicKey, deviceIdOf, issueChallenge, checkChallenge, powOk, readSignedHeaders, verifyRequest } from './auth';
import { loadConfig, type Config, type Env } from './config';
import { ApiError } from './errors';
import { durableLimiter, type LimiterClient } from './limiter';
import { combinePrompt, schemaFor, systemPrompt, userPrompt, type Task } from './prompts';
import { ProviderFailure, providerFor } from './providers';
import { normalizeOutput, parseModelJson, parseRequest } from './validate';

export { Limiter } from './limiter';

export interface Deps {
  config: Config;
  limiter: LimiterClient;
  fetch: typeof fetch;
  now: () => number;
  log: (entry: Record<string, unknown>) => void;
}

const MAX_BODY_BYTES = 2_500_000;

function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), { status, headers: { 'content-type': 'application/json', 'cache-control': 'no-store' } });
}

async function readBody(req: Request): Promise<string> {
  const len = Number(req.headers.get('content-length') ?? '0');
  if (len > MAX_BODY_BYTES) throw new ApiError('too_large', 'request body too large');
  const text = await req.text();
  if (text.length > MAX_BODY_BYTES) throw new ApiError('too_large', 'request body too large');
  return text;
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    throw new ApiError('invalid_request', 'body must be JSON');
  }
}

function country(req: Request): string {
  return ((req as unknown as { cf?: { country?: string } }).cf?.country ?? req.headers.get('x-test-country') ?? '').toUpperCase();
}

function clientIp(req: Request): string {
  return req.headers.get('cf-connecting-ip') ?? 'unknown';
}

function status(req: Request, d: Deps): Response {
  const c = d.config;
  const blocked = c.blockedCountries.has(country(req));
  return json({
    service: 'framely-cloud',
    available: !blocked,
    reason: blocked ? 'region_blocked' : null,
    dataUse: c.dataUse,
    limits: { maxInputChars: c.maxInputChars, requestsPerMinute: c.limits.deviceMinute, requestsPerDay: c.limits.deviceDay },
    registrationBits: c.powBits,
  });
}

async function register(req: Request, d: Deps): Promise<Response> {
  const body = parseJson(await readBody(req)) as { challenge?: unknown; publicKey?: unknown; nonce?: unknown };
  if (typeof body.challenge !== 'string' || typeof body.publicKey !== 'string' || typeof body.nonce !== 'string' || body.nonce.length > 64) {
    throw new ApiError('invalid_request', 'challenge, publicKey and nonce are required');
  }
  await checkChallenge(d.config.tokenSecret, body.challenge, d.now());
  const pk = decodePublicKey(body.publicKey);
  if (!(await powOk(body.challenge, body.publicKey, body.nonce, d.config.powBits))) throw new ApiError('unauthorized', 'proof of work not satisfied');
  const deviceId = await deviceIdOf(pk);
  const r = await d.limiter.register(deviceId, body.publicKey, clientIp(req), d.now());
  if (!r.ok) throw new ApiError(r.code === 'unauthorized' ? 'unauthorized' : r.code, 'too many registrations from this network today', r.retryAfter);
  d.log({ event: 'register', device: deviceId.slice(0, 8) });
  return json({ deviceId });
}

/** Signed, read-only: how much of today's limit this device has used. */
async function usage(req: Request, d: Deps): Promise<Response> {
  const headers = readSignedHeaders(req);
  const pk = await d.limiter.publicKey(headers.deviceId);
  if (!pk) throw new ApiError('unauthorized', 'unknown device');
  await verifyRequest(req.method, new URL(req.url).pathname, '', headers, pk, d.now());
  return json(await d.limiter.usage(headers.deviceId, d.now()));
}

async function generate(task: Task, req: Request, d: Deps): Promise<Response> {
  const started = d.now();
  const path = new URL(req.url).pathname;
  const headers = readSignedHeaders(req);
  const raw = await readBody(req);
  const pk = await d.limiter.publicKey(headers.deviceId);
  if (!pk) throw new ApiError('unauthorized', 'unknown device');
  await verifyRequest(req.method, path, raw, headers, pk, d.now());

  const input = parseRequest(task, parseJson(raw), d.config.maxInputChars);
  const system = systemPrompt(input.language);
  const user = input.partials ? combinePrompt(input.detail, input.partials) : userPrompt(task, input.detail, input.transcript!, input.part);
  const maxOutputTokens = d.config.maxOutputTokens;
  const estimated = Math.ceil((system.length + user.length) / 3) + maxOutputTokens;

  const gate = await d.limiter.consume(headers.deviceId, headers.requestId, estimated, d.now());
  if (!gate.ok) {
    const message = gate.code === 'unauthorized' ? 'duplicate request' : gate.code === 'service_busy' ? 'FrameLy Cloud is at capacity' : 'usage limit reached';
    throw new ApiError(gate.code, message, gate.retryAfter);
  }

  const log = { event: task, provider: d.config.provider, device: headers.deviceId.slice(0, 8), inChars: user.length, part: input.part?.index ?? null };
  try {
    const result = await providerFor(d.config)({ system, user, schema: schemaFor(task), maxOutputTokens }, d.config, d.fetch);
    let output: Record<string, unknown>;
    try {
      output = normalizeOutput(task, parseModelJson(result.text));
    } catch (e) {
      throw new ProviderFailure('invalid', `invalid model output: ${(e as Error).message}`);
    }
    await d.limiter.recordTokens(estimated, result.inputTokens + result.outputTokens, d.now());
    d.log({ ...log, status: 200, ms: d.now() - started, inTokens: result.inputTokens, outTokens: result.outputTokens });
    return json({ result: output });
  } catch (e) {
    if (e instanceof ProviderFailure) {
      d.log({ ...log, status: 'provider_failure', kind: e.kind, ms: d.now() - started });
      switch (e.kind) {
        case 'quota':
          throw new ApiError('quota_exhausted', 'FrameLy Cloud reached its usage limit for today', e.retryAfter);
        case 'busy':
          throw new ApiError('service_busy', 'FrameLy Cloud is busy, try again shortly', e.retryAfter);
        case 'unavailable':
        case 'timeout':
          throw new ApiError('unavailable', 'the AI service is temporarily unavailable', 30);
        case 'too_long':
          throw new ApiError('too_large', 'the result would be too long; use a shorter part');
        default:
          throw new ApiError('provider_error', 'the AI service could not process this content');
      }
    }
    throw e;
  }
}

/** Request handling, independent of the Workers runtime (tests call it directly). */
export async function handle(req: Request, d: Deps): Promise<Response> {
  try {
    const url = new URL(req.url);
    const route = `${req.method} ${url.pathname}`;
    if (route === 'GET /v1/status') return status(req, d);
    if (d.config.blockedCountries.has(country(req))) {
      throw new ApiError('region_blocked', 'FrameLy Cloud is not available in this region yet');
    }
    switch (route) {
      case 'GET /v1/devices/challenge':
        return json({ challenge: await issueChallenge(d.config.tokenSecret, d.now()), bits: d.config.powBits });
      case 'POST /v1/devices':
        return await register(req, d);
      case 'POST /v1/ai/summarize':
        return await generate('summary', req, d);
      case 'POST /v1/ai/topics':
        return await generate('topics', req, d);
      case 'GET /v1/usage':
        return await usage(req, d);
      default:
        throw new ApiError('not_found', 'no such endpoint');
    }
  } catch (e) {
    if (e instanceof ApiError) return e.toResponse();
    d.log({ event: 'error', message: (e as Error).message });
    return new ApiError('unavailable', 'unexpected error').toResponse();
  }
}

export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    let config: Config;
    try {
      config = loadConfig(env);
    } catch (e) {
      console.error(JSON.stringify({ event: 'config', message: (e as Error).message }));
      return new ApiError('unavailable', 'service not configured').toResponse();
    }
    return handle(req, {
      config,
      limiter: durableLimiter(env.LIMITER),
      fetch: (input, init) => fetch(input, init),
      now: () => Date.now(),
      log: (entry) => console.log(JSON.stringify(entry)),
    });
  },
};
