import { describe, expect, it } from 'vitest';
import { fileSafe, transcriptTitle } from './format';

describe('transcript titles', () => {
  const media = { name: 'FrameLy 2026-11-14 at 19.30.13.mp4' };
  it('shows the file name until renamed', () => {
    expect(transcriptTitle({ title: media.name, customTitle: false, media })).toBe('FrameLy 2026-11-14 at 19.30.13');
    expect(transcriptTitle({ title: 'Reunião de planejamento', customTitle: true, media })).toBe('Reunião de planejamento');
  });
  it('makes titles safe for file names', () => {
    expect(fileSafe('Aula 3: vetores / matrizes?')).toBe('Aula 3 vetores matrizes');
    expect(fileSafe('  ...  ')).toBe('FrameLy');
    expect(fileSafe('Q4 report.')).toBe('Q4 report');
  });
});
