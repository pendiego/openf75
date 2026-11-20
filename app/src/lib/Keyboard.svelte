<script lang="ts">
  import type { Key } from './api';

  let {
    keys,
    fill,
    labelOf,
    selected = new Set<number>(),
    onpick,
    lit = false,
  }: {
    keys: Key[];
    fill: (matrix: number) => string;
    labelOf: (k: Key) => string;
    selected?: Set<number>;
    onpick?: (matrix: number, shift: boolean) => void;
    lit?: boolean;
  } = $props();

  const visible = $derived(keys.filter((k) => k.rect[2] > 0 && k.name !== 'Wheel'));

  const PAD = 12;
  const plate = $derived.by(() => {
    if (!visible.length) return { x: 0, y: 0, w: 660, h: 290 };
    const x = Math.min(...visible.map((k) => k.rect[0])) - PAD;
    const y = Math.min(...visible.map((k) => k.rect[1])) - PAD;
    const x2 = Math.max(...visible.map((k) => k.rect[2])) + PAD;
    const y2 = Math.max(...visible.map((k) => k.rect[3])) + PAD;
    return { x, y, w: x2 - x, h: y2 - y };
  });

  // WCAG relative luminance picks the legend ink: dark on light keycaps, light on dark ones.
  function ink(hex: string): string {
    const [r, g, b] = [1, 3, 5].map((i) => {
      const c = parseInt(hex.slice(i, i + 2), 16) / 255;
      return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.18 ? 'var(--accent-ink)' : 'var(--text)';
  }

  const glow = (f: string) => (lit && f !== '#000000' ? `drop-shadow(0 0 4px ${f})` : 'none');

  // Mono advance is ~0.6em: shrink long legends (Delete, Ctrl+Shift+A) to the cap width.
  const fit = (s: string, w: number, max: number) => Math.min(max, (w - 3) / (s.length * 0.6));

  let painting = false;
  function down(e: PointerEvent, m: number) {
    painting = true;
    onpick?.(m, e.shiftKey);
  }
  function enter(m: number) {
    if (painting) onpick?.(m, false);
  }
</script>

<svelte:window onpointerup={() => (painting = false)} />

<svg viewBox="{plate.x} {plate.y} {plate.w} {plate.h}" class="kb" role="group" aria-label="Teclado">
  <rect class="plate" x={plate.x} y={plate.y} width={plate.w} height={plate.h} rx="14" />
  <!-- shortcut: 81 tab stops, upgrade to roving tabindex + arrow keys if keyboard navigation matters -->
  {#each visible as k (k.matrix)}
    {@const w = k.rect[2] - k.rect[0]}
    {@const h = k.rect[3] - k.rect[1]}
    {@const f = fill(k.matrix)}
    {@const lines = labelOf(k).split('\n')}
    <g
      class="key"
      role="button"
      tabindex="0"
      aria-label={k.name}
      aria-pressed={selected.has(k.matrix)}
      transform="translate({k.rect[0]} {k.rect[1]})"
      onpointerdown={(e) => down(e, k.matrix)}
      onpointerenter={() => enter(k.matrix)}
      onkeydown={(e) => {
        if (e.key !== 'Enter' && e.key !== ' ') return;
        e.preventDefault();
        onpick?.(k.matrix, e.shiftKey);
      }}
    >
      <title>{lines.join(': ')}</title>
      <rect class="cap" width={w} height={h} rx="4" style:fill={f} style:filter={glow(f)} />
      <g class="ring" aria-hidden="true">
        <rect class="ring-dark" x="0.75" y="0.75" width={w - 1.5} height={h - 1.5} rx="3" />
        <rect class="ring-light" x="2.25" y="2.25" width={w - 4.5} height={h - 4.5} rx="2" />
      </g>
      <rect class="focus" x="-4" y="-4" width={w + 8} height={h + 8} rx="7" />
      <svg class="legend" x="0" y="0" width={w} height={h} pointer-events="none" style:fill={ink(f)}>
        {#if lines.length === 1}
          <text x={w / 2} y={h / 2} text-anchor="middle" dominant-baseline="central" font-size={fit(lines[0], w, 8.5)}>{lines[0]}</text>
        {:else}
          <text x={w / 2} y={h * 0.3} text-anchor="middle" dominant-baseline="central" font-size={fit(lines[0], w, 6)} opacity="0.7">{lines[0]}</text>
          <text x={w / 2} y={h * 0.65} text-anchor="middle" dominant-baseline="central" font-size={fit(lines[1], w, 8)}>{lines[1]}</text>
        {/if}
      </svg>
    </g>
  {/each}
</svg>

<style>
  .kb {
    display: block;
    width: 100%;
    user-select: none;
    touch-action: none;
  }
  .plate {
    fill: var(--rail);
    stroke: var(--line);
    stroke-width: 1;
  }
  .cap {
    stroke: var(--line);
    stroke-width: 1;
  }
  .key {
    cursor: pointer;
    outline: none;
  }
  .ring {
    opacity: 0;
  }
  .key[aria-pressed='true'] .ring {
    opacity: 1;
  }
  .ring rect {
    fill: none;
    stroke-width: 1.5;
  }
  .ring-dark {
    stroke: var(--bg);
  }
  .ring-light {
    stroke: var(--text);
  }
  .focus {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    opacity: 0;
  }
  .key:focus-visible .focus {
    opacity: 1;
  }
  .legend text {
    font-family: var(--mono);
  }
</style>
