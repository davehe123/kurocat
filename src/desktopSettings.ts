import { invoke } from '@tauri-apps/api/core';
import type { KeyboardOrientation } from './catEngine';
import type { KeyboardLayoutId } from './keyboardLayout';

export interface AppSettings {
  x: number | null;
  y: number | null;
  scale: number;
  alwaysOnTop: boolean;
  clickThroughLocked: boolean;
  showKeyBubbles: boolean;
  soundEnabled: boolean;
  autostart: boolean;
  catBreed: string;
  mcpEnabled: boolean;
  mcpPort: number;
  keyboardLayout: KeyboardLayoutId;
  keyboardOrientation: KeyboardOrientation;
  privacyPasswordGuard: boolean;
}

export type SettingsPatch = Partial<Omit<AppSettings, 'x' | 'y'>>;

let pending: Promise<unknown> = Promise.resolve();

export function saveDesktopSettings(patch: SettingsPatch): Promise<AppSettings> {
  const result = pending.then(() => invoke<AppSettings>('update_settings', { patch }));
  pending = result.catch(() => undefined);
  return result;
}
