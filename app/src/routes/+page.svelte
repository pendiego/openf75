<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { fade, fly, slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { dur } from '$lib/motion';
  import { isMac, modLabel } from '$lib/platform';
  import { api, errMsg, type Status } from '$lib/api';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { version } from '../../package.json';
  import Iluminacao from '$lib/tabs/Iluminacao.svelte';
  import PorTecla from '$lib/tabs/PorTecla.svelte';
  import Teclas from '$lib/tabs/Teclas.svelte';
  import Macros from '$lib/tabs/Macros.svelte';
  import Energia from '$lib/tabs/Energia.svelte';
  import Audio from '$lib/tabs/Audio.svelte';
  import Perfis from '$lib/tabs/Perfis.svelte';
  import CommandPalette, { type Command } from '$lib/CommandPalette.svelte';
  import { theme, setTheme } from '$lib/theme.svelte';
  import Kbd from '$lib/Kbd.svelte';

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

  const THEME_MODES = [
    { id: 'system', label: 'Sistema' },
    { id: 'light', label: 'Claro' },
    { id: 'dark', label: 'Escuro' },
  ] as const;
  const themeMode = $derived(THEME_MODES.find((m) => m.id === theme.mode) ?? THEME_MODES[0]);
  const cycleTheme = () => setTheme(THEME_MODES[(THEME_MODES.indexOf(themeMode) + 1) % THEME_MODES.length].id);

  // Ctrl+K is handled inside CommandPalette; the header button opens the same palette.
  let paletteOpen = $state(false);
  const commands: Command[] = $derived([
    ...TABS.map((t) => ({
      id: `tab:${t.id}`,
      label: t.label,
      disabled: audioBusy && t.id !== 'audio',
      run: () => selectTab(t.id),
    })),
    ...THEME_MODES.map((m) => ({ id: `theme:${m.id}`, label: `Tema: ${m.label}`, run: () => setTheme(m.id) })),
  ]);

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
      <h1>OpenF75</h1>
      <span class="device">AULA F75</span>
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
    <div class="end">
      <button class="tool search" aria-label="Buscar comandos" aria-keyshortcuts={isMac ? 'Meta+K' : 'Control+K'} title="Buscar comandos ({modLabel}+K)" onclick={() => (paletteOpen = true)}>
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="11" cy="11" r="6.5" /><path d="m20 20-4.2-4.2" /></svg>
        <Kbd keys={[modLabel, 'K']} />
      </button>
      <button
        class="tool icon"
        aria-label="Tema: {themeMode.label}. Trocar"
        title="Tema: {themeMode.label}"
        onclick={cycleTheme}>
        <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          {#if theme.mode === 'light'}
            <circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
          {:else if theme.mode === 'dark'}
            <path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" />
          {:else}
            <rect x="3" y="4" width="18" height="12" rx="2" /><path d="M8 20h8M12 16v4" />
          {/if}
        </svg>
      </button>
      <div class="pill" title={pillText}>
        <span class="dot" class:ok={linkState === 'ok'} class:warn={linkState === 'warn'}></span>
        <span class="pill-text">{badge}</span>
      </div>
    </div>
  </header>

  <main>
    {#if status?.writable_error}
      <div class="banner" role="alert" transition:slide={{ duration: dur(200) }}>{status.writable_error}</div>
    {/if}
    {#if statusErr && statusErr !== status?.writable_error}
      <div class="banner" role="alert" transition:slide={{ duration: dur(200) }}>{statusErr}</div>
    {/if}
    <span class="sr" role="status">{update ? `Atualização ${update.version} disponível` : ''}</span>
    {#if update}
      <section class="update row" aria-label="Atualização disponível" transition:slide={{ duration: dur(200) }}>
        <span class="update-text">Versão {update.version} disponível</span>
        <button class="primary" aria-disabled={updateStep !== 'idle'} onclick={installUpdate}>
          {updateStep === 'idle' ? 'Atualizar' : updateStep === 'download' ? 'Baixando…' : 'Instalando…'}
        </button>
        <button class="ghost" aria-disabled={updateStep !== 'idle'} onclick={dismissUpdate}>Depois</button>
        {#if updatePct != null}
          <progress class="meter" max="1" value={updatePct} aria-label="Progresso do download"></progress>
        {/if}
      </section>
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
      {#if status?.oem_running}<span class="oem">OemDrv.exe aberto: feche antes de gravar</span>{/if}
    </div>
    <div class="right">
      {#if status?.fw != null}<span class="num">fw {status.fw.toString(16)}</span>{/if}
      <span class="num">v{version}</span>
    </div>
  </footer>
</div>
<CommandPalette commands={commands} bind:open={paletteOpen} />

<div class="toast-live" role="status" aria-live="polite">
  {#if toast}
    <div class="toast" in:fly={{ y: 8, duration: dur(160), easing: cubicOut }}
      out:fly={{ y: 8, duration: dur(120), easing: cubicOut }}>
      {#key toast}<span in:fade={{ duration: dur(120) }}>{toast}</span>{/key}
    </div>
  {/if}
</div>

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
  .brand { display: flex; align-items: baseline; gap: 8px; min-width: 0; white-space: nowrap; }
  h1 { margin: 0; font: 600 16px/1.2 var(--display); letter-spacing: -0.01em; }
  .device { color: var(--muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; }
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
    height: 28px;
    padding: 0 10px;
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
  .tool {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 6px 0 10px;
    border: 1px solid var(--line);
    border-radius: var(--r-m);
    background: none;
    box-shadow: none;
    color: var(--muted);
    font-size: 13px;
  }
  .tool:hover { background: var(--raised); color: var(--text); }
  .tool.icon { width: 32px; padding: 0; justify-content: center; }
  .tool :global(kbd) { height: 20px; min-width: 20px; padding: 0 5px; font-size: 11px; color: var(--muted); }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
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
  @media (max-width: 1000px) {
    .search :global(.kbd), .device { display: none; }
    .pill-text { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
    .search, .pill { width: 32px; padding: 0; justify-content: center; }
  }
  .oem { color: var(--warn); }
  .update { margin: 8px 16px 0; flex-shrink: 0; padding: 10px 12px; background: var(--panel); border: 1px solid var(--line); border-radius: var(--r-m); }
  .update-text { flex: 1; min-width: 0; }
  .update .meter { flex: 0 1 160px; }
  .update button[aria-disabled="true"] { opacity: 0.45; cursor: default; pointer-events: none; }
  .toast-live { position: fixed; bottom: 44px; right: 16px; max-width: 420px; }
  .toast { padding: 10px 14px; background: var(--raised); border: 1px solid var(--line); border-radius: var(--r-m); user-select: text; }
  .end { display: flex; align-items: center; gap: 8px; justify-self: end; min-width: 0; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
</style>
