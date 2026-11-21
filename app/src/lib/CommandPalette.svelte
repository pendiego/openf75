<script module lang="ts">
  export type Command = { id: string; label: string; disabled?: boolean; run: () => void };
</script>

<script lang="ts">
  // Combobox + listbox pattern (WAI-ARIA APG). The native <dialog> gives focus trapping, inert background and Escape.
  import Kbd from '$lib/Kbd.svelte';

  let { commands, open = $bindable(false) }: { commands: Command[]; open?: boolean } = $props();

  let dlg: HTMLDialogElement;
  let query = $state('');
  let active = $state(0);
  let opener: HTMLElement | null = null;

  const q = $derived(query.trim().toLowerCase());
  const matches = $derived(commands.filter((c) => c.label.toLowerCase().includes(q)));
  const cur = $derived(matches.length ? Math.min(active, matches.length - 1) : -1);

  $effect(() => {
    if (open && !dlg.open) {
      opener = document.activeElement as HTMLElement | null;
      query = '';
      active = 0;
      dlg.showModal();
    } else if (!open && dlg.open) dlg.close();
  });

  $effect(() => {
    if (cur >= 0) document.getElementById(`cmdpal-${cur}`)?.scrollIntoView({ block: 'nearest' });
  });

  // Runs on every close path (Escape, backdrop, command run) so focus returns to the opener.
  function onClose() {
    open = false;
    opener?.focus();
  }

  function run(c: Command) {
    if (c.disabled) return;
    open = false;
    c.run();
  }

  function onKey(e: KeyboardEvent) {
    const n = matches.length;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (!n) return;
      const step = e.key === 'ArrowDown' ? 1 : -1;
      active = (cur + step + n) % n;
    } else if (e.key === 'Enter' && cur >= 0) {
      e.preventDefault();
      run(matches[cur]);
    }
  }
</script>

<svelte:window
  onkeydown={(e) => {
    // Skip keys a focused control already consumed (e.g. macro recording).
    if (!e.defaultPrevented && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      open = !open;
    }
  }} />

<dialog
  bind:this={dlg}
  class="cp"
  aria-label="Paleta de comandos"
  onclose={onClose}
  onclick={(e) => e.target === dlg && (open = false)}>
  <input
    class="q"
    role="combobox"
    aria-expanded="true"
    aria-controls="cmdpal-list"
    aria-autocomplete="list"
    aria-activedescendant={cur >= 0 ? `cmdpal-${cur}` : undefined}
    aria-label="Buscar comandos"
    placeholder="Buscar comandos…"
    autocomplete="off"
    spellcheck="false"
    bind:value={query}
    oninput={() => (active = 0)}
    onkeydown={onKey} />
  <ul id="cmdpal-list" class="opts" role="listbox" aria-label="Comandos">
    {#each matches as c, i (c.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -- keyboard runs the option through the combobox input -->
      <li
        id="cmdpal-{i}"
        role="option"
        aria-selected={i === cur}
        aria-disabled={c.disabled || undefined}
        onpointermove={() => (active = i)}
        onclick={() => run(c)}>{c.label}</li>
    {:else}
      <li class="empty" role="presentation">Nenhum comando encontrado</li>
    {/each}
  </ul>
  <span class="sr" role="status">{matches.length ? `${matches.length} comandos` : 'Nenhum comando encontrado'}</span>
  <div class="hint">
    <span><Kbd keys={['Ctrl', 'K']} /> alterna</span>
    <span><Kbd keys={['↑', '↓']} /> navega</span>
    <span><Kbd keys={['Enter']} /> executa</span>
    <span><Kbd keys={['Esc']} /> fecha</span>
  </div>
</dialog>

<style>
  dialog.cp {
    width: min(560px, calc(100vw - 32px));
    max-height: min(60vh, 480px);
    margin: 12vh auto 0;
    padding: 0;
    background: var(--panel);
    color: var(--text);
    border: 1px solid var(--line);
    border-radius: var(--r-l);
  }
  dialog.cp[open] { display: flex; flex-direction: column; }
  dialog.cp::backdrop { background: color-mix(in srgb, var(--bg) 60%, transparent); }
  .q {
    flex-shrink: 0;
    margin: 0;
    padding: 14px 16px;
    font: inherit;
    font-size: 15px;
    color: var(--text);
    background: none;
    border: 0;
    border-bottom: 1px solid var(--line);
    border-radius: 0;
    box-shadow: none;
    outline-offset: -2px;
  }
  .opts { flex: 1; min-height: 0; margin: 0; padding: 6px; list-style: none; overflow-y: auto; }
  .opts li { padding: 8px 10px; border-radius: var(--r-s); cursor: pointer; }
  .opts li[aria-selected="true"] { background: var(--accent); color: var(--accent-ink); }
  .opts li[aria-disabled="true"] { opacity: 0.45; cursor: default; }
  .opts .empty { color: var(--muted); cursor: default; }
  .hint {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    flex-shrink: 0;
    padding: 8px 12px;
    font-size: 12px;
    color: var(--muted);
    border-top: 1px solid var(--line);
  }
  .hint span { display: inline-flex; align-items: center; gap: 6px; }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
