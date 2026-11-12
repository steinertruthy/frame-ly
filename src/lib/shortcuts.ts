/** Keyboard shortcut helpers shared by Settings and the quick panel. Pure, unit-tested. */

export type ShortcutAction =
  | 'openPanel'
  | 'captureRegion'
  | 'captureWindow'
  | 'captureScreen'
  | 'recordRegion'
  | 'recordWindow'
  | 'recordScreen'
  | 'stopRecording'
  | 'togglePause';

export const SHORTCUT_ACTIONS: ShortcutAction[] = [
  'openPanel',
  'captureRegion',
  'captureWindow',
  'captureScreen',
  'recordRegion',
  'recordWindow',
  'recordScreen',
  'stopRecording',
  'togglePause',
];

export const RECORDING_ACTIONS: ShortcutAction[] = ['recordRegion', 'recordWindow', 'recordScreen', 'stopRecording', 'togglePause'];

const MODIFIER_CODES = new Set(['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight', 'CapsLock', 'Fn']);

const NAMED: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Escape: 'Escape',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Minus: 'Minus',
  Equal: 'Equal',
  Comma: 'Comma',
  Period: 'Period',
  Slash: 'Slash',
  Backslash: 'Backslash',
  Semicolon: 'Semicolon',
  Quote: 'Quote',
  Backquote: 'Backquote',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  PrintScreen: 'PrintScreen',
};

/** Physical key (layout-independent `KeyboardEvent.code`) → accelerator key name. */
export function keyName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  return NAMED[code] ?? null;
}

type KeyEventLike = Pick<KeyboardEvent, 'code' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>;

/**
 * Builds an accelerator from a key press, or null while only modifiers are held.
 * The platform's primary modifier (⌘ on macOS, Ctrl on Windows) becomes `CmdOrCtrl`.
 */
export function eventToAccelerator(e: KeyEventLike, platform: string): string | null {
  if (MODIFIER_CODES.has(e.code)) return null;
  const key = keyName(e.code);
  if (!key) return null;
  const mac = platform === 'macos';
  const parts: string[] = [];
  if (mac ? e.metaKey : e.ctrlKey) parts.push('CmdOrCtrl');
  if (mac && e.ctrlKey) parts.push('Ctrl');
  if (!mac && e.metaKey) parts.push('Super');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  parts.push(key);
  return parts.join('+');
}

const MAC_SYMBOLS: Record<string, string> = {
  CmdOrCtrl: '⌘',
  Super: '⌘',
  Ctrl: '⌃',
  Alt: '⌥',
  Shift: '⇧',
  Space: 'Space',
  Enter: '↩',
  Backspace: '⌫',
  Delete: '⌦',
  Escape: 'Esc',
  Tab: '⇥',
  Up: '↑',
  Down: '↓',
  Left: '←',
  Right: '→',
};

const WIN_NAMES: Record<string, string> = { CmdOrCtrl: 'Ctrl', Super: 'Win', Up: '↑', Down: '↓', Left: '←', Right: '→' };

const PUNCT: Record<string, string> = {
  Minus: '-',
  Equal: '=',
  Comma: ',',
  Period: '.',
  Slash: '/',
  Backslash: '\\',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  BracketLeft: '[',
  BracketRight: ']',
  PrintScreen: 'PrtSc',
};

/** Accelerator → display chips, in the platform's conventions (⌃⌥⇧⌘ order on macOS). */
export function formatAccelerator(accel: string, platform: string): string[] {
  if (!accel) return [];
  const parts = accel.split('+');
  const key = parts.pop()!;
  const mac = platform === 'macos';
  const order = mac ? ['Ctrl', 'Alt', 'Shift', 'CmdOrCtrl', 'Super'] : ['CmdOrCtrl', 'Super', 'Alt', 'Shift'];
  const mods = order.filter((m) => parts.includes(m)).map((m) => (mac ? MAC_SYMBOLS[m] : (WIN_NAMES[m] ?? m)) ?? m);
  const k = PUNCT[key] ?? (mac ? MAC_SYMBOLS[key] : WIN_NAMES[key]) ?? key;
  return [...mods, k];
}
