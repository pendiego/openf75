<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api, errMsg, type Status } from '$lib/api';
  import Iluminacao from '$lib/tabs/Iluminacao.svelte';
  import PorTecla from '$lib/tabs/PorTecla.svelte';
  import Teclas from '$lib/tabs/Teclas.svelte';
  import Macros from '$lib/tabs/Macros.svelte';
  import Energia from '$lib/tabs/Energia.svelte';
  import Audio from '$lib/tabs/Audio.svelte';
  import Perfis from '$lib/tabs/Perfis.svelte';

  const TABS = [
    { id: 'luz', label: 'Iluminação' },
    { id: 'tecla_rgb', label: 'Por tecla' },
    { id: 'teclas', label: 'Teclas' },
    { id: 'macros', label: 'Macros' },
    { id: 'energia', label: 'Energia' },
    { id: 'audio', label: 'Áudio' },
    { id: 'perfis', label: 'Perfis' },
  ] as const;
  type TabId = (typeof TABS)[number]['id'];

  let tab = $state<TabId>('luz');
  let status = $state<Status | null>(null);
  let statusErr = $state('');
  let audioBusy = $state(false);
  let toast = $state('');
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const notify = (m: string) => {
    toast = m;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = ''), 4000);
  };

  const badge = $derived(
    !status || !status.connected
      ? 'Desconectado'
      : status.kind === 'cable'
        ? 'Cabo'
        : status.online
          ? 'Receptor 2.4G'
          : 'Receptor 2.4G (dormindo)',
  );

  async function poll() {
    try {
      status = await api.status();
      statusErr = '';
    } catch (e) {
      status = null;
      statusErr = errMsg(e);
    }
  }

  let timer: ReturnType<typeof setInterval> | undefined;
  onMount(() => {
    poll();
    timer = setInterval(poll, 2000);
  });
  onDestroy(() => clearInterval(timer));
</script>

<header>
  <strong>OpenF75</strong>
  <span class="badge" class:ok={status?.connected}>{badge}</span>
  {#if status?.fw != null}<span class="muted">fw {status.fw.toString(16)}</span>{/if}
  {#if status?.oem_running}<span class="warn">OemDrv.exe em execução</span>{/if}
</header>

{#if status?.writable_error}
  <div class="banner">{status.writable_error}</div>
{/if}
{#if statusErr}
  <div class="banner">{statusErr}</div>
{/if}

<nav>
  {#each TABS as t (t.id)}
    <button class:on={tab === t.id} disabled={audioBusy && t.id !== 'audio'} onclick={() => (tab = t.id)}>{t.label}</button>
  {/each}
</nav>

<main>
  {#if tab === 'luz'}<Iluminacao {notify} />{/if}
  {#if tab === 'tecla_rgb'}<PorTecla {notify} />{/if}
  {#if tab === 'teclas'}<Teclas {notify} />{/if}
  {#if tab === 'macros'}<Macros {notify} />{/if}
  {#if tab === 'energia'}<Energia {notify} />{/if}
  {#if tab === 'audio'}<Audio {notify} bind:busy={audioBusy} />{/if}
  {#if tab === 'perfis'}<Perfis {notify} />{/if}
</main>

{#if toast}<div class="toast">{toast}</div>{/if}

<style>
  header { display: flex; gap: 12px; align-items: center; padding: 12px 18px; border-bottom: 1px solid var(--line); }
  .badge { padding: 2px 8px; border-radius: 10px; background: #3a3f4b; font-size: 13px; }
  .badge.ok { background: #2d6a4f; }
  .muted { color: var(--muted); font-size: 13px; }
  .warn { color: #f0ad4e; font-size: 13px; }
  .banner { margin: 10px 18px 0; padding: 8px 12px; border-radius: 6px; background: #4a2626; color: #ffb4b4; }
  nav { display: flex; gap: 6px; flex-wrap: wrap; padding: 12px 18px; }
  nav button.on { background: var(--accent); color: #fff; border-color: var(--accent); }
  main { padding: 0 18px 24px; }
  .toast { position: fixed; bottom: 16px; right: 16px; background: var(--panel); border: 1px solid var(--line); padding: 10px 14px; border-radius: 8px; }
</style>
