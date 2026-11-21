<script lang="ts">
  import '../app.css';
  import '$lib/theme.svelte';
  let { children } = $props();
</script>

{@render children()}
