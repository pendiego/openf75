<script lang="ts">
  import { onMount } from 'svelte';
  import { api, errMsg } from '../api';
  import AsyncButton from '../AsyncButton.svelte';
  import Num from '../Num.svelte';

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
  let loading = $state(true);

  onMount(async () => {
    try {
      const [secs, deb] = await api.getPower();
      sleep = secs;
      debounce = deb;
    } catch (e) {
      notify(errMsg(e));
    } finally {
      loading = false;
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
    await api.factoryReset();
    confirmReset = false;
    notify('Teclado restaurado de fábrica');
  }
</script>

<div class="workspace">
  <div class="stage">
    <div class="stage-body">
      <div class="stack narrow">
        <div class="setting">
          <h3>Suspensão automática</h3>
          <select
            aria-label="Suspensão automática"
            disabled={loading}
            value={sleep ?? ''}
            onchange={(e) => setSleep(e.currentTarget.value === '' ? null : +e.currentTarget.value)}
          >
            {#each SLEEP as [label, secs] (label)}
              <option value={secs ?? ''}>{label}</option>
            {/each}
          </select>
          <p class="hint">Vale só no modo sem fio (receptor 2.4G). No cabo o teclado não dorme.</p>
        </div>

        <div class="setting">
          <h3>Debounce</h3>
          <label class="field">
            <span>Atraso</span>
            <input type="range" min="0" max="4" value={debounce} disabled={loading} oninput={(e) => setDebounce(+e.currentTarget.value)} />
            <Num value={debounce} />
          </label>
          <p class="hint">Atraso antirrebote das teclas. Valores de 0 a 4.</p>
        </div>

        <div class="card danger">
          <h3>Restaurar de fábrica</h3>
          <p class="hint">Apaga as configurações gravadas no teclado. Faça um backup antes.</p>
          {#if confirmReset}
            <p>Tem certeza?</p>
            <div class="row">
              <AsyncButton class="danger" run={reset} notify={notify} label="Sim, restaurar" busyLabel="Restaurando…" />
              <button onclick={() => (confirmReset = false)}>Cancelar</button>
            </div>
          {:else}
            <div class="row">
              <button class="danger" onclick={() => (confirmReset = true)}>Restaurar de fábrica</button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
