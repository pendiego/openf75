<script lang="ts">
  import { onMount } from 'svelte';
  import Keyboard from '../Keyboard.svelte';
  import Kbd from '../Kbd.svelte';
  import { api, errMsg, type Key, type KeyAction, type Macro } from '../api';
  import { HID_KEYS, MEDIA_KEYS, MOD_BITS, comboLabels } from '../hid';
  import AsyncButton from '../AsyncButton.svelte';

  let { notify }: { notify: (m: string) => void } = $props();

  const FN_MATRIX = 53;
  let keys = $state<Key[]>([]);
  let macros = $state<Macro[]>([]);
  let layer = $state<0 | 1>(0);
  let actions = $state<KeyAction[]>([]);
  let sel = $state<number | null>(null);
  let loading = $state(true);

  type Kind = 'tecla' | 'midia' | 'macro' | 'off' | 'raw';
  let kind = $state<Kind>('tecla');
  let usage = $state(0x04);
  let mods = $state(0);
  let media = $state(0xcd);
  let macroIdx = $state(0);
  let macroMode = $state<1 | 2 | 4>(1);
  let macroCount = $state(1);
  let rawHex = $state('00000000');
  let query = $state('');

  const label = (m: number) => keys.find((k) => k.matrix === m)?.name ?? String(m);

  function describe(a: KeyAction | undefined): string {
    if (!a) return '—';
    switch (a.type) {
      case 'Key':
        return comboLabels(a.mods, a.usage).join('+');
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

  // A second line marks the key as remapped: only when it does something other than its factory action.
  const keyLabel = (k: Key) => {
    const a = actions[k.matrix];
    const stock = !a || a.type === 'Fn' || (a.type === 'Key' && a.mods === 0 && a.usage === k.default_usage);
    return stock ? k.name : `${k.name}\n${describe(a)}`;
  };

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
    try {
      await load();
    } finally {
      loading = false;
    }
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
    if (typeof a === 'string') throw new Error(a);
    await api.setKey(layer, sel, a);
    await load();
    notify('Tecla gravada');
  }

  const toggleMod = (bit: number) => (mods ^= bit);

  const q = $derived(query.trim().toLowerCase());
  const hit = (s: string) => s.toLowerCase().includes(q);
  const teclas = $derived(HID_KEYS.filter((k) => hit(k.label)));
  const midias = $derived(MEDIA_KEYS.filter((m) => hit(m.label)));
  const desativada = $derived(hit('Desativada'));
  const macroRows = $derived(macros.map((m, i) => ({ i, name: m.name })).filter((r) => hit(r.name)));
  const vazio = $derived(q !== '' && !teclas.length && !midias.length && !desativada && !macroRows.length);
</script>

<div class="workspace" aria-busy={loading}>
  <div class="stage">
    <div class="strip">
      <span class="label">Camada</span>
      <div class="seg">
        <button aria-pressed={layer === 0} onclick={() => (layer = 0)}>Base</button>
        <button aria-pressed={layer === 1} onclick={() => (layer = 1)}>Fn</button>
      </div>
      <p class="hint">{loading ? 'Lendo do teclado…' : 'Clique em uma tecla para remapear'}</p>
    </div>
    <div class="stage-body hero">
      <Keyboard
        {keys}
        labelOf={keyLabel}
        selected={new Set(sel === null ? [] : [sel])}
        onpick={(m) => pick(m)}
      />
    </div>
  </div>

  <aside class="inspector">
    {#if sel === null}
      <div class="empty"><p class="hint">Clique em uma tecla para remapear.</p></div>
    {:else}
      {@const cur = actions[sel]}
      <div class="head">
        <h2>{label(sel)}</h2>
        <p class="hint">Camada {layer === 0 ? 'Base' : 'Fn'}</p>
      </div>
      <div class="card">
        <div class="label">Ação atual</div>
        {#if cur?.type === 'Key'}
          <Kbd keys={comboLabels(cur.mods, cur.usage)} />
        {:else}
          <p>{describe(cur)}</p>
        {/if}
      </div>
      <input type="search" placeholder="Buscar ação…" aria-label="Buscar ação" bind:value={query} />
      <div class="picker">
        {#if teclas.length}
          <div class="group">
            <div class="label">Teclado</div>
            <div class="list">
              {#each teclas as k (k.usage)}
                <button
                  aria-pressed={kind === 'tecla' && usage === k.usage}
                  onclick={() => {
                    kind = 'tecla';
                    usage = k.usage;
                  }}>{k.label}</button
                >
              {/each}
            </div>
          </div>
        {/if}
        {#if midias.length}
          <div class="group">
            <div class="label">Mídia</div>
            <div class="list">
              {#each midias as m (m.usage)}
                <button
                  aria-pressed={kind === 'midia' && media === m.usage}
                  onclick={() => {
                    kind = 'midia';
                    media = m.usage;
                  }}>{m.label}</button
                >
              {/each}
            </div>
          </div>
        {/if}
        {#if macroRows.length || (!macros.length && !q)}
          <div class="group">
            <div class="label">Macros</div>
            {#if macros.length}
              <div class="list">
                {#each macroRows as r (r.i)}
                  <button
                    aria-pressed={kind === 'macro' && macroIdx === r.i}
                    onclick={() => {
                      kind = 'macro';
                      macroIdx = r.i;
                    }}>{r.name}</button
                  >
                {/each}
              </div>
            {:else}
              <p class="hint">Nenhuma macro salva. Crie uma na aba Macros.</p>
            {/if}
          </div>
        {/if}
        {#if desativada}
          <div class="group">
            <div class="label">Outros</div>
            <div class="list">
              <button aria-pressed={kind === 'off'} onclick={() => (kind = 'off')}>Desativada</button>
            </div>
          </div>
        {/if}
        {#if vazio}
          <p class="hint">Nenhuma ação encontrada.</p>
        {/if}
      </div>

      <div class="opts">
        {#if kind === 'tecla'}
          <div class="label">Modificadores</div>
          <div class="row">
            {#each MOD_BITS as b (b.bit)}
              <button aria-pressed={!!(mods & b.bit)} onclick={() => toggleMod(b.bit)}>{b.label}</button>
            {/each}
          </div>
        {:else if kind === 'macro'}
          <div class="label">Modo</div>
          <div class="seg">
            <button aria-pressed={macroMode === 1} onclick={() => (macroMode = 1)}>N vezes</button>
            <button aria-pressed={macroMode === 2} onclick={() => (macroMode = 2)}>Até soltar</button>
            <button aria-pressed={macroMode === 4} onclick={() => (macroMode = 4)}>Até outra tecla</button>
          </div>
          {#if macroMode === 1}
            <label class="field">
              <span>Vezes</span>
              <input class="num" type="number" min="1" max="255" bind:value={macroCount} />
            </label>
          {/if}
        {/if}
        <details open={kind === 'raw'}>
          <summary class="label">Avançado: Raw</summary>
          <label class="field">
            <span>Hex</span>
            <input
              class="num"
              bind:value={rawHex}
              maxlength="8"
              placeholder="XXXXXXXX"
              oninput={() => (kind = 'raw')}
            />
          </label>
        </details>
      </div>

      <div class="save">
        <AsyncButton class="primary" run={save} notify={notify} label="Gravar tecla" busyLabel="Gravando…" />
      </div>
    {/if}
  </aside>
</div>

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .strip .hint {
    margin-left: auto;
  }
  .head {
    display: grid;
    gap: 2px;
  }
  .picker {
    flex: 1 1 0;
    min-height: 120px;
    overflow-y: auto;
    display: grid;
    gap: 12px;
    align-content: start;
  }
  .group {
    display: grid;
    gap: 4px;
  }
  .opts {
    display: grid;
    gap: 8px;
  }
  summary {
    cursor: pointer;
  }
  .save :global(button) {
    width: 100%;
  }
</style>
