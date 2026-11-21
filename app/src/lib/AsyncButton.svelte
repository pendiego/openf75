<script lang="ts">
  import { onDestroy } from 'svelte';
  import { fly } from 'svelte/transition';
  import { errMsg } from './api';
  import { dur } from './motion';

  // run() throws on failure: the button notifies the error and shakes; success shows a check for a moment.
  let {
    run,
    notify,
    label,
    busyLabel,
    class: cls = '',
    disabled = false,
  }: {
    run: () => Promise<unknown>;
    notify: (m: string) => void;
    label: string;
    busyLabel: string;
    class?: string;
    disabled?: boolean;
  } = $props();

  let phase = $state<'idle' | 'busy' | 'done'>('idle');
  let shake = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  onDestroy(() => clearTimeout(timer));

  async function click() {
    if (phase === 'busy') return;
    clearTimeout(timer);
    shake = false;
    phase = 'busy';
    try {
      await run();
      phase = 'done';
      timer = setTimeout(() => (phase = 'idle'), 1200);
    } catch (e) {
      phase = 'idle';
      notify(errMsg(e));
      requestAnimationFrame(() => (shake = true));
    }
  }
</script>

<button
  class={[cls, shake && 'shake']}
  disabled={disabled || phase === 'busy'}
  aria-busy={phase === 'busy'}
  onclick={click}
  onanimationend={() => (shake = false)}
>
  {#key phase}
    <span class="swap" in:fly={{ y: 6, duration: dur(160) }}>
      {#if phase === 'done'}
        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M3 8.5l3.2 3L13 4.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
      {/if}
      {phase === 'busy' ? busyLabel : label}
    </span>
  {/key}
</button>

<style>
  .swap { display: inline-flex; align-items: center; justify-content: center; gap: 6px; }
</style>
