import { handle, type Deps } from '../src/index';
import { loadConfig, type Env } from '../src/config';
import { LimiterCore, MemoryStore, type LimiterClient } from '../src/limiter';
import { leadingZeroBits, sha256, toBase64 } from '../src/crypto';
import { signingString } from '../src/auth';

export const SECRET_KEY = 'AIza-test-secret-key-never-exposed';

export function env(overrides: Partial<Env> = {}): Env {
  return {
    LIMITER: undefined as unknown as DurableObjectNamespace,
    PROVIDER: 'mock',
    GEMINI_MODEL: 'gemini-test-flash',
    DATA_USE: 'may_train',
    PROVIDER_TIMEOUT_MS: '5000',
    MAX_INPUT_CHARS: '20000',
    MAX_OUTPUT_TOKENS: '1000',
    REGISTRATION_POW_BITS: '6',
    DEVICE_REQUESTS_PER_MINUTE: '3',
    DEVICE_REQUESTS_PER_DAY: '5',
    IP_REGISTRATIONS_PER_DAY: '3',
    GLOBAL_REQUESTS_PER_MINUTE: '100',
    GLOBAL_REQUESTS_PER_DAY: '1000',
    GLOBAL_TOKENS_PER_MINUTE: '1000000',
    BLOCKED_COUNTRIES: 'DE,GB,CH',
    GEMINI_API_KEY: SECRET_KEY,
    TOKEN_SECRET: 'test-token-secret-0123456789',
    ...overrides,
  };
}

export interface Harness {
  deps: Deps;
  logs: Record<string, unknown>[];
  now: { t: number };
  call: (method: string, path: string, init?: { body?: unknown; headers?: Record<string, string> }) => Promise<Response>;
}

export function harness(e: Env = env(), fetchImpl: typeof fetch = (() => Promise.reject(new Error('no network'))) as typeof fetch): Harness {
  const config = loadConfig(e);
  const core = new LimiterCore(new MemoryStore(), config.limits);
  const limiter: LimiterClient = core;
  const logs: Record<string, unknown>[] = [];
  const now = { t: Date.UTC(2026, 10, 15, 12, 0, 0) };
  const deps: Deps = { config, limiter, fetch: fetchImpl, now: () => now.t, log: (x) => logs.push(x) };
  return {
    deps,
    logs,
    now,
    call: (method, path, init = {}) =>
      handle(
        new Request(`https://cloud.test${path}`, {
          method,
          headers: { 'content-type': 'application/json', 'cf-connecting-ip': '203.0.113.7', ...init.headers },
          body: init.body === undefined ? undefined : typeof init.body === 'string' ? init.body : JSON.stringify(init.body),
        }),
        deps,
      ),
  };
}

export interface Device {
  deviceId: string;
  sign: (method: string, path: string, body: string, opts?: { timestamp?: number; requestId?: string }) => Promise<Record<string, string>>;
}

/** Registers a device like the desktop app does: key pair, proof of work, registration. */
export async function registerDevice(h: Harness, ip = '203.0.113.7'): Promise<Device> {
  const keys = (await crypto.subtle.generateKey({ name: 'Ed25519' }, true, ['sign', 'verify'])) as CryptoKeyPair;
  const publicKey = toBase64(new Uint8Array((await crypto.subtle.exportKey('raw', keys.publicKey)) as ArrayBuffer));
  const { challenge, bits } = (await (await h.call('GET', '/v1/devices/challenge')).json()) as { challenge: string; bits: number };
  let nonce = 0;
  while (leadingZeroBits(await sha256(`${challenge}:${publicKey}:${nonce}`)) < bits) nonce++;
  const r = await h.call('POST', '/v1/devices', { body: { challenge, publicKey, nonce: String(nonce) }, headers: { 'cf-connecting-ip': ip } });
  if (r.status !== 200) throw new Error(`register ${r.status} ${await r.text()}`);
  const { deviceId } = (await r.json()) as { deviceId: string };
  let seq = 0;
  return {
    deviceId,
    async sign(method, path, body, opts = {}) {
      const timestamp = String(opts.timestamp ?? h.now.t);
      const requestId = opts.requestId ?? `0000000000000000${(seq++).toString(16).padStart(8, '0')}`;
      const msg = await signingString(method, path, timestamp, requestId, body);
      const sig = new Uint8Array(await crypto.subtle.sign({ name: 'Ed25519' }, keys.privateKey, new TextEncoder().encode(msg)));
      return { 'x-framely-device': deviceId, 'x-framely-timestamp': timestamp, 'x-framely-request': requestId, 'x-framely-signature': toBase64(sig) };
    },
  };
}

export async function signedCall(h: Harness, dev: Device, path: string, body: unknown, opts?: { timestamp?: number; requestId?: string; country?: string }) {
  const raw = JSON.stringify(body);
  const headers = await dev.sign('POST', path, raw, opts);
  if (opts?.country) headers['x-test-country'] = opts.country;
  return h.call('POST', path, { body: raw, headers });
}

export const TRANSCRIPT = '[00:00] Bom dia a todos.\n[00:40] O lançamento será no dia 15 de dezembro.\n[01:50] Criar o plano anual por 290 reais.';
