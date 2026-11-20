<script lang="ts">
  import { onMount } from 'svelte';
  import { api, errMsg } from '../api';

  let { notify }: { notify: (m: string) => void } = $props();

  // UI seconds → label; "" = never. Values match SLEEP_TABLE in the lib.
  const SLEEP: [string, number | null][] = [
    ['Nunca', null],
    ['1 min', 60],
    ['1,5 min', 90],
    ['2 min', 120],
    ['3 min', 180],
    ['4 min', 240],
    ['5 min', 300],
    ['10 min', 600],
    ['15 min', 900],
    ['20 min', 1200],
  ];

  let sleep = $state<number | null>(null);
  let debounce = $state(1);
  let confirmReset = $state(false);

  onMount(async () => {
    try {
      const [secs, deb] = await api.getPower();
      sleep = secs;
      debounce = deb;
    } catch (e) {
      notify(errMsg(e));
    }
  });

  async function setSleep(v: number | null) {
    sleep = v;
    try {
      await api.setSleep(v);
      notify('Suspensão gravada');
    } catch (e) {
      notify(errMsg(e));
    }
  }

  async function setDebounce(v: number) {
    debounce = v;
    try {
      await api.setDebounce(v);
      notify('Debounce gravado');
    } catch (e) {
      notify(errMsg(e));
    }
  }

  async function reset() {
    try {
      await api.factoryReset();
      confirmReset = false;
      notify('Teclado restaurado de fábrica');
    } catch (e) {
      notify(errMsg(e));
    }
  }
</script>

<section class="wrap">
  <div class="card">
    <h3>Suspensão automática</h3>
    <p class="hint">Vale só no modo sem fio (receptor 2.4G). No cabo o teclado não dorme.</p>
    <select value={sleep ?? ''} onchange={(e) => setSleep(e.currentTarget.value === '' ? null : +e.currentTarget.value)}>
      {#each SLEEP as [label, secs] (label)}
        <option value={secs ?? ''}>{label}</option>
      {/each}
    </select>
  </div>

  <div class="card">
    <h3>Debounce</h3>
    <p class="hint">Atraso antirrebote das teclas. Valores de 0 a 4.</p>
    <input type="range" min="0" max="4" value={debounce} oninput={(e) => setDebounce(+e.currentTarget.value)} />
    <span>{debounce}</span>
  </div>

  <div class="card danger">
    <h3>Restaurar de fábrica</h3>
    <p class="hint">Apaga as configurações gravadas no teclado. Faça um backup antes.</p>
    {#if confirmReset}
      <p>Tem certeza?</p>
      <button class="danger-btn" onclick={reset}>Sim, restaurar</button>
      <button onclick={() => (confirmReset = false)}>Cancelar</button>
    {:else}
      <button class="danger-btn" onclick={() => (confirmReset = true)}>Restaurar de fábrica</button>
    {/if}
  </div>
</section>

<style>
  .wrap { display: grid; gap: 14px; max-width: 560px; }
  .card { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 14px; display: grid; gap: 8px; }
  .card h3 { margin: 0; font-size: 15px; }
  .hint { margin: 0; color: var(--muted); font-size: 13px; }
  .danger { border-color: var(--danger); }
  .danger-btn { background: var(--danger); color: #fff; }
</style>
