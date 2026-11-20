<script lang="ts">
  import { onMount } from 'svelte';
  import Keyboard from '../Keyboard.svelte';
  import { api, errMsg, type Key, type KeyAction, type Macro } from '../api';
  import { HID_KEYS, MEDIA_KEYS, MOD_BITS, labelOf as usageLabel } from '../hid';

  let { notify }: { notify: (m: string) => void } = $props();

  const FN_MATRIX = 53;
  let keys = $state<Key[]>([]);
  let macros = $state<Macro[]>([]);
  let layer = $state<0 | 1>(0);
  let actions = $state<KeyAction[]>([]);
  let sel = $state<number | null>(null);

  type Kind = 'tecla' | 'midia' | 'macro' | 'off' | 'raw';
  let kind = $state<Kind>('tecla');
  let usage = $state(0x04);
  let mods = $state(0);
  let media = $state(0xcd);
  let macroIdx = $state(0);
  let macroMode = $state<1 | 2 | 4>(1);
  let macroCount = $state(1);
  let rawHex = $state('00000000');

  const label = (m: number) => keys.find((k) => k.matrix === m)?.name ?? String(m);

  function describe(a: KeyAction | undefined): string {
    if (!a) return '—';
    switch (a.type) {
      case 'Key':
        return `${MOD_BITS.filter((b) => a.mods & b.bit).map((b) => b.label + '+').join('')}${usageLabel(a.usage)}`;
      case 'Consumer':
        return MEDIA_KEYS.find((m) => m.usage === a.usage)?.label ?? `Mídia 0x${a.usage.toString(16)}`;
      case 'Macro':
        return `Macro ${macros[a.index]?.name ?? a.index}`;
      case 'Fn':
        return 'Fn';
      case 'Disabled':
        return 'Desativada';
      case 'Raw':
        return 'Raw ' + a.bytes.map((b) => b.toString(16).padStart(2, '0')).join('');
    }
  }

  async function load() {
    try {
      actions = await api.readLayer(layer);
    } catch (e) {
      notify(errMsg(e));
    }
  }

  onMount(async () => {
    try {
      keys = await api.layout();
      macros = await api.listMacros();
    } catch (e) {
      notify(errMsg(e));
    }
    await load();
  });

  $effect(() => {
    layer;
    load();
  });

  function pick(m: number) {
    if (m === FN_MATRIX) return;
    sel = m;
    const a = actions[m];
    kind = !a ? 'tecla' : a.type === 'Key' ? 'tecla' : a.type === 'Consumer' ? 'midia' : a.type === 'Macro' ? 'macro' : a.type === 'Disabled' ? 'off' : 'raw';
    if (a?.type === 'Key') {
      mods = a.mods;
      usage = a.usage;
    }
    if (a?.type === 'Consumer') media = a.usage;
    if (a?.type === 'Macro') {
      macroIdx = a.index;
      macroMode = a.mode as 1 | 2 | 4;
      macroCount = a.count;
    }
    if (a?.type === 'Raw') rawHex = a.bytes.map((b) => b.toString(16).padStart(2, '0')).join('');
  }

  function build(): KeyAction | string {
    switch (kind) {
      case 'tecla':
        return { type: 'Key', mods, usage };
      case 'midia':
        return { type: 'Consumer', usage: media };
      case 'macro':
        return { type: 'Macro', mode: macroMode, count: macroMode === 1 ? macroCount : 1, index: macroIdx };
      case 'off':
        return { type: 'Disabled' };
      case 'raw': {
        const h = rawHex.replace(/\s/g, '');
        if (!/^[0-9a-fA-F]{8}$/.test(h)) return 'Raw precisa de 8 caracteres hex';
        return { type: 'Raw', bytes: Array.from({ length: 4 }, (_, i) => parseInt(h.slice(i * 2, i * 2 + 2), 16)) };
      }
    }
  }

  async function save() {
    if (sel === null) return;
    const a = build();
    if (typeof a === 'string') return notify(a);
    try {
      await api.setKey(layer, sel, a);
      await load();
      notify('Tecla gravada');
    } catch (e) {
      notify(errMsg(e));
    }
  }

  const toggleMod = (bit: number) => (mods ^= bit);
</script>

<section class="wrap">
  <div class="layers">
    <button class:on={layer === 0} onclick={() => (layer = 0)}>Base</button>
    <button class:on={layer === 1} onclick={() => (layer = 1)}>Fn</button>
  </div>
  <div class="split">
    <Keyboard
      {keys}
      fill={(m) => (m === sel ? '#4f7cff' : '#d9dee8')}
      labelOf={(k) => `${k.name}\n${describe(actions[k.matrix])}`}
      selected={new Set(sel === null ? [] : [sel])}
      onpick={(m) => pick(m)}
    />
    <aside class="panel">
      {#if sel === null}
        <p>Clique em uma tecla.</p>
      {:else}
        <h3>{label(sel)}</h3>
        <p>Atual: {describe(actions[sel])}</p>
        <label>Tipo
          <select bind:value={kind}>
            <option value="tecla">Tecla</option>
            <option value="midia">Mídia</option>
            <option value="macro">Macro</option>
            <option value="off">Desativada</option>
            <option value="raw">Raw</option>
          </select>
        </label>
        {#if kind === 'tecla'}
          <select bind:value={usage}>
            {#each HID_KEYS as k (k.usage)}
              <option value={k.usage}>{k.label}</option>
            {/each}
          </select>
          <div class="mods">
            {#each MOD_BITS as b (b.bit)}
              <label><input type="checkbox" checked={!!(mods & b.bit)} onchange={() => toggleMod(b.bit)} /> {b.label}</label>
            {/each}
          </div>
        {:else if kind === 'midia'}
          <select bind:value={media}>
            {#each MEDIA_KEYS as m (m.usage)}
              <option value={m.usage}>{m.label}</option>
            {/each}
          </select>
        {:else if kind === 'macro'}
          <select bind:value={macroIdx}>
            {#each macros as m, i (i)}
              <option value={i}>{m.name}</option>
            {/each}
          </select>
          <select bind:value={macroMode}>
            <option value={1}>N vezes</option>
            <option value={2}>Até soltar</option>
            <option value={4}>Até outra tecla</option>
          </select>
          {#if macroMode === 1}
            <input type="number" min="1" max="255" bind:value={macroCount} />
          {/if}
        {:else if kind === 'raw'}
          <input bind:value={rawHex} maxlength="8" placeholder="XXXXXXXX" />
        {/if}
        <button class="primary" onclick={save}>Gravar tecla</button>
      {/if}
    </aside>
  </div>
</section>

<style>
  .wrap { display: grid; gap: 12px; }
  .layers { display: flex; gap: 6px; }
  .layers button.on { background: var(--accent); color: #fff; }
  .split { display: grid; grid-template-columns: minmax(0, 1fr) 280px; gap: 16px; align-items: start; }
  .panel { display: grid; gap: 10px; }
  .mods { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  @media (max-width: 800px) { .split { grid-template-columns: 1fr; } }
</style>
