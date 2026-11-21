<script lang="ts">
  import { onDestroy } from 'svelte';
  import { api, errMsg } from '../api';
  import Num from '../Num.svelte';

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

<div class="workspace">
  <div class="stage">
    <div class="stage-body">
      <div class="stack narrow">
        <div class="setting">
          <h3>Ganho</h3>
          <div class="row">
            <input type="range" min="0.5" max="4" step="0.1" bind:value={gain} disabled={running} aria-label="Ganho" />
            <Num value={gain} digits={1} />
          </div>
          <p class="hint">Multiplica o áudio captado antes da reação (0.5 a 4).</p>
        </div>
        <div class="setting">
          <h3>Entrada</h3>
          <span class="hint">Loopback do sistema</span>
          <p class="hint">Reage ao som do sistema (loopback). No receptor 2.4G a taxa cai para cerca de 3 fps: prefira o cabo.</p>
        </div>
        <div class="stack">
          {#if running}
            <button class="danger wide" onclick={stop}>Parar</button>
            <p class="hint"><span class="dot rec"></span> Reagindo ao áudio. Enquanto roda, as outras gravações ficam desativadas.</p>
          {:else}
            <button class="primary wide" onclick={start}>Iniciar</button>
            <p class="hint"><span class="dot"></span> Parado</p>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .wide {
    width: 100%;
  }
</style>
