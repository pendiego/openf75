<script lang="ts">
  import { onMount } from 'svelte';
  import { api, errMsg, type Macro, type MacroEvent } from '../api';
  import { fromEventCode, labelOf } from '../hid';
  import Kbd from '../Kbd.svelte';
  import AsyncButton from '../AsyncButton.svelte';

  let { notify }: { notify: (m: string) => void } = $props();

  const NAME_MAX = 30;
  const BYTES_MAX = 4096;

  let macros = $state<Macro[]>([]);
  let cur = $state(-1);
  let recording = $state(false);
  let loading = $state(true);
  let confirmDel = $state(false);
  let last = 0;

  const active = $derived(cur >= 0 ? macros[cur] : null);

  // Size as the firmware counts it: header 4 bytes per macro, name, 4 bytes per event.
  const bytes = $derived(
    macros.reduce((n, m) => n + 4 + 1 + new TextEncoder().encode(m.name).length * 2 + m.events.length * 4, 0),
  );

  $effect(() => {
    void cur;
    confirmDel = false;
  });

  // A take belongs to one macro: switching, adding or deleting macros ends it (events kept).
  // Derived count: `macros = [...macros]` on every recorded key must not end the take.
  const macroCount = $derived(macros.length);
  $effect(() => {
    void cur;
    void macroCount;
    recording = false;
    take = null;
  });

  onMount(async () => {
    try {
      macros = await api.listMacros();
      if (macros.length) cur = 0;
    } catch (e) {
      notify(errMsg(e));
    } finally {
      loading = false;
    }
  });

  function newMacro() {
    macros = [...macros, { name: `Macro ${macros.length + 1}`, events: [] }];
    cur = macros.length - 1;
  }

  function remove() {
    if (cur < 0) return;
    macros = macros.filter((_, i) => i !== cur);
    cur = Math.min(cur, macros.length - 1);
    confirmDel = false;
  }

  function rename(v: string) {
    if (!active) return;
    active.name = v.slice(0, NAME_MAX);
  }

  // Esc restores this snapshot; bound to the macro the take started on.
  let take: { macro: Macro; before: MacroEvent[] } | null = null;

  function onkey(e: KeyboardEvent, down: boolean) {
    if (!recording || !active) return;
    e.preventDefault();
    // Esc cancels: restores the events from before this take (no unmatched key-down left behind).
    if (down && e.code === 'Escape') {
      recording = false;
      if (take && take.macro === active) active.events = take.before;
      take = null;
      macros = [...macros];
      return;
    }
    const k = fromEventCode(e.code);
    if (!k || (e.repeat && down)) return;
    const now = performance.now();
    const delay = last ? Math.round(now - last) : 0;
    last = now;
    active.events.push({ usage: k.usage, down, delay_ms: delay });
    macros = [...macros];
  }

  function startRec() {
    if (!active) return;
    last = 0;
    take = { macro: active, before: $state.snapshot(active.events) };
    recording = true;
  }

  function move(i: number, d: number) {
    if (!active) return;
    const j = i + d;
    if (j < 0 || j >= active.events.length) return;
    [active.events[i], active.events[j]] = [active.events[j], active.events[i]];
    macros = [...macros];
  }

  function editEv(i: number, patch: Partial<MacroEvent>) {
    if (!active) return;
    Object.assign(active.events[i], patch);
    macros = [...macros];
  }

  function delEv(i: number) {
    if (!active) return;
    active.events.splice(i, 1);
    macros = [...macros];
  }

  async function save() {
    await api.saveMacros(macros);
    notify('Macros salvas no teclado');
  }
</script>

<div class="workspace" aria-busy={loading}>
  <div class="stage">
    <div class="strip">
      <button onclick={newMacro} disabled={loading}>Nova macro</button>
      <div class="storage">
        <div class="meter" class:over={bytes > BYTES_MAX} style:--v={bytes / BYTES_MAX}></div>
        <span class="num" class:over={bytes > BYTES_MAX}>{bytes} / {BYTES_MAX} bytes</span>
      </div>
      <AsyncButton class="primary" run={save} notify={notify} label="Salvar no teclado" busyLabel="Salvando…" disabled={loading} />
    </div>

    <div class="stage-body">
      {#if loading}
        <p class="hint">Lendo do teclado…</p>
      {:else if macros.length}
        <div class="narrow list">
          {#each macros as m, i (i)}
            <button aria-pressed={i === cur} onclick={() => (cur = i)}>
              <span class="name">{m.name}</span>
              <span class="hint num">{m.events.length} {m.events.length === 1 ? 'evento' : 'eventos'}</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="narrow empty">Nenhuma macro gravada. Crie uma com “Nova macro”.</div>
      {/if}
    </div>
  </div>

  {#if !loading}
    <aside class="inspector">
      {#if active}
        <div class="head">
          <h2>{active.name}</h2>
          {#if confirmDel}
            <span class="hint">Excluir “{active.name}”?</span>
            <button class="danger" onclick={remove}>Sim, excluir</button>
            <button onclick={() => (confirmDel = false)}>Cancelar</button>
          {:else}
            <button class="ghost" onclick={() => (confirmDel = true)}>Excluir macro</button>
          {/if}
        </div>

        <label class="field">
          <span>Nome</span>
          <input value={active.name} maxlength={NAME_MAX} oninput={(e) => rename(e.currentTarget.value)} />
          <output class="num">{active.name.length}/{NAME_MAX}</output>
        </label>

        <div class="record">
          <div class="row">
            <button class:primary={!recording} onclick={startRec} disabled={recording}>Gravar</button>
            <button onclick={() => (recording = false)} disabled={!recording}>Parar</button>
            <span class="rec-state" role="status" aria-live="polite">
              {#if recording}<span class="dot rec"></span> Gravando — Esc cancela{/if}
            </span>
          </div>
          <input
            placeholder="Foque aqui e pressione teclas"
            onkeydown={(e) => onkey(e, true)}
            onkeyup={(e) => onkey(e, false)}
          />
        </div>

        {#if active.events.length}
          <div class="events">
            <table>
              <thead><tr><th>Tecla</th><th>Ação</th><th>Atraso</th><th></th></tr></thead>
              <tbody>
                {#each active.events as ev, i (i)}
                  <tr>
                    <td><Kbd keys={[labelOf(ev.usage)]} /></td>
                    <td>
                      <button
                        aria-label={ev.down ? 'Pressionar' : 'Soltar'}
                        title={ev.down ? 'Pressionar' : 'Soltar'}
                        onclick={() => editEv(i, { down: !ev.down })}
                      >{ev.down ? '↓' : '↑'}</button>
                    </td>
                    <td>
                      <input class="num delay" type="number" min="0" aria-label="Atraso (ms)" value={ev.delay_ms} onchange={(e) => editEv(i, { delay_ms: +e.currentTarget.value })} />
                    </td>
                    <td class="acts">
                      <button class="ghost" aria-label="Mover para cima" onclick={() => move(i, -1)}>▲</button>
                      <button class="ghost" aria-label="Mover para baixo" onclick={() => move(i, 1)}>▼</button>
                      <button class="ghost" aria-label="Remover evento" onclick={() => delEv(i)}>✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="empty">Clique em Gravar, foque o campo e pressione as teclas.</div>
        {/if}
      {:else}
        <div class="empty">Selecione uma macro para editar.</div>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .storage { display: flex; align-items: center; gap: 8px; margin-left: auto; min-width: 0; }
  .meter { width: 120px; }
  .over { color: var(--danger); }
  .list { display: grid; gap: 6px; align-content: start; }
  .list button { display: grid; gap: 2px; width: 100%; text-align: left; }
  .list .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .list .hint { margin: 0; }
  .head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .head h2 { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .record { display: grid; gap: 8px; }
  .record input { width: 100%; min-width: 0; }
  .rec-state { display: inline-flex; align-items: center; gap: 6px; }
  .events { overflow-x: auto; }
  table { table-layout: fixed; width: 100%; }
  th, td { padding: 6px 2px; }
  td:first-child { width: 40px; }
  td:nth-child(2) { width: 36px; }
  .delay { width: 50px; padding: 4px 2px; }
  .acts { white-space: nowrap; }
  .acts button { padding: 4px 2px; min-width: 0; }
</style>
