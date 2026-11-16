import { describe, expect, it } from 'vitest';
import { releaseNotes } from './updates';

describe('releaseNotes', () => {
  it('keeps the changelog part as plain lines', () => {
    const body = 'FrameLy AI Cloud is live.\n\n### Added\n\n- **Cloud** in `builds`, see [site](https://x.y)\n\n---\n\n### Downloads\n| a | b |';
    expect(releaseNotes(body)).toEqual([
      { text: 'FrameLy AI Cloud is live.', heading: false },
      { text: 'Added', heading: true },
      { text: 'Cloud in builds, see site', heading: false },
    ]);
  });
  it('handles empty notes', () => {
    expect(releaseNotes(null)).toEqual([]);
    expect(releaseNotes('  ')).toEqual([]);
  });
});
