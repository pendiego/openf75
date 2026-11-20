<script lang="ts">
  import { onMount } from 'svelte';
  import Keyboard from '../Keyboard.svelte';
  import { api, hex, unhex, errMsg, type Key } from '../api';

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
    try {
      await api.applyPerkey([...dirty.entries()].map(([m, rgb]) => [m, rgb]));
      dirty = new Map();
      notify('Cores por tecla aplicadas');
    } catch (e) {
      notify(errMsg(e));
    }
  }

  const fill = (m: number) => hex(colours[m] ?? [0, 0, 0]);
</script>

<section class="stack" aria-busy={loading}>
  <div class="row">
    <input type="color" bind:value={paint} aria-label="Cor do pincel" />
    <div class="seg">
      <label><input type="radio" bind:group={mode} value="paint" /> Pintar</label>
      <label><input type="radio" bind:group={mode} value="select" /> Selecionar</label>
    </div>
    <button onclick={fillAll}>Preencher tudo</button>
    <button class="primary push" onclick={apply} disabled={dirty.size === 0}>Aplicar ({dirty.size})</button>
  </div>
  {#if loading}<p class="hint">Lendo do teclado…</p>{/if}
  <Keyboard {keys} {fill} {selected} lit labelOf={(k) => k.name} onpick={pick} />
</section>

<style>
  .push {
    margin-left: auto;
  }
</style>
