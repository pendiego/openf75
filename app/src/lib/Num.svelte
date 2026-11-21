<script lang="ts">
  import { Tween } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { dur } from './motion';

  let { value, digits = 0 }: { value: number; digits?: number } = $props();
  const t = Tween.of(() => value, { duration: dur(160), easing: cubicOut });
</script>

<output class="num">{t.current.toFixed(digits)}</output>
