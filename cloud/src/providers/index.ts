/**
 * The AI provider behind FrameLy Cloud. One interface, one implementation per provider; the
 * active one is chosen by the PROVIDER var. Adding Groq, OpenRouter, Workers AI, OpenAI or
 * Anthropic later means adding a file here and a case in `providerFor` — nothing in the app.
 */
import type { Config } from '../config';
import { deepseek } from './deepseek';
import { gemini } from './gemini';
import { mock } from './mock';

export interface GenerateRequest {
  system: string;
  user: string;
  schema: Record<string, unknown>;
  maxOutputTokens: number;
}

export interface GenerateResult {
  text: string;
  inputTokens: number;
  outputTokens: number;
}

/** Provider failures the API maps to structured errors. */
export class ProviderFailure extends Error {
  constructor(
    public kind: 'quota' | 'busy' | 'unavailable' | 'invalid' | 'refused' | 'too_long' | 'timeout',
    message: string,
    public retryAfter?: number,
  ) {
    super(message);
  }
}

export type Provider = (req: GenerateRequest, cfg: Config, fetchImpl: typeof fetch) => Promise<GenerateResult>;

export function providerFor(cfg: Config): Provider {
  switch (cfg.provider) {
    case 'deepseek':
      return deepseek;
    case 'gemini':
      return gemini;
    default:
      return mock;
  }
}
