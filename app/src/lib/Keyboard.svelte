<script lang="ts">
  import type { Key } from './api';

  let {
    keys,
    fill,
    labelOf,
    selected = new Set<number>(),
    onpick,
  }: {
    keys: Key[];
    fill: (matrix: number) => string;
    labelOf: (k: Key) => string;
    selected?: Set<number>;
    onpick?: (matrix: number, shift: boolean) => void;
  } = $props();

  const visible = $derived(keys.filter((k) => k.rect[2] > 0 && k.name !== 'Wheel'));

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

<svg viewBox="0 0 660 290" class="kb" role="group" aria-label="Teclado">
  {#each visible as k (k.matrix)}
    <g
      role="button"
      tabindex="-1"
      class:sel={selected.has(k.matrix)}
      onpointerdown={(e) => down(e, k.matrix)}
      onpointerenter={() => enter(k.matrix)}
    >
      <rect x={k.rect[0]} y={k.rect[1]} width={k.rect[2]} height={k.rect[3]} rx="4" style:fill={fill(k.matrix)} />
      <text x={k.rect[0] + k.rect[2] / 2} y={k.rect[1] + k.rect[3] / 2} text-anchor="middle" dominant-baseline="central">
        {labelOf(k)}
      </text>
    </g>
  {/each}
</svg>

<style>
  .kb {
    width: 100%;
    max-width: 860px;
    user-select: none;
    touch-action: none;
  }
  rect {
    stroke: #2a2f3a;
    stroke-width: 1.5;
  }
  g.sel rect {
    stroke: #fff;
    stroke-width: 3;
  }
  text {
    font-size: 11px;
    fill: #111;
    pointer-events: none;
  }
</style>
