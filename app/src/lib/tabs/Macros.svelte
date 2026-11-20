<script lang="ts">
  import { onMount } from 'svelte';
  import { api, errMsg, type Macro, type MacroEvent } from '../api';
  import { fromEventCode, labelOf } from '../hid';

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

  function onkey(e: KeyboardEvent, down: boolean) {
    if (!recording || !active) return;
    e.preventDefault();
    const k = fromEventCode(e.code);
    if (!k || (e.repeat && down)) return;
    const now = performance.now();
    const delay = last ? Math.round(now - last) : 0;
    last = now;
    active.events.push({ usage: k.usage, down, delay_ms: delay });
    macros = [...macros];
  }

  function startRec() {
    last = 0;
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
    try {
      await api.saveMacros(macros);
      notify('Macros salvas no teclado');
    } catch (e) {
      notify(errMsg(e));
    }
  }
</script>

<section class="wrap">
  {#if loading}
    <p class="hint">Lendo do teclado…</p>
  {:else}
    <div class="split">
      <div class="list">
        {#each macros as m, i (i)}
          <button aria-pressed={i === cur} onclick={() => (cur = i)}>{m.name}</button>
        {/each}
        <button onclick={newMacro}>+ Nova</button>
      </div>

      {#if active}
        <div class="card">
          <div class="head">
            <h3>{active.name}</h3>
            {#if confirmDel}
              <span>Excluir “{active.name}”?</span>
              <button class="danger" onclick={remove}>Sim, excluir</button>
              <button onclick={() => (confirmDel = false)}>Cancelar</button>
            {:else}
              <button onclick={() => (confirmDel = true)}>Excluir</button>
            {/if}
          </div>

          <label class="field">
            <span>Nome</span>
            <input value={active.name} maxlength={NAME_MAX} oninput={(e) => rename(e.currentTarget.value)} />
            <output class="num">{active.name.length}/{NAME_MAX}</output>
          </label>

          <div class="row">
            <button class:primary={!recording} onclick={startRec} disabled={recording}>Gravar</button>
            <button onclick={() => (recording = false)} disabled={!recording}>Parar</button>
            {#if recording}
              <span class="dot rec"></span>
              <span>Gravando</span>
            {/if}
            <input
              class="catch"
              placeholder="Foque aqui e pressione teclas"
              onkeydown={(e) => onkey(e, true)}
              onkeyup={(e) => onkey(e, false)}
            />
          </div>

          {#if active.events.length}
            <table>
              <thead><tr><th>Tecla</th><th>Ação</th><th>Atraso (ms)</th><th></th></tr></thead>
              <tbody>
                {#each active.events as ev, i (i)}
                  <tr>
                    <td>{labelOf(ev.usage)}</td>
                    <td>
                      <button
                        aria-label={ev.down ? 'Pressionar' : 'Soltar'}
                        title={ev.down ? 'Pressionar' : 'Soltar'}
                        onclick={() => editEv(i, { down: !ev.down })}
                      >{ev.down ? '↓' : '↑'}</button>
                    </td>
                    <td>
                      <input class="num delay" type="number" min="0" value={ev.delay_ms} onchange={(e) => editEv(i, { delay_ms: +e.currentTarget.value })} />
                    </td>
                    <td>
                      <button class="ghost" aria-label="Mover para cima" onclick={() => move(i, -1)}>▲</button>
                      <button class="ghost" aria-label="Mover para baixo" onclick={() => move(i, 1)}>▼</button>
                      <button class="ghost" aria-label="Remover evento" onclick={() => delEv(i)}>✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {:else}
            <div class="empty">Clique em Gravar, foque o campo e pressione as teclas.</div>
          {/if}
        </div>
      {:else}
        <div class="empty">Nenhuma macro gravada. Crie uma com “Nova”.</div>
      {/if}
    </div>
  {/if}

  <div class="foot">
    <div class="meter" class:over={bytes > BYTES_MAX} style:--v={bytes / BYTES_MAX}></div>
    <span class="num" class:over={bytes > BYTES_MAX}>{bytes} / {BYTES_MAX} bytes</span>
    <button class="primary" onclick={save} disabled={loading}>Salvar no teclado</button>
  </div>
</section>

<style>
  .wrap { display: grid; gap: 14px; container-type: inline-size; }
  .split { display: grid; grid-template-columns: 200px minmax(0, 1fr); gap: 14px; align-items: start; }
  .list { display: grid; gap: 6px; align-content: start; }
  .list button { width: 100%; justify-content: flex-start; text-align: left; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .head h3 { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .catch { flex: 1; min-width: 160px; }
  .delay { width: 96px; }
  .foot { display: flex; gap: 12px; align-items: center; }
  .meter { flex: 1; }
  .over { color: var(--danger); }

  @container (max-width: 699px) {
    .split { grid-template-columns: minmax(0, 1fr); }
  }
</style>
