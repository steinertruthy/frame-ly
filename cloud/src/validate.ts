/** Input validation and output validation/normalization. */
import { ApiError } from './errors';
import type { Detail, Partial, Task } from './prompts';

export interface AiRequest {
  task: Task;
  language: string | null;
  detail: Detail;
  /** Transcript lines "[mm:ss] text", or null when combining partial summaries. */
  transcript: string | null;
  part: { index: number; count: number } | null;
  partials: Partial[] | null;
}

const DETAILS: Detail[] = ['brief', 'balanced', 'detailed'];
const isObj = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);
const TIME = /^\d{1,2}(:\d{2}){1,2}$/;

function clean(s: unknown, max: number): string {
  return typeof s === 'string' ? s.replace(/\s+/g, ' ').trim().slice(0, max) : '';
}

export function parseRequest(task: Task, body: unknown, maxInputChars: number): AiRequest {
  if (!isObj(body)) throw new ApiError('invalid_request', 'body must be a JSON object');
  const detail = body.detail ?? 'balanced';
  if (!DETAILS.includes(detail as Detail)) throw new ApiError('invalid_request', 'detail must be brief, balanced or detailed');
  const language = body.language == null ? null : String(body.language);
  if (language !== null && !/^[a-z]{2,3}$/.test(language)) throw new ApiError('invalid_request', 'language must be an ISO 639-1 code');

  let transcript: string | null = null;
  let partials: Partial[] | null = null;
  if (task === 'summary' && body.partials != null) {
    if (!Array.isArray(body.partials) || body.partials.length < 2 || body.partials.length > 64) throw new ApiError('invalid_request', 'partials must have 2 to 64 items');
    partials = body.partials.map((p) => {
      if (!isObj(p) || !Array.isArray(p.points)) throw new ApiError('invalid_request', 'invalid partial summary');
      return {
        overview: clean(p.overview, 4_000),
        points: p.points.slice(0, 40).map((x) => (isObj(x) ? { text: clean(x.text, 1_000), time: TIME.test(String(x.time)) ? String(x.time) : '' } : { text: '', time: '' })),
      };
    });
  } else {
    if (typeof body.transcript !== 'string' || !body.transcript.trim()) throw new ApiError('invalid_request', 'transcript is required');
    transcript = body.transcript;
  }
  const size = (transcript ?? JSON.stringify(partials)).length;
  if (size > maxInputChars) throw new ApiError('too_large', `input exceeds ${maxInputChars} characters`);

  let part: AiRequest['part'] = null;
  if (body.part != null) {
    const p = body.part;
    if (!isObj(p) || !Number.isInteger(p.index) || !Number.isInteger(p.count) || (p.index as number) < 0 || (p.index as number) >= (p.count as number) || (p.count as number) > 256) {
      throw new ApiError('invalid_request', 'invalid part');
    }
    part = { index: p.index as number, count: p.count as number };
  }
  return { task, language, detail: detail as Detail, transcript, part, partials };
}

function timedList(v: unknown, max: number, extra: string[] = []): Record<string, string>[] {
  if (!Array.isArray(v)) return [];
  return v
    .filter(isObj)
    .map((x) => {
      const o: Record<string, string> = { text: clean(x.text, 1_000), time: clean(x.time, 16) };
      for (const k of extra) o[k] = clean(x[k], 200);
      return o;
    })
    .filter((x) => x.text)
    .slice(0, max);
}

/**
 * Validates the model's JSON against the expected shape and returns a trimmed, size-capped copy.
 * Anything structurally wrong is an error (the app never sees half-valid data).
 */
export function normalizeOutput(task: Task, raw: unknown): Record<string, unknown> {
  if (!isObj(raw)) throw new Error('response is not a JSON object');
  if (task === 'summary') {
    if (typeof raw.overview !== 'string' || !Array.isArray(raw.points)) throw new Error('summary shape');
    return { overview: clean(raw.overview, 6_000), points: timedList(raw.points, 40) };
  }
  if (!Array.isArray(raw.topics)) throw new Error('topics shape');
  const topics = raw.topics
    .filter(isObj)
    .map((t) => ({
      title: clean(t.title, 300),
      time: clean(t.time, 16),
      subtopics: (Array.isArray(t.subtopics) ? t.subtopics : [])
        .filter(isObj)
        .map((s) => ({
          title: clean(s.title, 300),
          time: clean(s.time, 16),
          points: (Array.isArray(s.points) ? s.points : []).map((p) => clean(p, 1_000)).filter(Boolean).slice(0, 30),
        }))
        .filter((s) => s.title)
        .slice(0, 30),
    }))
    .filter((t) => t.title)
    .slice(0, 40);
  return {
    topics,
    decisions: timedList(raw.decisions, 50),
    actions: timedList(raw.actions, 50, ['owner', 'due']),
    dates: timedList(raw.dates, 50),
    numbers: timedList(raw.numbers, 50),
    keywords: (Array.isArray(raw.keywords) ? raw.keywords : []).map((k) => clean(k, 80)).filter(Boolean).slice(0, 12),
  };
}

/** Extracts the JSON object from model text (tolerates code fences). */
export function parseModelJson(text: string): unknown {
  const a = text.indexOf('{');
  const b = text.lastIndexOf('}');
  if (a < 0 || b < a) throw new Error('no JSON object in response');
  return JSON.parse(text.slice(a, b + 1));
}
