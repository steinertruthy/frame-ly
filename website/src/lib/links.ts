/**
 * Every link that leaves the site opens in a new tab: adds target/rel to absolute http(s) links in
 * the rendered HTML (components, changelog Markdown, release links). Runs at build time for the
 * static pages. Links inside the site are relative, so they're left alone.
 */
export function externalLinks(html: string, siteOrigin: string): string {
  return html.replace(/<a\b([^>]*?)\bhref="(https?:\/\/[^"]+)"([^>]*)>/g, (tag, before: string, href: string, after: string) => {
    if (href.startsWith(siteOrigin) || /\btarget=/.test(before + after)) return tag;
    const rel = /\brel="/.test(before + after) ? '' : ' rel="noopener noreferrer"';
    return `<a${before}href="${href}"${after} target="_blank"${rel}>`;
  });
}
