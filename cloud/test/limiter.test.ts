import { describe, expect, it } from 'vitest';
import { LimiterCore, MemoryStore } from '../src/limiter';

const limits = { deviceMinute: 2, deviceDay: 10, ipRegistrationsDay: 2, globalMinute: 100, globalDay: 100, globalTokensMinute: 10_000 };

describe('LimiterCore', () => {
  it('counts atomically and only on success; prunes old buckets and request ids', async () => {
    const store = new MemoryStore();
    const l = new LimiterCore(store, limits);
    const t0 = Date.UTC(2026, 10, 15, 10, 0, 0);
    expect(await l.consume('dev1', 'r1', 100, t0)).toEqual({ ok: true });
    expect(await l.consume('dev1', 'r2', 100, t0)).toEqual({ ok: true });
    expect((await l.consume('dev1', 'r3', 100, t0)).ok).toBe(false);
    // A refused request isn't counted against the day.
    expect(store.data.get(`cnt:dev:dev1:d${Math.floor(t0 / 86_400_000)}`)).toBe(2);
    // Token accounting corrected with real usage.
    await l.recordTokens(100, 40, t0);
    expect(store.data.get(`cnt:gtok:m${Math.floor(t0 / 60_000)}`)).toBe(140);
    // Two days later everything old is dropped.
    await l.prune(t0 + 2 * 86_400_000 + 1);
    expect([...store.data.keys()]).toEqual([]);
  });

  it('registration is idempotent per key and limited per IP', async () => {
    const l = new LimiterCore(new MemoryStore(), limits);
    const t = Date.UTC(2026, 10, 15);
    expect(await l.register('a', 'pkA', '1.1.1.1', t)).toEqual({ ok: true });
    expect(await l.register('a', 'pkA', '1.1.1.1', t)).toEqual({ ok: true });
    expect(await l.register('b', 'pkB', '1.1.1.1', t)).toEqual({ ok: true });
    expect((await l.register('c', 'pkC', '1.1.1.1', t)).ok).toBe(false);
    expect(await l.publicKey('a')).toBe('pkA');
    expect(await l.register('c', 'pkC', '1.1.1.1', t + 86_400_000)).toEqual({ ok: true });
  });
});
