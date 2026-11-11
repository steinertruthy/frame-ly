import { getCurrentWindow } from '@tauri-apps/api/window';
import { commands } from '$lib/ipc/commands';
import type { AudioDevice, Display, Permission, Permissions } from '$lib/ipc/types';
import { logUi } from '$lib/ipc/log';

/**
 * OS-level state: permissions and capture devices. Re-read whenever the window regains
 * focus, which is exactly when the user comes back from System Settings or plugs a device.
 */
class SystemStore {
  permissions = $state<Permissions | null>(null);
  displays = $state<Display[]>([]);
  audioInputs = $state<AudioDevice[]>([]);
  /** Permissions requested in this session — macOS may need a relaunch to apply them. */
  requested = $state<Set<Permission>>(new Set());

  async init(): Promise<void> {
    await this.refresh();
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) void this.refresh();
    });
  }

  async refresh(): Promise<void> {
    const [p, d, a] = await Promise.allSettled([commands.getPermissions(), commands.listDisplays(), commands.listAudioInputs()]);
    if (p.status === 'fulfilled') this.permissions = p.value;
    if (d.status === 'fulfilled') this.displays = d.value;
    if (a.status === 'fulfilled') this.audioInputs = a.value;
    for (const r of [p, d, a]) if (r.status === 'rejected') logUi('warn', `system refresh failed: ${JSON.stringify(r.reason)}`);
  }

  async request(permission: Permission): Promise<void> {
    this.requested = new Set([...this.requested, permission]);
    try {
      const state = await commands.requestPermission(permission);
      if (this.permissions) this.permissions = { ...this.permissions, [permission]: state };
    } catch (e) {
      logUi('error', `request_permission(${permission}) failed: ${JSON.stringify(e)}`);
    }
  }

  get screenReady(): boolean {
    return this.permissions?.screen === 'granted';
  }
}

export const system = new SystemStore();
