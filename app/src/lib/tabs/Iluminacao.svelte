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

<section class="panel">
  <label>
    Efeito
    <select bind:value={row}>
      {#each effects as e, i (e.row)}
        <option value={i}>{e.row + 1}. {e.name}</option>
      {/each}
    </select>
  </label>

  {#if cur?.brightness}
    <label>Brilho <input type="range" min="0" max="9" bind:value={brightness} /> {brightness}</label>
  {/if}
  {#if cur?.speed}
    <label>Velocidade <input type="range" min="0" max="4" bind:value={speed} /> {speed}</label>
  {/if}

  {#if cur?.colour}
    <div class="row">
      <span>Cor</span>
      {#each light?.palette[row] ?? [] as rgb, i (i)}
        <button
          class="sw"
          class:on={mode === 'slot' && slot === i}
          style:background={hex(rgb)}
          aria-label={`Cor ${i + 1}`}
          onclick={() => ((mode = 'slot'), (slot = i))}
        ></button>
      {/each}
      <label class="sw-custom">
        <input type="color" bind:value={custom} oninput={() => (mode = 'custom')} />
      </label>
      <label><input type="checkbox" checked={mode === 'random'} onchange={(e) => (mode = e.currentTarget.checked ? 'random' : 'slot')} /> Aleatório</label>
    </div>
  {/if}

  <button class="primary" onclick={apply}>Aplicar</button>
</section>

<style>
  .panel { display: grid; gap: 14px; max-width: 560px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .sw { width: 28px; height: 28px; border-radius: 6px; border: 2px solid transparent; cursor: pointer; }
  .sw.on { border-color: var(--fg); }
  .sw-custom input { width: 36px; height: 28px; padding: 0; border: none; background: none; }
</style>
