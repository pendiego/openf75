<script lang="ts">
  import { onMount } from 'svelte';
  import { api, hex, unhex, errMsg, type Effect, type Lighting, type Colour } from '../api';

  let { notify }: { notify: (m: string) => void } = $props();

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

  onMount(async () => {
    try {
      effects = await api.effects();
      light = await api.readLighting();
      if (light.row !== null) row = light.row;
      const s = light.slots[row];
      if (s) {
        brightness = s[0];
        speed = s[1];
      }
    } catch (e) {
      notify(errMsg(e));
    } finally {
      loading = false;
    }
  });

  async function apply() {
    const colour: Colour = mode === 'random' ? 'Random' : mode === 'custom' ? { Custom: unhex(custom) } : { Slot: slot };
    try {
      await api.applyEffect(row, brightness, speed, colour);
      light = await api.readLighting();
      notify('Efeito aplicado');
    } catch (e) {
      notify(errMsg(e));
    }
  }
</script>

<section class="panel" aria-busy={loading}>
  {#if loading}
    <p class="hint">Lendo do teclado…</p>
  {:else}
    <div class="layout">
      <div class="effects" role="group" aria-label="Efeito">
        {#each effects as e, i (e.row)}
          <button aria-pressed={row === i} onclick={() => (row = i)}>
            <span class="num">{e.row + 1}</span>
            {e.name}
          </button>
        {/each}
      </div>

      <div class="card">
        {#if cur?.brightness}
          <label class="field">
            <span>Brilho</span>
            <input type="range" min="0" max="9" bind:value={brightness} />
            <output class="num">{brightness}</output>
          </label>
        {/if}
        {#if cur?.speed}
          <label class="field">
            <span>Velocidade</span>
            <input type="range" min="0" max="4" bind:value={speed} />
            <output class="num">{speed}</output>
          </label>
        {/if}

        {#if cur?.colour}
          <div class="row">
            <span class="hint">Cor</span>
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
            <label><input type="checkbox" checked={mode === 'random'} onchange={(e) => (mode = e.currentTarget.checked ? 'random' : 'slot')} /> Aleatório</label>
          </div>
        {/if}

        <button class="primary apply" onclick={apply}>Aplicar</button>
      </div>
    </div>
  {/if}
</section>

<style>
  .panel { container-type: inline-size; }
  .layout { display: grid; gap: 16px; align-items: start; }
  @container (width >= 760px) {
    .layout { grid-template-columns: minmax(0, 1fr) 300px; }
  }
  .effects { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 6px; }
  .effects button { display: flex; gap: 8px; align-items: baseline; text-align: left; }
  .effects .num { color: var(--muted); }
  .effects button[aria-pressed="true"] .num { color: inherit; }
  .custom.sel { outline: 2px solid var(--text); outline-offset: 2px; }
  .apply { width: 100%; }
</style>
