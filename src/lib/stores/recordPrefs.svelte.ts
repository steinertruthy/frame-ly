/** Per-session choices on Home: monitor for full-screen recordings ('' = the one under the pointer) and screenshot kind. */
export const recordPrefs = $state<{ displayId: string; shotMode: 'region' | 'window' | 'screen' }>({ displayId: '', shotMode: 'region' });
