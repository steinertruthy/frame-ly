/**
 * DeepSeek (OpenAI-compatible chat completions, JSON output mode). DeepSeek's JSON mode
 * guarantees a JSON object but not a schema, so the schema goes into the system prompt and the
 * service validates the result (validate.ts) like any other provider's.
 */
import type { Config } from '../config';
import { ProviderFailure, type Provider } from './index';

const ENDPOINT = 'https://api.deepseek.com/chat/completions';
const MAX_RETRIES = 2;
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const deepseek: Provider = async (req, cfg: Config, fetchImpl) => {
  const system = `${req.system}\nReturn one JSON object that follows this JSON Schema exactly (no other text):\n${JSON.stringify(req.schema)}`;
  const body = JSON.stringify({
    model: cfg.model,
    messages: [
      { role: 'system', content: system },
      { role: 'user', content: req.user },
    ],
    response_format: { type: 'json_object' },
    thinking: { type: cfg.deepseekThinking },
    max_tokens: req.maxOutputTokens,
    stream: false,
  });
  const deadline = Date.now() + cfg.timeoutMs;
  for (let attempt = 0; ; attempt++) {
    const remaining = deadline - Date.now();
    if (remaining <= 0) throw new ProviderFailure('timeout', 'provider timed out');
    const ctl = new AbortController();
    const timer = setTimeout(() => ctl.abort(), remaining);
    let res: Response;
    let text: string;
    try {
      res = await fetchImpl(ENDPOINT, {
        method: 'POST',
        headers: { 'content-type': 'application/json', authorization: `Bearer ${cfg.apiKey ?? ''}` },
        body,
        signal: ctl.signal,
      });
      // Non-streaming responses may start with keep-alive blank lines.
      text = (await res.text()).trim();
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
      let data: {
        choices?: { message?: { content?: string }; finish_reason?: string }[];
        usage?: { prompt_tokens?: number; completion_tokens?: number };
      };
      try {
        data = JSON.parse(text);
      } catch {
        throw new ProviderFailure('invalid', 'unreadable provider response');
      }
      const c = data.choices?.[0];
      if (c?.finish_reason === 'length') throw new ProviderFailure('too_long', 'response exceeded the output limit');
      if (c?.finish_reason === 'content_filter') throw new ProviderFailure('refused', 'content filtered');
      const content = (c?.message?.content ?? '').trim();
      // JSON mode replaces output it can't parse with blanks: try again, then give up.
      if (!content) {
        if (attempt < MAX_RETRIES) continue;
        throw new ProviderFailure('invalid', 'empty response');
      }
      return { text: content, inputTokens: data.usage?.prompt_tokens ?? 0, outputTokens: data.usage?.completion_tokens ?? 0 };
    }

    // 402: the account balance ran out — reported, never replaced by another (paid) provider.
    if (res.status === 402) throw new ProviderFailure('quota', 'provider balance exhausted');
    if ((res.status === 429 || res.status >= 500) && attempt < MAX_RETRIES) {
      const wait = Number(res.headers.get('retry-after')) || attempt + 1;
      if (wait * 1000 < deadline - Date.now()) {
        await sleep(wait * 1000);
        continue;
      }
    }
    if (res.status === 429) throw new ProviderFailure('busy', 'provider concurrency limit', 30);
    if (res.status >= 500) throw new ProviderFailure('unavailable', `provider error ${res.status}`);
    // 400/401/422: misconfiguration (bad key, unknown model, invalid parameter) — never retried.
    throw new ProviderFailure('invalid', `provider rejected the request (${res.status})`);
  }
};
