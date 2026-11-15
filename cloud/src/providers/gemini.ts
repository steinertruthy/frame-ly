/**
 * Google Gemini (generateContent, JSON output constrained by a schema). The model id comes from
 * GEMINI_MODEL only. Timeout per attempt, at most two retries for short-term limits and server
 * errors, and a daily quota that's exhausted is reported — never routed to a paid alternative.
 */
import type { Config } from '../config';
import { ProviderFailure, type Provider } from './index';

const ENDPOINT = 'https://generativelanguage.googleapis.com/v1beta/models';
const MAX_RETRIES = 2;

interface GeminiError {
  error?: { code?: number; status?: string; message?: string; details?: { '@type'?: string; retryDelay?: string; violations?: { quotaId?: string }[] }[] };
}

function retryDelaySeconds(body: GeminiError): number | undefined {
  for (const d of body.error?.details ?? []) {
    const m = d.retryDelay && /^(\d+(?:\.\d+)?)s$/.exec(d.retryDelay);
    if (m) return Number(m[1]);
  }
  return undefined;
}

/** A per-day quota (free tier daily requests) can't be waited out within a request. */
function isDailyQuota(body: GeminiError): boolean {
  return (body.error?.details ?? []).some((d) => (d.violations ?? []).some((v) => /PerDay/i.test(v.quotaId ?? '')));
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const gemini: Provider = async (req, cfg: Config, fetchImpl) => {
  const url = `${ENDPOINT}/${encodeURIComponent(cfg.model)}:generateContent`;
  const body = JSON.stringify({
    systemInstruction: { parts: [{ text: req.system }] },
    contents: [{ role: 'user', parts: [{ text: req.user }] }],
    generationConfig: { responseMimeType: 'application/json', responseJsonSchema: req.schema, maxOutputTokens: req.maxOutputTokens },
  });
  const deadline = Date.now() + cfg.timeoutMs;
  for (let attempt = 0; ; attempt++) {
    const remaining = deadline - Date.now();
    if (remaining <= 0) throw new ProviderFailure('timeout', 'provider timed out');
    const ctl = new AbortController();
    const timer = setTimeout(() => ctl.abort(), remaining);
    let res: Response;
    try {
      res = await fetchImpl(url, {
        method: 'POST',
        headers: { 'content-type': 'application/json', 'x-goog-api-key': cfg.geminiApiKey ?? '' },
        body,
        signal: ctl.signal,
      });
    } catch (e) {
      clearTimeout(timer);
      if (ctl.signal.aborted) throw new ProviderFailure('timeout', 'provider timed out');
      if (attempt < MAX_RETRIES) {
        await sleep(1000 * (attempt + 1));
        continue;
      }
      throw new ProviderFailure('unavailable', `provider unreachable: ${(e as Error).message}`);
    }
    clearTimeout(timer);

    if (res.ok) {
      const data = (await res.json()) as {
        candidates?: { content?: { parts?: { text?: string }[] }; finishReason?: string }[];
        promptFeedback?: { blockReason?: string };
        usageMetadata?: { promptTokenCount?: number; candidatesTokenCount?: number; thoughtsTokenCount?: number };
      };
      if (data.promptFeedback?.blockReason) throw new ProviderFailure('refused', `blocked: ${data.promptFeedback.blockReason}`);
      const c = data.candidates?.[0];
      const reason = c?.finishReason ?? '';
      if (reason === 'MAX_TOKENS') throw new ProviderFailure('too_long', 'response exceeded the output limit');
      if (reason && reason !== 'STOP') throw new ProviderFailure('refused', `finished with ${reason}`);
      const text = (c?.content?.parts ?? []).map((p) => p.text ?? '').join('');
      if (!text) throw new ProviderFailure('invalid', 'empty response');
      const u = data.usageMetadata ?? {};
      return { text, inputTokens: u.promptTokenCount ?? 0, outputTokens: (u.candidatesTokenCount ?? 0) + (u.thoughtsTokenCount ?? 0) };
    }

    const err = ((await res.json().catch(() => ({}))) ?? {}) as GeminiError;
    if (res.status === 429) {
      const wait = retryDelaySeconds(err);
      if (isDailyQuota(err)) throw new ProviderFailure('quota', 'daily provider quota exhausted', wait);
      if (attempt < MAX_RETRIES && (wait ?? 2) * 1000 < deadline - Date.now()) {
        await sleep((wait ?? 2) * 1000);
        continue;
      }
      throw new ProviderFailure('busy', 'provider rate limit', wait ?? 30);
    }
    if (res.status >= 500 && attempt < MAX_RETRIES) {
      await sleep(1000 * (attempt + 1));
      continue;
    }
    if (res.status >= 500) throw new ProviderFailure('unavailable', `provider error ${res.status}`);
    // 400/403/404: misconfiguration (bad key, unknown model, invalid schema) — never retried.
    throw new ProviderFailure('invalid', `provider rejected the request (${res.status} ${err.error?.status ?? ''})`);
  }
};
