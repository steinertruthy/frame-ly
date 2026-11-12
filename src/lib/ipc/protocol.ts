/**
 * URL of an in-memory image served by FrameLy's `framely://` protocol.
 * WebKit (macOS) uses the custom scheme directly; WebView2 (Windows) maps it to http://<scheme>.localhost.
 */
export function framelyUrl(path: string): string {
  const windows = navigator.userAgent.includes('Windows');
  return `${windows ? 'http://framely.localhost' : 'framely://localhost'}/${path}`;
}
