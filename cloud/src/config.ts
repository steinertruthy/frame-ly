/** The only place settings are read. Everything comes from wrangler.toml vars and secrets. */
export interface Env {
  LIMITER: DurableObjectNamespace;
  PROVIDER: string;
  GEMINI_MODEL: string;
  DATA_USE: string;
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
  BLOCKED_COUNTRIES: string;
  /** Secrets. */
  GEMINI_API_KEY?: string;
  TOKEN_SECRET?: string;
}

export interface Limits {
  deviceMinute: number;
  deviceDay: number;
  ipRegistrationsDay: number;
  globalMinute: number;
  globalDay: number;
  globalTokensMinute: number;
}

export interface Config {
  provider: 'gemini' | 'mock';
  model: string;
  /** Who processes the text, shown before consent. */
  processor: string;
  dataUse: 'may_train' | 'no_training';
  timeoutMs: number;
  maxInputChars: number;
  maxOutputTokens: number;
  powBits: number;
  limits: Limits;
  blockedCountries: Set<string>;
  geminiApiKey?: string;
  tokenSecret: string;
}

function int(env: Env, name: keyof Env, min: number, max: number): number {
  const v = Number(env[name]);
  if (!Number.isInteger(v) || v < min || v > max) throw new Error(`config: ${String(name)} must be an integer in [${min}, ${max}]`);
  return v;
}

export function loadConfig(env: Env): Config {
  const provider = env.PROVIDER === 'mock' ? 'mock' : env.PROVIDER === 'gemini' ? 'gemini' : null;
  if (!provider) throw new Error('config: PROVIDER must be "gemini" or "mock"');
  if (!env.TOKEN_SECRET || env.TOKEN_SECRET.length < 16) throw new Error('config: TOKEN_SECRET secret missing');
  if (provider === 'gemini' && !env.GEMINI_API_KEY) throw new Error('config: GEMINI_API_KEY secret missing');
  return {
    provider,
    model: env.GEMINI_MODEL,
    processor: provider === 'gemini' ? 'Google Gemini' : 'FrameLy (mock)',
    dataUse: env.DATA_USE === 'no_training' ? 'no_training' : 'may_train',
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
    blockedCountries: new Set(env.BLOCKED_COUNTRIES.split(',').map((c) => c.trim().toUpperCase()).filter(Boolean)),
    geminiApiKey: env.GEMINI_API_KEY,
    tokenSecret: env.TOKEN_SECRET,
  };
}
