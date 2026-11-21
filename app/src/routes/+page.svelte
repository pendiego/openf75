<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { fade, fly, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { dur } from '$lib/motion';
  import { api, errMsg, type Status } from '$lib/api';
  import { version } from '../../package.json';
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
    { id: 'audio', label: 'Áudio' },
    { id: 'teclas', label: 'Teclas' },
    { id: 'macros', label: 'Macros' },
    { id: 'energia', label: 'Energia' },
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

  // Plain, not state: only read when `tab` changes, to pick the slide direction.
  let prevIdx = 0;
  const tabIdx = (id: TabId) => TABS.findIndex((t) => t.id === id);
  const selectTab = (id: TabId) => {
    prevIdx = tabIdx(tab);
    tab = id;
  };
  const slideX = $derived(tabIdx(tab) > prevIdx ? 12 : -12);

  let nav: HTMLElement;
  let ind = $state<{ x: number; w: number } | null>(null);
  const measure = () => {
    const b = nav?.querySelector<HTMLElement>('[aria-current="page"]');
    ind = b ? { x: b.offsetLeft, w: b.offsetWidth } : null;
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
  const linkState = $derived<'ok' | 'warn' | 'off'>(
    !status?.connected ? 'off' : status.online === false ? 'warn' : 'ok',
  );
  const pillText = $derived(
    linkState === 'ok' ? 'Conectado' : linkState === 'warn' ? 'Dormindo' : 'Desconectado',
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

  $effect(() => {
    void tab;
    measure();
  });
  $effect(() => {
    const ro = new ResizeObserver(measure);
    ro.observe(nav);
    return () => ro.disconnect();
  });
</script>

<div class="shell">
  <header>
    <div class="brand">
      <span class="wordmark">OpenF75</span>
      <h1>AULA F75</h1>
    </div>
    <nav class="tabs" aria-label="Seções" bind:this={nav}>
      {#if ind}<span class="ind" style:left="{ind.x}px" style:width="{ind.w}px"></span>{/if}
      {#each TABS as t (t.id)}
        <button
          aria-current={tab === t.id ? 'page' : undefined}
          disabled={audioBusy && t.id !== 'audio'}
          onclick={() => selectTab(t.id)}>{t.label}</button>
      {/each}
    </nav>
    <div class="pill">
      <span class="dot" class:ok={linkState === 'ok'} class:warn={linkState === 'warn'}></span>
      {pillText}
    </div>
  </header>

  <main>
    {#if status?.writable_error}
      <div class="banner" role="alert" transition:slide={{ duration: dur(200) }}>{status.writable_error}</div>
    {/if}
    {#if statusErr && statusErr !== status?.writable_error}
      <div class="banner" role="alert" transition:slide={{ duration: dur(200) }}>{statusErr}</div>
    {/if}
    {#key tab}<div class="tab" in:fly={{ x: slideX, duration: dur(200), easing: cubicOut }}>
      {#if tab === 'luz'}<Iluminacao {notify} />{/if}
      {#if tab === 'tecla_rgb'}<PorTecla {notify} />{/if}
      {#if tab === 'teclas'}<Teclas {notify} />{/if}
      {#if tab === 'macros'}<Macros {notify} />{/if}
      {#if tab === 'energia'}<Energia {notify} />{/if}
      {#if tab === 'audio'}<Audio {notify} bind:busy={audioBusy} />{/if}
      {#if tab === 'perfis'}<Perfis {notify} />{/if}
    </div>{/key}
  </main>

  <footer>
    <div class="left">
      <span class="dot" class:ok={linkState === 'ok'} class:warn={linkState === 'warn'}></span>
      <span>{badge}</span>
      {#if status?.oem_running}<span class="oem">OemDrv.exe aberto: feche antes de gravar</span>{/if}
    </div>
    <div class="right">
      {#if status?.fw != null}<span class="num">fw {status.fw.toString(16)}</span>{/if}
      <span class="num">v{version}</span>
    </div>
  </footer>
</div>

{#if toast}
  <div class="toast" role="status" aria-live="polite"
    in:fly={{ y: 8, duration: dur(160), easing: cubicOut }}
    out:fly={{ y: 8, duration: dur(120), easing: cubicOut }}>
    {#key toast}<span in:fade={{ duration: dur(120) }}>{toast}</span>{/key}
  </div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-rows: 56px minmax(0, 1fr) 32px;
    height: 100dvh;
    overflow: hidden;
  }
  header {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    gap: 16px;
    padding: 0 20px;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
  }
  .brand { display: flex; align-items: baseline; gap: 10px; min-width: 0; white-space: nowrap; }
  .wordmark { color: var(--muted); font-size: 13px; }
  h1 { margin: 0; font: 600 18px/1.2 var(--display); letter-spacing: -0.01em; overflow: hidden; text-overflow: ellipsis; }
  .tabs {
    display: flex;
    position: relative;
    gap: 2px;
    padding: 2px;
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--r-m);
  }
  .ind {
    position: absolute;
    top: 2px;
    bottom: 2px;
    background: var(--panel);
    border-radius: var(--r-s);
    pointer-events: none;
    transition: left var(--dur-base) var(--ease-out), width var(--dur-base) var(--ease-out);
  }
  .tabs button {
    padding: 4px 9px;
    position: relative;
    z-index: 1;
    border: 0;
    border-radius: var(--r-s);
    background: none;
    box-shadow: none;
    color: var(--muted);
    font: inherit;
    font-size: 13px;
    white-space: nowrap;
  }
  .tabs button:hover:not(:disabled) { background: none; color: var(--text); }
  .tabs button[aria-current="page"] { color: var(--text); }
  .pill {
    justify-self: end;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border: 1px solid var(--line);
    border-radius: 999px;
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
  }
  main {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }
  .tab { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .banner { margin: 8px 16px 0; flex-shrink: 0; padding: 8px 12px; border-radius: var(--r-m); color: var(--danger); background: color-mix(in srgb, var(--danger) 14%, var(--panel)); }
  footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 0 20px;
    border-top: 1px solid var(--line);
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
  }
  footer .left, footer .right { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .oem { color: var(--warn); }
  .toast { position: fixed; bottom: 44px; right: 16px; max-width: 420px; padding: 10px 14px; background: var(--raised); border: 1px solid var(--line); border-radius: var(--r-m); user-select: text; }
</style>
