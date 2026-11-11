/** English strings. This file defines the canonical key set; every locale must provide all keys. */
export const en = {
  'app.name': 'FrameLy',
  'nav.home': 'Home',
  'nav.settings': 'Settings',

  'home.greeting': 'Welcome to FrameLy',
  'home.subtitle': 'Record your screen and capture screenshots in one click.',
  'home.folders': 'Your folders',
  'home.folders.recordings': 'Recordings',
  'home.folders.screenshots': 'Screenshots',
  'home.open': 'Open',
  'home.system': 'This Mac/PC',

  'settings.title': 'Settings',
  'settings.general': 'General',
  'settings.appearance': 'Appearance',
  'settings.storage': 'Storage',
  'settings.system': 'System',
  'settings.language': 'Language',
  'settings.language.hint': 'Interface and menu language.',
  'settings.language.system': 'System',
  'settings.theme': 'Theme',
  'settings.theme.hint': 'Follow the system or force a look.',
  'settings.theme.system': 'System',
  'settings.theme.light': 'Light',
  'settings.theme.dark': 'Dark',
  'settings.recordingsDir': 'Recordings folder',
  'settings.screenshotsDir': 'Screenshots folder',
  'settings.change': 'Change…',
  'settings.show': 'Show',
  'settings.chooseFolder': 'Choose a folder',
  'settings.version': 'Version',
  'settings.platform': 'Platform',
  'settings.logs': 'Diagnostic logs',
  'settings.logs.hint': 'Structured logs to help investigate problems.',
  'settings.saved': 'Settings saved',

  'error.generic': 'Something went wrong.',
  'error.invalid_settings': 'These settings are not valid.',
  'error.io': 'Could not access the file system.',
  'error.runtime': 'The operation could not be completed.',
} as const;

export type MessageKey = keyof typeof en;
export type Messages = Record<MessageKey, string>;
