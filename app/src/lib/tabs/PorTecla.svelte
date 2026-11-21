<script lang="ts">
  import { onMount } from 'svelte';
  import Keyboard from '../Keyboard.svelte';
  import { api, hex, unhex, errMsg, type Key } from '../api';
  import AsyncButton from '../AsyncButton.svelte';

  let { notify }: { notify: (m: string) => void } = $props();

  let keys = $state<Key[]>([]);
  let colours = $state<[number, number, number][]>(Array.from({ length: 126 }, () => [0, 0, 0]));
  let selected = $state(new Set<number>());
  let paint = $state('#ff0000');
  let mode = $state<'paint' | 'select'>('paint');
  let dirty = $state(new Map<number, [number, number, number]>());
  let loading = $state(true);

  onMount(async () => {
    try {
      keys = await api.layout();
      const light = await api.readLighting();
      colours = light.perkey;
    } catch (e) {
      notify(errMsg(e));
    } finally {
      loading = false;
    }
  });

  function pick(m: number, shift: boolean) {
    if (mode === 'select') {
      const next = new Set(selected);
      if (shift || next.has(m)) next.delete(m);
      else next.add(m);
      selected = next;
      return;
    }
    colours[m] = unhex(paint);
    dirty.set(m, colours[m]);
    dirty = new Map(dirty);
  }

  function fillAll() {
    const rgb = unhex(paint);
    colours = colours.map(() => rgb);
    for (let m = 0; m < 126; m++) dirty.set(m, rgb);
    dirty = new Map(dirty);
  }

  async function apply() {
    await api.applyPerkey([...dirty.entries()].map(([m, rgb]) => [m, rgb]));
    dirty = new Map();
    notify('Cores por tecla aplicadas');
  }

  const fill = (m: number) => hex(colours[m] ?? [0, 0, 0]);
</script>

<div class="workspace" aria-busy={loading}>
  <div class="stage">
    <div class="strip">
      <div class="seg">
        <label><input type="radio" bind:group={mode} value="paint" /> Pintar</label>
        <label><input type="radio" bind:group={mode} value="select" /> Selecionar</label>
      </div>
      <span class="hint push">Clique ou arraste sobre as teclas para pintar</span>
    </div>
    <div class="stage-body hero">
      {#if loading}<p class="hint">Lendo do teclado…</p>{/if}
      <Keyboard {keys} tint={fill} {selected} labelOf={(k) => k.name} onpick={pick} />
    </div>
  </div>

  <aside class="inspector">
    <div>
      <h2>Cor</h2>
      <p class="hint">Cor usada ao pintar as teclas</p>
    </div>
    <div class="stack">
      <span class="label">Pincel</span>
      <div class="row">
        <input type="color" bind:value={paint} aria-label="Cor do pincel" />
        <button onclick={fillAll}>Preencher tudo</button>
      </div>
    </div>
    <AsyncButton class="primary wide" label={`Aplicar (${dirty.size})`} busyLabel="Aplicando…" run={apply} {notify} disabled={dirty.size === 0} />
  </aside>
</div>

<style>
  .push {
    margin-left: auto;
  }
  .inspector :global(button.wide) {
    width: 100%;
    margin-top: auto;
  }
</style>
