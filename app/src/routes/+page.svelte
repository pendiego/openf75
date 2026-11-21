<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api, errMsg, type Status } from '$lib/api';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import Iluminacao from '$lib/tabs/Iluminacao.svelte';
  import PorTecla from '$lib/tabs/PorTecla.svelte';
  import Teclas from '$lib/tabs/Teclas.svelte';
  import Macros from '$lib/tabs/Macros.svelte';
  import Energia from '$lib/tabs/Energia.svelte';
  import Audio from '$lib/tabs/Audio.svelte';
  import Perfis from '$lib/tabs/Perfis.svelte';

  const GROUPS = [
    {
      label: 'Luz',
      tabs: [
        { id: 'luz', label: 'Iluminação' },
        { id: 'tecla_rgb', label: 'Por tecla' },
        { id: 'audio', label: 'Áudio' },
      ],
    },
    {
      label: 'Teclas',
      tabs: [
        { id: 'teclas', label: 'Teclas' },
        { id: 'macros', label: 'Macros' },
      ],
    },
    {
      label: 'Teclado',
      tabs: [
        { id: 'energia', label: 'Energia' },
        { id: 'perfis', label: 'Perfis' },
      ],
    },
  ] as const;
  type TabId = (typeof GROUPS)[number]['tabs'][number]['id'];
  const TABS = GROUPS.flatMap<{ id: TabId; label: string }>((g) => g.tabs);

  let tab = $state<TabId>('luz');
  let status = $state<Status | null>(null);
  let statusErr = $state('');
  let audioBusy = $state(false);
  let toast = $state('');
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const tabLabel = $derived(TABS.find((t) => t.id === tab)?.label ?? '');

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

  // Startup update check. Failures are silent: offline, dev mode and repos without releases are normal.
  let update = $state<Update | null>(null);
  let updateStep = $state<'idle' | 'download' | 'install'>('idle');
  let updatePct = $state<number | null>(null); // 0..1, null while the download size is unknown

  async function installUpdate() {
    const u = update;
    if (!u || updateStep !== 'idle') return;
    updateStep = 'download';
    let got = 0;
    let total = 0;
    try {
      await u.downloadAndInstall((ev) => {
        if (ev.event === 'Started') total = ev.data.contentLength ?? 0;
        else if (ev.event === 'Progress') {
          got += ev.data.chunkLength;
          updatePct = total ? Math.min(got / total, 1) : null;
        } else updateStep = 'install';
      });
    } catch (e) {
      updateStep = 'idle';
      updatePct = null;
      notify(`Falha ao atualizar: ${errMsg(e)}`);
    }
  }

  function dismissUpdate() {
    if (updateStep !== 'idle') return;
    const u = update;
    update = null;
    u?.close().catch(() => {});
  }

  let timer: ReturnType<typeof setInterval> | undefined;
  onMount(() => {
    poll();
    check()
      .then((u) => (update = u))
      .catch((e) => console.warn('update check failed:', e));
    timer = setInterval(poll, 2000);
  });
  onDestroy(() => clearInterval(timer));
</script>

<div class="shell">
  <aside>
    <div class="wordmark">OpenF75</div>
    <nav aria-label="Seções">
      {#each GROUPS as g (g.label)}
        <div class="group">
          <span class="group-label">{g.label}</span>
          {#each g.tabs as t (t.id)}
            <button
              class="item"
              aria-current={tab === t.id ? 'page' : undefined}
              disabled={audioBusy && t.id !== 'audio'}
              onclick={() => (tab = t.id)}>{t.label}</button>
          {/each}
        </div>
      {/each}
    </nav>
    <div class="device">
      <div class="device-line">
        <span
          class="dot"
          class:ok={status?.connected && status?.online !== false}
          class:warn={status?.connected && status?.online === false}></span>
        <span>{badge}</span>
      </div>
      {#if status?.fw != null}<span class="num fw">fw {status.fw.toString(16)}</span>{/if}
      {#if status?.oem_running}<span class="oem">OemDrv.exe aberto: feche antes de gravar</span>{/if}
    </div>
  </aside>

  <main>
    <h1>{tabLabel}</h1>
    {#if status?.writable_error}
      <div class="banner" role="alert">{status.writable_error}</div>
    {/if}
    {#if statusErr && statusErr !== status?.writable_error}
      <div class="banner" role="alert">{statusErr}</div>
    {/if}
    {#if update}
      <section class="update row" aria-label="Atualização disponível">
        <span class="update-text">Versão {update.version} disponível</span>
        <button class="primary" aria-disabled={updateStep !== 'idle'} onclick={installUpdate}>
          {updateStep === 'idle' ? 'Atualizar' : updateStep === 'download' ? 'Baixando…' : 'Instalando…'}
        </button>
        <button class="ghost" aria-disabled={updateStep !== 'idle'} onclick={dismissUpdate}>Depois</button>
        {#if updatePct != null}
          <div
            class="meter"
            style:--v={updatePct}
            role="progressbar"
            aria-label="Progresso do download"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow={Math.round(updatePct * 100)}></div>
        {/if}
      </section>
    {/if}
    {#if tab === 'luz'}<Iluminacao {notify} />{/if}
    {#if tab === 'tecla_rgb'}<PorTecla {notify} />{/if}
    {#if tab === 'teclas'}<Teclas {notify} />{/if}
    {#if tab === 'macros'}<Macros {notify} />{/if}
    {#if tab === 'energia'}<Energia {notify} />{/if}
    {#if tab === 'audio'}<Audio {notify} bind:busy={audioBusy} />{/if}
    {#if tab === 'perfis'}<Perfis {notify} />{/if}
  </main>
</div>

{#if toast}<div class="toast" role="status" aria-live="polite">{toast}</div>{/if}

<style>
  .shell { display: grid; grid-template-columns: 220px minmax(0, 1fr); height: 100dvh; overflow: hidden; }
  aside { display: flex; flex-direction: column; gap: 20px; padding: 20px 12px; background: var(--rail); border-right: 1px solid var(--line); min-height: 0; }
  .wordmark { padding: 0 8px; font: 600 18px/1.2 var(--display); letter-spacing: -0.01em; }
  nav { display: grid; gap: 16px; align-content: start; flex: 1; overflow: auto; }
  .group { display: grid; gap: 2px; }
  .group-label { padding: 0 8px 4px; font-size: 12px; color: var(--muted); }
  .item { display: block; width: 100%; padding: 6px 8px; border: 0; border-radius: var(--r-s); background: none; box-shadow: none; color: var(--muted); text-align: left; font: inherit; }
  .item:hover:not(:disabled) { background: none; color: var(--text); }
  .item[aria-current="page"] { background: var(--raised); color: var(--text); }
  .device { display: grid; gap: 6px; padding: 12px 8px 0; border-top: 1px solid var(--line); font-size: 13px; }
  .device-line { display: flex; gap: 8px; align-items: center; }
  .fw { color: var(--muted); font-size: 12px; }
  .oem { color: var(--warn); }
  main { display: grid; gap: 18px; align-content: start; padding: 24px 28px 32px; overflow: auto; min-width: 0; min-height: 0; }
  .banner { padding: 8px 12px; border-radius: var(--r-m); color: var(--danger); background: color-mix(in srgb, var(--danger) 14%, var(--panel)); }
  .update { padding: 10px 12px; background: var(--panel); border: 1px solid var(--line); border-radius: var(--r-m); animation: toast-in 180ms ease-out; }
  .update-text { flex: 1; min-width: 0; }
  .update .meter { flex: 0 1 160px; }
  .update button[aria-disabled="true"] { opacity: 0.45; cursor: default; pointer-events: none; }
  .toast { position: fixed; bottom: 16px; right: 16px; max-width: 420px; padding: 10px 14px; background: var(--raised); border: 1px solid var(--line); border-radius: var(--r-m); user-select: text; animation: toast-in 180ms ease-out; }
  @keyframes toast-in { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
</style>
