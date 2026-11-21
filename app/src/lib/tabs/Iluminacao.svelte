<script lang="ts">
  import { onMount } from 'svelte';
  import Keyboard from '../Keyboard.svelte';
  import Num from '../Num.svelte';
  import AsyncButton from '../AsyncButton.svelte';
  import { isReactive, makeFx, type RGB } from '../fx';
  import { api, hex, unhex, errMsg, type Effect, type Key, type Lighting, type Colour } from '../api';

  let { notify }: { notify: (m: string) => void } = $props();

  let keys = $state<Key[]>([]);
  let effects = $state<Effect[]>([]);
  let light = $state<Lighting | null>(null);
  let row = $state(0);
  let brightness = $state(9);
  let speed = $state(2);
  let mode = $state<'slot' | 'custom' | 'random'>('slot');
  let slot = $state(0);
  let custom = $state('#ff0000');
  let loading = $state(true);

  const cur = $derived(effects[row]);
  const noOptions = $derived(!cur?.brightness && !cur?.speed && !cur?.colour);

  const unit = (c: [number, number, number]): RGB => [c[0] / 255, c[1] / 255, c[2] / 255];

  // Animated simulation of the chosen settings. Rebuilt on every change; LEDs stay off until the keyboard is read.
  const fx = $derived.by(() => {
    if (!cur || loading) return undefined;
    const fixed = mode === 'custom' ? unhex(custom) : (light?.palette[row]?.[slot] ?? [255, 255, 255]);
    return makeFx({
      name: cur.name,
      colour: !cur.colour || mode === 'random' ? null : unit(fixed),
      speed: cur.speed ? speed : 2,
      brightness: cur.brightness ? brightness : 9,
      perkey: light?.perkey.map(unit),
    });
  });

  onMount(async () => {
    try {
      [keys, effects] = await Promise.all([api.layout(), api.effects()]);
      light = await api.readLighting();
      if (light.row !== null) row = light.row;
      const s = light.slots[row];
      if (s) {
        brightness = s[0];
        speed = s[1];
        // Colour nibble: 0-6 palette slot, 7 random (Device::apply_effect).
        if (s[2] === 7) mode = 'random';
        else if (s[2] <= 6) slot = s[2];
      }
    } catch (e) {
      notify(errMsg(e));
    } finally {
      loading = false;
    }
  });

  async function apply() {
    // Snapshot: the controls stay editable during the HID write.
    const r = row, b = brightness, s = speed, m = mode, sl = slot, rgb = unhex(custom);
    const colour: Colour = m === 'random' ? 'random' : m === 'custom' ? { custom: rgb } : { slot: sl };
    await api.applyEffect(r, b, s, colour);
    // No re-read: the receiver often misses the per-key read (0x42) right after a write.
    if (light) {
      light.row = r;
      const curSlot = light.slots[r]?.[2] ?? 0;
      const nibble = m === 'random' ? 7 : m === 'slot' ? sl : curSlot === 7 ? 0 : curSlot;
      light.slots[r] = [b, s, nibble];
      if (m === 'custom') light.palette[r][nibble] = rgb;
    }
    notify('Efeito aplicado');
  }
</script>

<div class="workspace" aria-busy={loading}>
  <div class="stage">
    <div class="stage-body">
      <!-- The model shows while the keyboard is read; LEDs light up once the reading arrives. -->
      <Keyboard {keys} labelOf={(k) => k.name} {fx} />
      {#if loading}
        <p class="hint">Lendo do teclado…</p>
        <div class="gallery" aria-hidden="true">
          {#each Array(6) as _}<span class="skeleton tile"></span>{/each}
        </div>
      {:else}
        <div class="gallery" role="group" aria-label="Efeito">
          {#each effects as e, i (e.row)}
            <button aria-pressed={row === i} onclick={() => (row = i)}>
              <span class="num">{e.row + 1}</span>
              <span class="name">{e.name}</span>
              {#if isReactive(e.name)}<span class="tag">Reage à digitação</span>{/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <aside class="inspector">
    <div>
      <h2>{cur?.name ?? ''}</h2>
      <p class="hint">Efeito {row + 1}</p>
      {#if cur && isReactive(cur.name)}
        <p class="hint">O teclado fica apagado até você pressionar uma tecla: a luz nasce de cada tecla digitada.</p>
      {/if}
    </div>

    {#if cur?.brightness}
      <label class="field">
        <span>Brilho</span>
        <input type="range" min="0" max="9" bind:value={brightness} />
        <Num value={brightness} />
      </label>
    {/if}
    {#if cur?.speed}
      <label class="field">
        <span>Velocidade</span>
        <input type="range" min="0" max="4" bind:value={speed} />
        <Num value={speed} />
      </label>
    {/if}

    {#if cur?.colour}
      <div class="colour">
        <span class="label">Cor</span>
        <div class="row">
          {#each light?.palette[row] ?? [] as rgb, i (i)}
            <button
              class="swatch"
              aria-pressed={mode === 'slot' && slot === i}
              aria-label={`Cor ${i + 1}`}
              style:background={hex(rgb)}
              onclick={() => ((mode = 'slot'), (slot = i))}
            ></button>
          {/each}
          <input
            class="custom"
            class:sel={mode === 'custom'}
            type="color"
            aria-label="Cor personalizada"
            bind:value={custom}
            oninput={() => (mode = 'custom')}
          />
        </div>
        <label class="check">
          <input type="checkbox" checked={mode === 'random'} onchange={(e) => (mode = e.currentTarget.checked ? 'random' : 'slot')} />
          Aleatório
        </label>
      </div>
    {/if}

    {#if noOptions}
      <p class="hint">Este efeito não tem ajustes.</p>
    {/if}

    <AsyncButton class="primary apply" label="Aplicar" busyLabel="Aplicando…" run={apply} {notify} />
  </aside>
</div>

<style>
  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 8px;
  }
  .gallery button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    min-height: 72px;
    padding: 12px;
    text-align: left;
  }
  .gallery .num { color: var(--muted); }
  .tile { height: 72px; }
  .gallery button[aria-pressed="true"] .num { color: inherit; }
  .name { overflow-wrap: anywhere; }
  .tag { font-size: 11px; color: var(--muted); }
  .gallery button[aria-pressed="true"] .tag { color: inherit; }
  .colour { display: grid; gap: 8px; }
  .colour .row { gap: 6px; }
  .custom.sel { outline: 2px solid var(--text); outline-offset: 2px; }
  .check { display: flex; align-items: center; gap: 8px; }
  .inspector :global(button.apply) { width: 100%; }
</style>
