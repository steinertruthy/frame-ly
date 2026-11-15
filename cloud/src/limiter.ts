/**
 * Usage limits and device registry. Lives in a single Durable Object (Cloudflare's built-in,
 * strongly consistent state; no separate database): Workers isolates don't share memory, so
 * per-device and global counters need one place to be counted atomically.
 *
 * Stored: device public keys, counters per minute/day (device, IP registrations, global
 * requests and tokens) and recent request ids (replay protection). No content, ever.
 */
import { loadConfig, type Env, type Limits } from './config';

export interface Store {
  get<T>(key: string): Promise<T | undefined>;
  put<T>(key: string, value: T): Promise<void>;
  delete(keys: string[]): Promise<number>;
  list<T>(options: { prefix: string }): Promise<Map<string, T>>;
}

export type LimitOutcome =
  | { ok: true }
  | { ok: false; code: 'rate_limited' | 'daily_limit' | 'service_busy' | 'unauthorized'; retryAfter?: number };

const minuteOf = (now: number) => Math.floor(now / 60_000);
const dayOf = (now: number) => Math.floor(now / 86_400_000);
const REPLAY_TTL_MS = 11 * 60_000;

export class LimiterCore {
  private ops = 0;
  constructor(
    private store: Store,
    private limits: Limits,
  ) {}

  private async count(key: string): Promise<number> {
    return (await this.store.get<number>(key)) ?? 0;
  }

  private async bump(key: string, by = 1): Promise<void> {
    await this.store.put(key, (await this.count(key)) + by);
  }

  async publicKey(deviceId: string): Promise<string | undefined> {
    return (await this.store.get<{ pk: string }>(`dev:${deviceId}`))?.pk;
  }

  /** Registers a device (idempotent for a known key); limited per IP per day. */
  async register(deviceId: string, publicKey: string, ip: string, now: number): Promise<LimitOutcome> {
    if (await this.publicKey(deviceId)) return { ok: true };
    const key = `cnt:ipreg:${ip}:d${dayOf(now)}`;
    if ((await this.count(key)) >= this.limits.ipRegistrationsDay) return { ok: false, code: 'daily_limit', retryAfter: secondsToNextDay(now) };
    await this.bump(key);
    await this.store.put(`dev:${deviceId}`, { pk: publicKey, created: now });
    return { ok: true };
  }

  /** Checks every limit for one AI request and, if all pass, counts it (all-or-nothing). */
  async consume(deviceId: string, requestId: string, estimatedTokens: number, now: number): Promise<LimitOutcome> {
    if (await this.store.get(`req:${deviceId}:${requestId}`)) return { ok: false, code: 'unauthorized' };
    const m = minuteOf(now);
    const d = dayOf(now);
    const keys = {
      devMin: `cnt:dev:${deviceId}:m${m}`,
      devDay: `cnt:dev:${deviceId}:d${d}`,
      gMin: `cnt:g:m${m}`,
      gDay: `cnt:g:d${d}`,
      gTok: `cnt:gtok:m${m}`,
    };
    const toNextMinute = 60 - (now % 60_000) / 1000;
    if ((await this.count(keys.gDay)) >= this.limits.globalDay) return { ok: false, code: 'service_busy', retryAfter: secondsToNextDay(now) };
    if ((await this.count(keys.gMin)) >= this.limits.globalMinute) return { ok: false, code: 'service_busy', retryAfter: toNextMinute };
    if ((await this.count(keys.gTok)) + estimatedTokens > this.limits.globalTokensMinute) return { ok: false, code: 'service_busy', retryAfter: toNextMinute };
    if ((await this.count(keys.devDay)) >= this.limits.deviceDay) return { ok: false, code: 'daily_limit', retryAfter: secondsToNextDay(now) };
    if ((await this.count(keys.devMin)) >= this.limits.deviceMinute) return { ok: false, code: 'rate_limited', retryAfter: toNextMinute };
    await this.store.put(`req:${deviceId}:${requestId}`, now);
    await this.bump(keys.devMin);
    await this.bump(keys.devDay);
    await this.bump(keys.gMin);
    await this.bump(keys.gDay);
    await this.bump(keys.gTok, estimatedTokens);
    if (++this.ops % 50 === 0) await this.prune(now);
    return { ok: true };
  }

  /** Corrects the token estimate with what the provider actually used. */
  async recordTokens(estimated: number, actual: number, now: number): Promise<void> {
    const key = `cnt:gtok:m${minuteOf(now)}`;
    await this.store.put(key, Math.max(0, (await this.count(key)) - estimated + actual));
  }

  /** Drops expired counters and request ids. */
  async prune(now: number): Promise<void> {
    const m = minuteOf(now);
    const d = dayOf(now);
    const stale: string[] = [];
    for (const [k, v] of await this.store.list<number>({ prefix: 'cnt:' })) {
      const min = /:m(\d+)$/.exec(k);
      const day = /:d(\d+)$/.exec(k);
      if ((min && Number(min[1]) < m - 1) || (day && Number(day[1]) < d - 1)) stale.push(k);
      void v;
    }
    for (const [k, t] of await this.store.list<number>({ prefix: 'req:' })) if (now - t > REPLAY_TTL_MS) stale.push(k);
    for (let i = 0; i < stale.length; i += 128) await this.store.delete(stale.slice(i, i + 128));
  }
}

function secondsToNextDay(now: number): number {
  return 86_400 - (now % 86_400_000) / 1000;
}

/** In-memory store for tests and local reasoning. */
export class MemoryStore implements Store {
  data = new Map<string, unknown>();
  async get<T>(key: string) {
    return this.data.get(key) as T | undefined;
  }
  async put<T>(key: string, value: T) {
    this.data.set(key, value);
  }
  async delete(keys: string[]) {
    let n = 0;
    for (const k of keys) if (this.data.delete(k)) n++;
    return n;
  }
  async list<T>({ prefix }: { prefix: string }) {
    return new Map([...this.data].filter(([k]) => k.startsWith(prefix))) as Map<string, T>;
  }
}

/** Calls the limiter, whether it's the Durable Object or an in-process core (tests). */
export interface LimiterClient {
  publicKey(deviceId: string): Promise<string | undefined>;
  register(deviceId: string, publicKey: string, ip: string, now: number): Promise<LimitOutcome>;
  consume(deviceId: string, requestId: string, estimatedTokens: number, now: number): Promise<LimitOutcome>;
  recordTokens(estimated: number, actual: number, now: number): Promise<void>;
}

type Op =
  | { op: 'publicKey'; deviceId: string }
  | { op: 'register'; deviceId: string; publicKey: string; ip: string; now: number }
  | { op: 'consume'; deviceId: string; requestId: string; estimatedTokens: number; now: number }
  | { op: 'recordTokens'; estimated: number; actual: number; now: number };

/** RPC client for the single global Limiter Durable Object. */
export function durableLimiter(ns: DurableObjectNamespace): LimiterClient {
  const stub = ns.get(ns.idFromName('global'));
  const call = async <T>(op: Op): Promise<T> => {
    const r = await stub.fetch('https://limiter/rpc', { method: 'POST', body: JSON.stringify(op) });
    if (!r.ok) throw new Error(`limiter ${r.status}`);
    return (await r.json()) as T;
  };
  return {
    publicKey: async (deviceId) => (await call<{ pk?: string }>({ op: 'publicKey', deviceId })).pk,
    register: (deviceId, publicKey, ip, now) => call({ op: 'register', deviceId, publicKey, ip, now }),
    consume: (deviceId, requestId, estimatedTokens, now) => call({ op: 'consume', deviceId, requestId, estimatedTokens, now }),
    recordTokens: async (estimated, actual, now) => {
      await call({ op: 'recordTokens', estimated, actual, now });
    },
  };
}

/** Durable Object hosting `LimiterCore`. Requests are handled one at a time (input gates). */
export class Limiter {
  private core: LimiterCore | null = null;
  constructor(
    private state: DurableObjectState,
    private env: Env,
  ) {}

  async fetch(req: Request): Promise<Response> {
    this.core ??= new LimiterCore(this.state.storage as unknown as Store, loadConfig(this.env).limits);
    const op = (await req.json()) as Op;
    const c = this.core;
    let out: unknown;
    switch (op.op) {
      case 'publicKey':
        out = { pk: await c.publicKey(op.deviceId) };
        break;
      case 'register':
        out = await c.register(op.deviceId, op.publicKey, op.ip, op.now);
        break;
      case 'consume':
        out = await c.consume(op.deviceId, op.requestId, op.estimatedTokens, op.now);
        break;
      case 'recordTokens':
        await c.recordTokens(op.estimated, op.actual, op.now);
        out = {};
        break;
    }
    return Response.json(out);
  }
}
