/**
 * The only place settings are read. Everything comes from wrangler.toml vars and secrets.
 * Provider-specific settings use the provider's name as prefix (DEEPSEEK_MODEL,
 * GEMINI_DATA_USE…), so switching PROVIDER switches all of them together.
 */
export interface Env {
  LIMITER: DurableObjectNamespace;
  /** "deepseek" (default) | "gemini" | "mock". */
  PROVIDER: string;
  PROVIDER_TIMEOUT_MS: string;
  MAX_INPUT_CHARS: string;
  MAX_OUTPUT_TOKENS: string;
  REGISTRATION_POW_BITS: string;
  DEVICE_REQUESTS_PER_MINUTE: string;
  DEVICE_REQUESTS_PER_DAY: string;
  IP_REGISTRATIONS_PER_DAY: string;
  GLOBAL_REQUESTS_PER_MINUTE: string;
  GLOBAL_REQUESTS_PER_DAY: string;
  GLOBAL_TOKENS_PER_MINUTE: string;
  /** Secrets. */
  TOKEN_SECRET?: string;
  DEEPSEEK_API_KEY?: string;
  GEMINI_API_KEY?: string;
  /** Per-provider vars: <PROVIDER>_MODEL, <PROVIDER>_DATA_USE, <PROVIDER>_BLOCKED_COUNTRIES, DEEPSEEK_THINKING. */
  [key: string]: unknown;
}

export interface Limits {
  deviceMinute: number;
  deviceDay: number;
  ipRegistrationsDay: number;
  globalMinute: number;
  globalDay: number;
  globalTokensMinute: number;
}

export type ProviderName = 'deepseek' | 'gemini' | 'mock';

export interface Config {
  provider: ProviderName;
  /** Model of the active provider (never sent to the app). */
  model: string;
  apiKey?: string;
  /** How the active provider may use content, shown before consent (without naming it). */
  dataUse: 'may_train' | 'no_training';
  /** Countries where the active provider may not be offered. */
  blockedCountries: Set<string>;
  /** DeepSeek: "enabled" | "disabled" reasoning before answering. */
  deepseekThinking: 'enabled' | 'disabled';
  timeoutMs: number;
  maxInputChars: number;
  maxOutputTokens: number;
  powBits: number;
  limits: Limits;
  tokenSecret: string;
}

function int(env: Env, name: string, min: number, max: number): number {
  const v = Number(env[name]);
  if (!Number.isInteger(v) || v < min || v > max) throw new Error(`config: ${name} must be an integer in [${min}, ${max}]`);
  return v;
}

function str(env: Env, name: string): string {
  const v = env[name];
  return typeof v === 'string' ? v.trim() : '';
}

export function loadConfig(env: Env): Config {
  const provider = (['deepseek', 'gemini', 'mock'] as const).find((p) => p === str(env, 'PROVIDER'));
  if (!provider) throw new Error('config: PROVIDER must be "deepseek", "gemini" or "mock"');
  const P = provider.toUpperCase();
  if (!env.TOKEN_SECRET || env.TOKEN_SECRET.length < 16) throw new Error('config: TOKEN_SECRET secret missing');
  const apiKey = str(env, `${P}_API_KEY`) || undefined;
  if (provider !== 'mock' && !apiKey) throw new Error(`config: ${P}_API_KEY secret missing`);
  const model = str(env, `${P}_MODEL`);
  if (provider !== 'mock' && !model) throw new Error(`config: ${P}_MODEL missing`);
  return {
    provider,
    model: model || 'mock',
    apiKey,
    dataUse: str(env, `${P}_DATA_USE`) === 'no_training' ? 'no_training' : 'may_train',
    blockedCountries: new Set(str(env, `${P}_BLOCKED_COUNTRIES`).split(',').map((c) => c.trim().toUpperCase()).filter(Boolean)),
    deepseekThinking: str(env, 'DEEPSEEK_THINKING') === 'enabled' ? 'enabled' : 'disabled',
    timeoutMs: int(env, 'PROVIDER_TIMEOUT_MS', 1_000, 300_000),
    maxInputChars: int(env, 'MAX_INPUT_CHARS', 1_000, 2_000_000),
    maxOutputTokens: int(env, 'MAX_OUTPUT_TOKENS', 256, 64_000),
    powBits: int(env, 'REGISTRATION_POW_BITS', 0, 28),
    limits: {
      deviceMinute: int(env, 'DEVICE_REQUESTS_PER_MINUTE', 1, 10_000),
      deviceDay: int(env, 'DEVICE_REQUESTS_PER_DAY', 1, 1_000_000),
      ipRegistrationsDay: int(env, 'IP_REGISTRATIONS_PER_DAY', 1, 10_000),
      globalMinute: int(env, 'GLOBAL_REQUESTS_PER_MINUTE', 1, 1_000_000),
      globalDay: int(env, 'GLOBAL_REQUESTS_PER_DAY', 1, 100_000_000),
      globalTokensMinute: int(env, 'GLOBAL_TOKENS_PER_MINUTE', 1_000, 100_000_000),
    },
    tokenSecret: env.TOKEN_SECRET,
  };
}
