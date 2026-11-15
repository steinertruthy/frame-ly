/**
 * Anonymous device authentication (FrameLy has no user accounts):
 *
 * 1. `GET /v1/devices/challenge` → a short-lived challenge signed with TOKEN_SECRET (stateless).
 * 2. The app creates an Ed25519 key pair (kept in the OS keychain) and searches a nonce so that
 *    SHA-256(challenge:publicKey:nonce) starts with `powBits` zero bits — cheap once per
 *    install, costly for scripts that would mint thousands of identities.
 * 3. `POST /v1/devices` registers the public key; its device id is a hash of it.
 * 4. Every AI request is signed: Ed25519 over method, path, timestamp, request id and a hash of
 *    the body. Replayed request ids and stale timestamps are rejected.
 */
import { ApiError } from './errors';
import { fromBase64, hmacHex, leadingZeroBits, safeEqual, sha256, toHex, verifyEd25519 } from './crypto';

const CHALLENGE_TTL_MS = 10 * 60_000;
/** Accepted clock skew for signed requests. */
export const SIGNATURE_WINDOW_MS = 5 * 60_000;

export async function issueChallenge(secret: string, now: number): Promise<string> {
  const rand = toHex(crypto.getRandomValues(new Uint8Array(16)));
  const payload = `${now + CHALLENGE_TTL_MS}.${rand}`;
  return `${payload}.${await hmacHex(secret, payload)}`;
}

export async function checkChallenge(secret: string, challenge: string, now: number): Promise<void> {
  const parts = challenge.split('.');
  if (parts.length !== 3) throw new ApiError('invalid_request', 'malformed challenge');
  const [expires, rand, sig] = parts as [string, string, string];
  if (!safeEqual(sig, await hmacHex(secret, `${expires}.${rand}`))) throw new ApiError('unauthorized', 'invalid challenge');
  if (Number(expires) < now) throw new ApiError('unauthorized', 'challenge expired');
}

export async function powOk(challenge: string, publicKey: string, nonce: string, bits: number): Promise<boolean> {
  return leadingZeroBits(await sha256(`${challenge}:${publicKey}:${nonce}`)) >= bits;
}

export async function deviceIdOf(publicKey: Uint8Array): Promise<string> {
  return toHex(await sha256(publicKey)).slice(0, 32);
}

export function decodePublicKey(b64: string): Uint8Array {
  let pk: Uint8Array;
  try {
    pk = fromBase64(b64);
  } catch {
    throw new ApiError('invalid_request', 'publicKey must be base64');
  }
  if (pk.length !== 32) throw new ApiError('invalid_request', 'publicKey must be a 32-byte Ed25519 key');
  return pk;
}

/** The exact bytes a device signs for a request. */
export async function signingString(method: string, path: string, timestamp: string, requestId: string, body: string): Promise<string> {
  return `${method}\n${path}\n${timestamp}\n${requestId}\n${toHex(await sha256(body))}`;
}

export interface SignedHeaders {
  deviceId: string;
  timestamp: string;
  requestId: string;
  signature: string;
}

export function readSignedHeaders(req: Request): SignedHeaders {
  const h = (n: string) => req.headers.get(n) ?? '';
  const s = {
    deviceId: h('x-framely-device'),
    timestamp: h('x-framely-timestamp'),
    requestId: h('x-framely-request'),
    signature: h('x-framely-signature'),
  };
  if (!/^[0-9a-f]{32}$/.test(s.deviceId) || !/^\d{10,16}$/.test(s.timestamp) || !/^[0-9a-f-]{16,64}$/i.test(s.requestId) || !s.signature) {
    throw new ApiError('unauthorized', 'missing or malformed signature headers');
  }
  return s;
}

export async function verifyRequest(
  method: string,
  path: string,
  body: string,
  h: SignedHeaders,
  publicKeyB64: string,
  now: number,
): Promise<void> {
  if (Math.abs(now - Number(h.timestamp)) > SIGNATURE_WINDOW_MS) throw new ApiError('unauthorized', 'request timestamp out of range');
  let sig: Uint8Array;
  try {
    sig = fromBase64(h.signature);
  } catch {
    throw new ApiError('unauthorized', 'bad signature');
  }
  const ok = await verifyEd25519(fromBase64(publicKeyB64), sig, await signingString(method, path, h.timestamp, h.requestId, body));
  if (!ok) throw new ApiError('unauthorized', 'bad signature');
}
