// UI colour theme: 'system' follows the OS, 'light' and 'dark' force it.
// Dark is the default look (app.css :root); light lives under :root[data-theme="light"].
export type ThemeMode = 'system' | 'light' | 'dark';

const KEY = 'openf75.ui-theme';
const media = typeof matchMedia === 'function' ? matchMedia('(prefers-color-scheme: light)') : null;

function readMode(): ThemeMode {
  try {
    const v = localStorage.getItem(KEY);
    return v === 'light' || v === 'dark' ? v : 'system';
  } catch {
    return 'system';
  }
}

export const theme = $state<{ mode: ThemeMode }>({ mode: readMode() });

export function resolved(): 'light' | 'dark' {
  if (theme.mode !== 'system') return theme.mode;
  return media?.matches ? 'light' : 'dark';
}

function apply() {
  document.documentElement.dataset.theme = resolved();
}

export function setTheme(mode: ThemeMode) {
  theme.mode = mode;
  try {
    localStorage.setItem(KEY, mode);
  } catch {
    // Storage can be unavailable; the choice then lasts for this session only.
  }
  apply();
}

// Follow OS changes while in 'system' mode; apply() is a no-op for forced modes.
media?.addEventListener('change', apply);
apply();
