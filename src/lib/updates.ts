/**
 * Release notes for the update notice: the CHANGELOG part of the GitHub release body (everything
 * before the first `---`), as plain lines. Markdown is reduced to text; nothing is rendered as HTML.
 */
export function releaseNotes(body: string | null): { text: string; heading: boolean }[] {
  if (!body) return [];
  const main = body.split(/\r?\n---\s*\r?\n/)[0] ?? '';
  return main
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)
    .map((l) => {
      const heading = /^#{1,6}\s/.test(l);
      const text = l
        .replace(/^#{1,6}\s+/, '')
        .replace(/^[-*]\s+/, '')
        .replace(/\*\*([^*]+)\*\*/g, '$1')
        .replace(/`([^`]+)`/g, '$1')
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
      return { text, heading };
    })
    .slice(0, 40);
}
