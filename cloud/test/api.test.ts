import { describe, expect, it } from 'vitest';
import { DEEPSEEK_KEY, env, harness, registerDevice, signedCall, SECRET_KEY, TRANSCRIPT } from './helpers';

const geminiOk = (payload: unknown, usage = { promptTokenCount: 100, candidatesTokenCount: 50 }) =>
  new Response(JSON.stringify({ candidates: [{ content: { parts: [{ text: JSON.stringify(payload) }] }, finishReason: 'STOP' }], usageMetadata: usage }), { status: 200 });

describe('status and registration', () => {
  it('reports availability and data use without naming the provider or model; blocks regions', async () => {
    const h = harness(env({ PROVIDER: 'deepseek', DEEPSEEK_BLOCKED_COUNTRIES: 'DE' }));
    const raw = await (await h.call('GET', '/v1/status')).text();
    expect(JSON.parse(raw)).toMatchObject({ available: true, dataUse: 'may_train' });
    expect(raw.toLowerCase()).not.toMatch(/deepseek|gemini|model|processor/);
    const de = (await (await h.call('GET', '/v1/status', { headers: { 'x-test-country': 'de' } })).json()) as Record<string, unknown>;
    expect(de).toMatchObject({ available: false, reason: 'region_blocked' });
    expect((await h.call('GET', '/v1/devices/challenge', { headers: { 'x-test-country': 'DE' } })).status).toBe(451);
    // Region rules follow the active provider: DeepSeek blocks only what its list says.
    expect((await h.call('GET', '/v1/devices/challenge', { headers: { 'x-test-country': 'GB' } })).status).toBe(200);
    const g = harness(env({ PROVIDER: 'gemini' }));
    expect((await g.call('GET', '/v1/devices/challenge', { headers: { 'x-test-country': 'GB' } })).status).toBe(451);
  });

  it('requires a valid challenge and proof of work', async () => {
    const h = harness(env({ REGISTRATION_POW_BITS: '16' }));
    const { challenge } = (await (await h.call('GET', '/v1/devices/challenge')).json()) as { challenge: string };
    const pk = btoa(String.fromCharCode(...new Uint8Array(32).fill(7)));
    expect((await h.call('POST', '/v1/devices', { body: { challenge, publicKey: pk, nonce: '0' } })).status).toBe(401);
    const forged = challenge.replace(/\.[0-9a-f]+$/, '.00');
    expect((await h.call('POST', '/v1/devices', { body: { challenge: forged, publicKey: pk, nonce: '0' } })).status).toBe(401);
    h.now.t += 11 * 60_000;
    expect((await h.call('POST', '/v1/devices', { body: { challenge, publicKey: pk, nonce: '0' } })).status).toBe(401);
  });

  it('limits registrations per network per day', async () => {
    const h = harness();
    for (let i = 0; i < 3; i++) await registerDevice(h, '198.51.100.1');
    await expect(registerDevice(h, '198.51.100.1')).rejects.toThrow(/register 429/);
    await registerDevice(h, '198.51.100.2');
  });
});

describe('signed requests', () => {
  it('summarizes and organizes topics through the provider (mock)', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    const r = await signedCall(h, dev, '/v1/ai/summarize', { language: 'pt', detail: 'brief', transcript: TRANSCRIPT });
    expect(r.status).toBe(200);
    const body = (await r.json()) as { result: { overview: string; points: { text: string; time: string }[] } };
    expect(body.result.overview).toContain('Bom dia');
    expect(body.result.points[1]).toEqual({ text: 'O lançamento será no dia 15 de dezembro.', time: '00:40' });
    const t = (await (await signedCall(h, dev, '/v1/ai/topics', { language: 'pt', transcript: TRANSCRIPT })).json()) as { result: { topics: unknown[]; keywords: unknown[] } };
    expect(t.result.topics.length).toBe(3);
  });

  it('combines partial summaries', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    const partials = [
      { overview: 'Parte 1', points: [{ text: 'A', time: '00:10' }] },
      { overview: 'Parte 2', points: [{ text: 'B', time: '05:10' }] },
    ];
    const r = await signedCall(h, dev, '/v1/ai/summarize', { language: 'pt', detail: 'balanced', partials });
    expect(r.status).toBe(200);
    expect(((await r.json()) as { result: { points: { time: string }[] } }).result.points.map((p) => p.time)).toEqual(['00:10', '05:10']);
  });

  it('rejects unknown devices, bad or stale signatures and replays', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    const body = { transcript: TRANSCRIPT };
    const raw = JSON.stringify(body);
    const headers = await dev.sign('POST', '/v1/ai/summarize', raw);
    // Body changed after signing.
    expect((await h.call('POST', '/v1/ai/summarize', { body: { transcript: 'outro' }, headers })).status).toBe(401);
    // Signed for another endpoint.
    const other = await dev.sign('POST', '/v1/ai/topics', raw);
    expect((await h.call('POST', '/v1/ai/summarize', { body: raw, headers: other })).status).toBe(401);
    // Stale timestamp.
    expect((await signedCall(h, dev, '/v1/ai/summarize', body, { timestamp: h.now.t - 6 * 60_000 })).status).toBe(401);
    // Replay of an accepted request.
    expect((await signedCall(h, dev, '/v1/ai/summarize', body, { requestId: 'aaaaaaaaaaaaaaaa1' })).status).toBe(200);
    expect((await signedCall(h, dev, '/v1/ai/summarize', body, { requestId: 'aaaaaaaaaaaaaaaa1' })).status).toBe(401);
    // Unknown device / missing headers.
    expect((await h.call('POST', '/v1/ai/summarize', { body })).status).toBe(401);
    const forgedDevice = { ...headers, 'x-framely-device': 'f'.repeat(32) };
    expect((await h.call('POST', '/v1/ai/summarize', { body: raw, headers: forgedDevice })).status).toBe(401);
  });

  it('validates input and size', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { detail: 'huge', transcript: TRANSCRIPT })).status).toBe(400);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { language: 'Português', transcript: TRANSCRIPT })).status).toBe(400);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: '' })).status).toBe(400);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: 'x'.repeat(20_001) })).status).toBe(413);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT, part: { index: 3, count: 2 } })).status).toBe(400);
  });
});

describe('limits', () => {
  it('per device per minute and per day', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    const ok = () => signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    for (let i = 0; i < 3; i++) expect((await ok()).status).toBe(200);
    const limited = await ok();
    expect(limited.status).toBe(429);
    expect(((await limited.json()) as { error: { code: string } }).error.code).toBe('rate_limited');
    expect(Number(limited.headers.get('retry-after'))).toBeGreaterThan(0);
    h.now.t += 61_000;
    for (let i = 0; i < 2; i++) expect((await ok()).status).toBe(200);
    const daily = await ok();
    expect(((await daily.json()) as { error: { code: string } }).error.code).toBe('daily_limit');
  });

  it('reports today\'s usage to the signed device only', async () => {
    const h = harness();
    const dev = await registerDevice(h);
    const other = await registerDevice(h, '198.51.100.9');
    const usage = async (d: typeof dev) => (await h.call('GET', '/v1/usage', { headers: await d.sign('GET', '/v1/usage', '') })).json() as Promise<Record<string, number>>;
    expect(await usage(dev)).toMatchObject({ usedToday: 0 });
    for (let i = 0; i < 2; i++) expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    const u = await usage(dev);
    expect(u.usedToday).toBe(2);
    expect(u.limitPerDay).toBeGreaterThan(2);
    expect(u.resetsIn).toBeGreaterThan(0);
    expect((await usage(other)).usedToday).toBe(0);
    expect((await h.call('GET', '/v1/usage')).status).toBe(401);
  });

  it('global ceilings protect the provider quota', async () => {
    const h = harness(env({ GLOBAL_REQUESTS_PER_MINUTE: '2', DEVICE_REQUESTS_PER_MINUTE: '50', DEVICE_REQUESTS_PER_DAY: '50' }));
    const a = await registerDevice(h);
    const b = await registerDevice(h, '198.51.100.9');
    expect((await signedCall(h, a, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    expect((await signedCall(h, b, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    const busy = await signedCall(h, a, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    expect(busy.status).toBe(503);
    expect(((await busy.json()) as { error: { code: string } }).error.code).toBe('service_busy');
  });

  it('global token budget per minute', async () => {
    const h = harness(env({ GLOBAL_TOKENS_PER_MINUTE: '3000', MAX_OUTPUT_TOKENS: '1000' }));
    const a = await registerDevice(h);
    expect((await signedCall(h, a, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    expect((await signedCall(h, a, '/v1/ai/summarize', { transcript: 'x'.repeat(9000) })).status).toBe(503);
  });
});

describe('Gemini provider', () => {
  const geminiEnv = env({ PROVIDER: 'gemini' });

  it('sends the schema and key server-side and normalizes the answer', async () => {
    const calls: { url: string; init: RequestInit }[] = [];
    const fetchImpl = (async (url: string, init: RequestInit) => {
      calls.push({ url, init });
      return geminiOk({ overview: '  Visão   geral ', points: [{ text: 'Data', time: '00:40', extra: 1 }, { text: '', time: '00:01' }] });
    }) as unknown as typeof fetch;
    const h = harness(geminiEnv, fetchImpl);
    const dev = await registerDevice(h);
    const r = await signedCall(h, dev, '/v1/ai/summarize', { language: 'pt', transcript: TRANSCRIPT });
    const text = await r.text();
    expect(r.status).toBe(200);
    expect(JSON.parse(text).result).toEqual({ overview: 'Visão geral', points: [{ text: 'Data', time: '00:40' }] });
    expect(calls[0]!.url).toContain('/models/gemini-test-flash:generateContent');
    expect((calls[0]!.init.headers as Record<string, string>)['x-goog-api-key']).toBe(SECRET_KEY);
    const sent = JSON.parse(String(calls[0]!.init.body));
    expect(sent.generationConfig.responseMimeType).toBe('application/json');
    expect(sent.generationConfig.responseJsonSchema.required).toEqual(['overview', 'points']);
    expect(sent.systemInstruction.parts[0].text).toContain('Portuguese (Brazil)');
    // The key never reaches the app or the logs.
    expect(text).not.toContain(SECRET_KEY);
    expect(JSON.stringify(h.logs)).not.toContain(SECRET_KEY);
    expect(JSON.stringify(h.logs)).not.toContain('lançamento');
  });

  it('retries a short rate limit, then succeeds', async () => {
    let n = 0;
    const fetchImpl = (async () =>
      ++n === 1
        ? new Response(JSON.stringify({ error: { code: 429, status: 'RESOURCE_EXHAUSTED', details: [{ retryDelay: '0.01s', violations: [{ quotaId: 'GenerateRequestsPerMinutePerProjectPerModel-FreeTier' }] }] } }), { status: 429 })
        : geminiOk({ overview: 'ok', points: [] })) as unknown as typeof fetch;
    const h = harness(geminiEnv, fetchImpl);
    const dev = await registerDevice(h);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    expect(n).toBe(2);
  });

  it('reports an exhausted daily quota without retrying or falling back', async () => {
    let n = 0;
    const fetchImpl = (async () => {
      n++;
      return new Response(JSON.stringify({ error: { code: 429, status: 'RESOURCE_EXHAUSTED', details: [{ violations: [{ quotaId: 'GenerateRequestsPerDayPerProjectPerModel-FreeTier' }] }] } }), { status: 429 });
    }) as unknown as typeof fetch;
    const h = harness(geminiEnv, fetchImpl);
    const dev = await registerDevice(h);
    const r = await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    expect(r.status).toBe(503);
    expect(((await r.json()) as { error: { code: string } }).error.code).toBe('quota_exhausted');
    expect(n).toBe(1);
  });

  it('server errors: retried, then unavailable; invalid output and timeouts are errors', async () => {
    let n = 0;
    const down = (async () => {
      n++;
      return new Response('{}', { status: 503 });
    }) as unknown as typeof fetch;
    const h = harness(env({ PROVIDER: 'gemini', PROVIDER_TIMEOUT_MS: '20000' }), down);
    const dev = await registerDevice(h);
    const r = await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    expect(r.status).toBe(503);
    expect(n).toBe(3);

    const bad = (async () => geminiOk({ unexpected: true })) as unknown as typeof fetch;
    const h2 = harness(geminiEnv, bad);
    const d2 = await registerDevice(h2);
    const r2 = await signedCall(h2, d2, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    expect(r2.status).toBe(502);
    expect(((await r2.json()) as { error: { code: string } }).error.code).toBe('provider_error');

    const slow = ((_: string, init: RequestInit) =>
      new Promise((_r, reject) => init.signal?.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError'))))) as unknown as typeof fetch;
    const h3 = harness(env({ PROVIDER: 'gemini', PROVIDER_TIMEOUT_MS: '1000' }), slow);
    const d3 = await registerDevice(h3);
    const r3 = await signedCall(h3, d3, '/v1/ai/topics', { transcript: TRANSCRIPT });
    expect(r3.status).toBe(503);
    expect(((await r3.json()) as { error: { code: string } }).error.code).toBe('unavailable');
  });

  it('config refuses to start without secrets', async () => {
    const { loadConfig } = await import('../src/config');
    expect(() => loadConfig(env({ PROVIDER: 'gemini', GEMINI_API_KEY: undefined }))).toThrow(/GEMINI_API_KEY/);
    expect(() => loadConfig(env({ PROVIDER: 'deepseek', DEEPSEEK_API_KEY: undefined }))).toThrow(/DEEPSEEK_API_KEY/);
    expect(() => loadConfig(env({ PROVIDER: 'other' }))).toThrow(/PROVIDER/);
    expect(loadConfig(env({ PROVIDER: 'deepseek' }))).toMatchObject({ provider: 'deepseek', model: 'deepseek-test', apiKey: DEEPSEEK_KEY });
    expect(() => loadConfig(env({ TOKEN_SECRET: 'short' }))).toThrow(/TOKEN_SECRET/);
  });
});

describe('DeepSeek provider (default)', () => {
  const deepseekEnv = env({ PROVIDER: 'deepseek' });
  const ok = (payload: unknown, finish = 'stop') =>
    new Response(`\n\n${JSON.stringify({ choices: [{ message: { content: JSON.stringify(payload) }, finish_reason: finish }], usage: { prompt_tokens: 120, completion_tokens: 40 } })}`, {
      status: 200,
    });

  it('uses JSON mode with the schema in the prompt, server-side key, and hides the provider from the app', async () => {
    const calls: { url: string; init: RequestInit }[] = [];
    const fetchImpl = (async (url: string, init: RequestInit) => {
      calls.push({ url, init });
      return ok({ topics: [{ title: 'Lançamento', time: '00:40', subtopics: [{ title: 'Data', time: '00:40', points: ['15/12'] }] }], decisions: [], actions: [], dates: [], numbers: [], keywords: ['Lumen'] });
    }) as unknown as typeof fetch;
    const h = harness(deepseekEnv, fetchImpl);
    const dev = await registerDevice(h);
    const r = await signedCall(h, dev, '/v1/ai/topics', { language: 'pt', detail: 'balanced', transcript: TRANSCRIPT });
    const text = await r.text();
    expect(r.status).toBe(200);
    expect(JSON.parse(text).result.topics[0].subtopics[0].points).toEqual(['15/12']);
    expect(calls[0]!.url).toBe('https://api.deepseek.com/chat/completions');
    expect((calls[0]!.init.headers as Record<string, string>).authorization).toBe(`Bearer ${DEEPSEEK_KEY}`);
    const sent = JSON.parse(String(calls[0]!.init.body));
    expect(sent).toMatchObject({ model: 'deepseek-test', response_format: { type: 'json_object' }, thinking: { type: 'disabled' }, stream: false });
    expect(sent.messages[0].content).toContain('JSON Schema');
    expect(sent.messages[0].content).toContain('"subtopics"');
    expect(sent.messages[1].content).toContain('[00:40] O lançamento');
    // Nothing about the provider, model or key reaches the app; no content in logs.
    expect(text.toLowerCase()).not.toMatch(/deepseek|model/);
    expect(JSON.stringify(h.logs)).not.toContain(DEEPSEEK_KEY);
    expect(JSON.stringify(h.logs)).not.toContain('lançamento');
  });

  it('retries blank JSON-mode answers and overload, reports an empty balance without fallback', async () => {
    let n = 0;
    const flaky = (async () => {
      n++;
      if (n === 1) return new Response(JSON.stringify({ choices: [{ message: { content: '   ' }, finish_reason: 'stop' }] }), { status: 200 });
      if (n === 2) return new Response('{}', { status: 503, headers: { 'retry-after': '0' } });
      return ok({ overview: 'ok', points: [] });
    }) as unknown as typeof fetch;
    const h = harness(deepseekEnv, flaky);
    const dev = await registerDevice(h);
    expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(200);
    expect(n).toBe(3);

    let m = 0;
    const broke = (async () => {
      m++;
      return new Response(JSON.stringify({ error: { message: 'Insufficient Balance' } }), { status: 402 });
    }) as unknown as typeof fetch;
    const h2 = harness(deepseekEnv, broke);
    const d2 = await registerDevice(h2);
    const r = await signedCall(h2, d2, '/v1/ai/summarize', { transcript: TRANSCRIPT });
    expect(r.status).toBe(503);
    expect(((await r.json()) as { error: { code: string } }).error.code).toBe('quota_exhausted');
    expect(m).toBe(1);
  });

  it('truncated, filtered, misconfigured and invalid answers are errors', async () => {
    const cases: [Response, number][] = [
      [ok({ overview: 'x', points: [] }, 'length'), 413],
      [ok({ overview: 'x', points: [] }, 'content_filter'), 502],
      [new Response('{}', { status: 401 }), 502],
      [ok({ wrong: true }), 502],
    ];
    for (const [resp, status] of cases) {
      const h = harness(deepseekEnv, (async () => resp.clone()) as unknown as typeof fetch);
      const dev = await registerDevice(h);
      expect((await signedCall(h, dev, '/v1/ai/summarize', { transcript: TRANSCRIPT })).status).toBe(status);
    }
  });
});
