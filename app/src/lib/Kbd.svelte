<script lang="ts">
  // Keycap chips for one key or a modifier combo. Labels arrive already formatted (see comboLabels in hid.ts).
  // Chip layout follows the idea of meta/astryx Kbd (MIT): role="img" wrapper with a spoken name, glyphs hidden from AT.
  let { keys }: { keys: string[] } = $props();
</script>

<span class="kbd" role="img" aria-label={keys.join(' + ')}>
  {#each keys as k, i (i)}
    <kbd aria-hidden="true">{k}</kbd>
  {/each}
</span>

<style>
  .kbd { display: inline-flex; flex-wrap: wrap; gap: 4px; align-items: center; vertical-align: middle; }
  kbd {
    display: inline-flex; align-items: center; justify-content: center; box-sizing: border-box;
    min-width: 22px; height: 22px; padding: 0 6px;
    font: 500 12px/1 var(--mono); color: var(--text); user-select: none;
    background: var(--raised); border: 1px solid var(--line); border-bottom-width: 2px; border-radius: var(--r-s);
  }
</style>
