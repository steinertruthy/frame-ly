/**
 * Deterministic provider for local development and tests: answers in the real output format
 * from the transcript lines it receives, without calling any service.
 */
import type { Provider } from './index';

export const mock: Provider = async (req) => {
  const lines = req.user
    .split('\n')
    .map((l) => /^\[(\d{1,2}(?::\d{2}){1,2})\]\s*(.+)$/.exec(l.trim()))
    .filter((m): m is RegExpExecArray => !!m)
    .map((m) => ({ time: m[1]!, text: m[2]!.slice(0, 160) }));
  const isTopics = 'topics' in ((req.schema.properties as Record<string, unknown>) ?? {});
  const first = lines[0] ?? { time: '00:00', text: 'Sem conteúdo.' };
  const out = isTopics
    ? {
        topics: lines.slice(0, 3).map((l, i) => ({ title: `Tópico ${i + 1}`, time: l.time, subtopics: [{ title: l.text.slice(0, 40), time: l.time, points: [l.text] }] })),
        decisions: [],
        actions: [],
        dates: [],
        numbers: [],
        keywords: [],
      }
    : { overview: `Resumo (mock): ${first.text}`, points: lines.slice(0, 5).map((l) => ({ text: l.text, time: l.time })) };
  const text = JSON.stringify(out);
  return { text, inputTokens: Math.ceil(req.user.length / 4), outputTokens: Math.ceil(text.length / 4) };
};
