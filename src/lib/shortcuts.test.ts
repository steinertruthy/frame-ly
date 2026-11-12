import { describe, expect, it } from 'vitest';
import { eventToAccelerator, formatAccelerator, keyName } from './shortcuts';

const ev = (code: string, mods: Partial<Record<'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey', boolean>> = {}) => ({
  code,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe('shortcuts', () => {
  it('maps physical keys', () => {
    expect(keyName('KeyR')).toBe('R');
    expect(keyName('Digit4')).toBe('4');
    expect(keyName('F13')).toBe('F13');
    expect(keyName('ArrowUp')).toBe('Up');
    expect(keyName('IntlBackslash')).toBeNull();
  });

  it('builds accelerators with the platform primary modifier', () => {
    expect(eventToAccelerator(ev('KeyR', { metaKey: true, altKey: true, shiftKey: true }), 'macos')).toBe('CmdOrCtrl+Alt+Shift+R');
    expect(eventToAccelerator(ev('KeyR', { ctrlKey: true, altKey: true, shiftKey: true }), 'windows')).toBe('CmdOrCtrl+Alt+Shift+R');
    expect(eventToAccelerator(ev('Digit4', { ctrlKey: true }), 'macos')).toBe('Ctrl+4');
    expect(eventToAccelerator(ev('KeyS', { metaKey: true }), 'windows')).toBe('Super+S');
  });

  it('waits while only modifiers are pressed', () => {
    expect(eventToAccelerator(ev('ShiftLeft', { shiftKey: true }), 'macos')).toBeNull();
    expect(eventToAccelerator(ev('MetaRight', { metaKey: true }), 'macos')).toBeNull();
  });

  it('formats for each platform', () => {
    expect(formatAccelerator('CmdOrCtrl+Alt+Shift+R', 'macos')).toEqual(['⌥', '⇧', '⌘', 'R']);
    expect(formatAccelerator('CmdOrCtrl+Alt+Shift+Space', 'windows')).toEqual(['Ctrl', 'Alt', 'Shift', 'Space']);
    expect(formatAccelerator('Ctrl+Slash', 'macos')).toEqual(['⌃', '/']);
    expect(formatAccelerator('', 'macos')).toEqual([]);
  });
});
