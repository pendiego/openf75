// Host OS only. F75 key names (Ctrl, Win) describe the keyboard and stay as they are.
export const isMac = /Mac|iPhone|iPad|iPod/.test(navigator.userAgent);

// Modifier for host shortcuts: Cmd on macOS, Ctrl elsewhere.
export const modLabel = isMac ? '⌘' : 'Ctrl';
