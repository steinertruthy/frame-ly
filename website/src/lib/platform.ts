export type Os = 'macos' | 'windows';
export type Detected = Os | 'other';

interface NavigatorLike {
  userAgent: string;
  platform?: string;
  userAgentData?: { platform?: string; mobile?: boolean };
}

/**
 * Desktop OS of the visitor. iPhone/iPad/Android/Linux and anything unclear → 'other', so the page
 * shows a neutral choice instead of guessing. (iPadOS reports "Macintosh" but has touch points.)
 */
export function detectOs(nav: NavigatorLike, maxTouchPoints = 0): Detected {
  const hint = nav.userAgentData?.platform?.toLowerCase() ?? '';
  if (nav.userAgentData?.mobile) return 'other';
  if (hint === 'macos') return 'macos';
  if (hint === 'windows') return 'windows';
  const ua = nav.userAgent;
  if (/iPhone|iPad|iPod|Android|CrOS|Linux/i.test(ua) && !/Windows/i.test(ua)) return 'other';
  if (/Windows NT/i.test(ua)) return 'windows';
  if (/Macintosh|Mac OS X/i.test(ua)) return maxTouchPoints > 1 ? 'other' : 'macos';
  return 'other';
}

/** Manual choice wins over detection. */
export function chooseOs(saved: string | null, detected: Detected): Detected {
  return saved === 'macos' || saved === 'windows' ? saved : detected;
}
