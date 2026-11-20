<script lang="ts">
  import { onDestroy } from 'svelte';
  import { api, errMsg } from '../api';

  let { notify, busy = $bindable(false) }: { notify: (m: string) => void; busy?: boolean } = $props();

  let gain = $state(1.5);
  let running = $state(false);

  async function start() {
    try {
      await api.audioStart(gain);
      running = true;
      busy = true;
      notify('Áudio iniciado');
    } catch (e) {
      notify(errMsg(e));
    }
  }

  async function stop() {
    try {
      await api.audioStop();
      notify('Áudio parado');
    } catch (e) {
      notify(errMsg(e));
    } finally {
      running = false;
      busy = false;
    }
  }

  onDestroy(() => {
    if (running) api.audioStop().catch(() => {});
  });
</script>

<section class="stack narrow">
  <div class="card">
    <p class="hint">Reage ao som do sistema (loopback). No receptor 2.4G a taxa cai para cerca de 3 fps: prefira o cabo.</p>
    <label class="field">
      <span>Ganho</span>
      <input type="range" min="0.5" max="4" step="0.1" bind:value={gain} disabled={running} />
      <output class="num">{gain.toFixed(1)}</output>
    </label>
    <div class="row">
      {#if running}
        <span class="dot rec"></span>
        <span>Reagindo ao áudio</span>
        <button class="danger" onclick={stop}>Parar</button>
      {:else}
        <button class="primary" onclick={start}>Iniciar</button>
      {/if}
    </div>
    {#if running}
      <p class="hint">Enquanto o áudio roda, as outras gravações ficam desativadas.</p>
    {/if}
  </div>
</section>
