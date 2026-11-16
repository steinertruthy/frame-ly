import { defineMiddleware } from 'astro:middleware';
import { externalLinks } from './lib/links';

/** External links open in a new tab (see lib/links.ts). */
export const onRequest = defineMiddleware(async (context, next) => {
  const response = await next();
  if (!response.headers.get('content-type')?.includes('text/html')) return response;
  const origin = context.site?.origin ?? '';
  const html = externalLinks(await response.text(), origin);
  return new Response(html, { status: response.status, headers: response.headers });
});
