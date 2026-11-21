const rm = typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;

/** Duration for Svelte transitions/tweens: they run in JS/WAAPI, which the app.css reduced-motion block can't stop. */
export const dur = (ms: number) => (rm?.matches ? 0 : ms);
