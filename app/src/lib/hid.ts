// HID keyboard usages (US layout names). `code` is KeyboardEvent.code.
export type HidKey = { code: string; label: string; usage: number };

const letters: HidKey[] = Array.from({ length: 26 }, (_, i) => ({
  code: 'Key' + String.fromCharCode(65 + i),
  label: String.fromCharCode(65 + i),
  usage: 0x04 + i,
}));
const digits: HidKey[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0].map((d, i) => ({
  code: 'Digit' + d,
  label: String(d),
  usage: 0x1e + i,
}));
const fkeys: HidKey[] = Array.from({ length: 12 }, (_, i) => ({
  code: 'F' + (i + 1),
  label: 'F' + (i + 1),
  usage: 0x3a + i,
}));
const named: HidKey[] = [
  { code: 'Enter', label: 'Enter', usage: 0x28 },
  { code: 'Escape', label: 'Esc', usage: 0x29 },
  { code: 'Backspace', label: 'Backspace', usage: 0x2a },
  { code: 'Tab', label: 'Tab', usage: 0x2b },
  { code: 'Space', label: 'Espaço', usage: 0x2c },
  { code: 'Minus', label: '-', usage: 0x2d },
  { code: 'Equal', label: '=', usage: 0x2e },
  { code: 'BracketLeft', label: '[', usage: 0x2f },
  { code: 'BracketRight', label: ']', usage: 0x30 },
  { code: 'Backslash', label: '\\', usage: 0x31 },
  { code: 'Semicolon', label: ';', usage: 0x33 },
  { code: 'Quote', label: "'", usage: 0x34 },
  { code: 'Backquote', label: '`', usage: 0x35 },
  { code: 'Comma', label: ',', usage: 0x36 },
  { code: 'Period', label: '.', usage: 0x37 },
  { code: 'Slash', label: '/', usage: 0x38 },
  { code: 'CapsLock', label: 'Caps', usage: 0x39 },
  { code: 'PrintScreen', label: 'PrtSc', usage: 0x46 },
  { code: 'ScrollLock', label: 'ScrLk', usage: 0x47 },
  { code: 'Pause', label: 'Pause', usage: 0x48 },
  { code: 'Insert', label: 'Ins', usage: 0x49 },
  { code: 'Home', label: 'Home', usage: 0x4a },
  { code: 'PageUp', label: 'PgUp', usage: 0x4b },
  { code: 'Delete', label: 'Del', usage: 0x4c },
  { code: 'End', label: 'End', usage: 0x4d },
  { code: 'PageDown', label: 'PgDn', usage: 0x4e },
  { code: 'ArrowRight', label: '→', usage: 0x4f },
  { code: 'ArrowLeft', label: '←', usage: 0x50 },
  { code: 'ArrowDown', label: '↓', usage: 0x51 },
  { code: 'ArrowUp', label: '↑', usage: 0x52 },
  { code: 'NumLock', label: 'NumLk', usage: 0x53 },
  { code: 'ContextMenu', label: 'Menu', usage: 0x65 },
];
const modifiers: HidKey[] = [
  { code: 'ControlLeft', label: 'LCtrl', usage: 0xe0 },
  { code: 'ShiftLeft', label: 'LShift', usage: 0xe1 },
  { code: 'AltLeft', label: 'LAlt', usage: 0xe2 },
  { code: 'MetaLeft', label: 'LWin', usage: 0xe3 },
  { code: 'ControlRight', label: 'RCtrl', usage: 0xe4 },
  { code: 'ShiftRight', label: 'RShift', usage: 0xe5 },
  { code: 'AltRight', label: 'RAlt', usage: 0xe6 },
  { code: 'MetaRight', label: 'RWin', usage: 0xe7 },
];

export const HID_KEYS: HidKey[] = [...letters, ...digits, ...fkeys, ...named, ...modifiers];

const byCode = new Map(HID_KEYS.map((k) => [k.code, k]));
const byUsage = new Map(HID_KEYS.map((k) => [k.usage, k]));

export function fromEventCode(code: string): HidKey | undefined {
  return byCode.get(code);
}

export function labelOf(usage: number): string {
  return byUsage.get(usage)?.label ?? `0x${usage.toString(16).padStart(2, '0')}`;
}

// Modifier bits for KeyAction::Key.mods (LCtrl 0x01 … RGui 0x80).
export const MOD_BITS: { bit: number; label: string }[] = [
  { bit: 0x01, label: 'Ctrl' },
  { bit: 0x02, label: 'Shift' },
  { bit: 0x04, label: 'Alt' },
  { bit: 0x08, label: 'Win' },
  { bit: 0x10, label: 'RCtrl' },
  { bit: 0x20, label: 'RShift' },
  { bit: 0x40, label: 'RAlt' },
  { bit: 0x80, label: 'RWin' },
];

export const MEDIA_KEYS: { usage: number; label: string }[] = [
  { usage: 0xcd, label: 'Play/Pausa' },
  { usage: 0xb5, label: 'Próxima' },
  { usage: 0xb6, label: 'Anterior' },
  { usage: 0xb7, label: 'Parar' },
  { usage: 0xe2, label: 'Mudo' },
  { usage: 0xe9, label: 'Volume +' },
  { usage: 0xea, label: 'Volume −' },
  { usage: 0x192, label: 'Calculadora' },
  { usage: 0x18a, label: 'E-mail' },
  { usage: 0x223, label: 'Navegador' },
  { usage: 0x194, label: 'Meu computador' },
];
