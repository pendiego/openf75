import { invoke } from '@tauri-apps/api/core';

export type Key = { name: string; matrix: number; rect: [number, number, number, number]; default_usage: number | null };

export type Effect = { row: number; hw: number; name: string; speed: boolean; brightness: boolean; colour: boolean };

export type Status = {
  connected: boolean;
  kind: 'cable' | 'dongle' | null;
  fw: number | null;
  psd: string | null;
  online: boolean | null;
  oem_running: boolean;
  writable_error: string | null;
};

export type Lighting = {
  row: number | null;
  slots: [number, number, number][];
  palette: [number, number, number][][];
  perkey: [number, number, number][];
};

export type KeyAction =
  | { type: 'Key'; mods: number; usage: number }
  | { type: 'Consumer'; usage: number }
  | { type: 'Macro'; mode: number; count: number; index: number }
  | { type: 'Fn' }
  | { type: 'Disabled' }
  | { type: 'Raw'; bytes: number[] };

export type MacroEvent = { usage: number; down: boolean; delay_ms: number };
export type Macro = { name: string; events: MacroEvent[] };

export type Colour = { Slot: number } | { Custom: [number, number, number] } | 'Random';

export const api = {
  layout: () => invoke<Key[]>('layout'),
  effects: () => invoke<Effect[]>('effects'),
  status: () => invoke<Status>('status'),
  readLighting: () => invoke<Lighting>('read_lighting'),
  applyEffect: (row: number, brightness: number, speed: number, colour: Colour) =>
    invoke<void>('apply_effect', { row, brightness, speed, colour }),
  applyPerkey: (colours: [number, [number, number, number]][]) => invoke<void>('apply_perkey', { colours }),
  readLayer: (layer: number) => invoke<KeyAction[]>('read_layer', { layer }),
  setKey: (layer: number, matrix: number, action: KeyAction) => invoke<void>('set_key', { layer, matrix, action }),
  listMacros: () => invoke<Macro[]>('list_macros'),
  saveMacros: (macros: Macro[]) => invoke<void>('save_macros', { macros }),
  deleteMacro: (index: number) => invoke<void>('delete_macro', { index }),
  setSleep: (secs: number | null) => invoke<void>('set_sleep', { secs }),
  setDebounce: (value: number) => invoke<void>('set_debounce', { value }),
  getPower: () => invoke<[number | null, number]>('get_power'),
  factoryReset: () => invoke<void>('factory_reset'),
  backupNow: () => invoke<string>('backup_now'),
  restoreBackup: (path: string) => invoke<void>('restore_backup', { path }),
  listProfiles: () => invoke<string[]>('list_profiles'),
  saveProfile: (name: string) => invoke<void>('save_profile', { name }),
  applyProfile: (name: string) => invoke<void>('apply_profile', { name }),
  deleteProfile: (name: string) => invoke<void>('delete_profile', { name }),
  exportProfile: (name: string, path: string) => invoke<void>('export_profile', { name, path }),
  importProfile: (path: string) => invoke<string>('import_profile', { path }),
  audioStart: (gain: number) => invoke<void>('audio_start', { gain }),
  audioStop: () => invoke<void>('audio_stop'),
};

export const hex = (rgb: [number, number, number]) =>
  '#' + rgb.map((c) => c.toString(16).padStart(2, '0')).join('');

export const unhex = (h: string): [number, number, number] => [
  parseInt(h.slice(1, 3), 16),
  parseInt(h.slice(3, 5), 16),
  parseInt(h.slice(5, 7), 16),
];

export function errMsg(e: unknown): string {
  return typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
}
