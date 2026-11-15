/** Structured errors returned to the desktop app as `{ error: { code, message, retryAfter? } }`. */
export type ErrorCode =
  | 'invalid_request'
  | 'too_large'
  | 'unauthorized'
  | 'region_blocked'
  | 'rate_limited'
  | 'daily_limit'
  | 'service_busy'
  | 'quota_exhausted'
  | 'unavailable'
  | 'provider_error'
  | 'not_found';

const STATUS: Record<ErrorCode, number> = {
  invalid_request: 400,
  too_large: 413,
  unauthorized: 401,
  region_blocked: 451,
  rate_limited: 429,
  daily_limit: 429,
  service_busy: 503,
  quota_exhausted: 503,
  unavailable: 503,
  provider_error: 502,
  not_found: 404,
};

export class ApiError extends Error {
  constructor(
    public code: ErrorCode,
    message: string,
    /** Seconds until trying again makes sense. */
    public retryAfter?: number,
  ) {
    super(message);
  }

  get status(): number {
    return STATUS[this.code];
  }

  toResponse(): Response {
    const headers: Record<string, string> = { 'content-type': 'application/json', 'cache-control': 'no-store' };
    if (this.retryAfter != null) headers['retry-after'] = String(Math.ceil(this.retryAfter));
    return new Response(JSON.stringify({ error: { code: this.code, message: this.message, retryAfter: this.retryAfter } }), {
      status: this.status,
      headers,
    });
  }
}
