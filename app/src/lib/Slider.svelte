<script lang="ts">
  import Num from './Num.svelte';

  let {
    id,
    label,
    value = $bindable(),
    min,
    max,
    step = 1,
    digits = 0,
    disabled = false,
    format = (v: number) => String(v),
    oninput,
    onchange,
  }: {
    id: string;
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    digits?: number;
    disabled?: boolean;
    /** Human reading of the value, exposed as aria-valuetext. */
    format?: (v: number) => string;
    oninput?: (v: number) => void;
    onchange?: (v: number) => void;
  } = $props();

  // Decimals of the step, so 0.1 steps don't drift to 1.2000000000000002.
  const dec = $derived((String(step).split('.')[1] ?? '').length);
  // PageUp/PageDown jump a fifth of the range, only where the native step is too small to be useful.
  const big = $derived(max - min > 10 ? Math.max(step, Math.round((max - min) / 5 / step) * step) : 0);

  function set(v: number) {
    value = Number(v.toFixed(dec));
  }

  function onInput(e: Event) {
    set(+(e.currentTarget as HTMLInputElement).value);
    oninput?.(value);
  }

  function onChange(e: Event) {
    set(+(e.currentTarget as HTMLInputElement).value);
    onchange?.(value);
  }

  function onKeydown(e: KeyboardEvent) {
    if (!big || (e.key !== 'PageUp' && e.key !== 'PageDown')) return;
    e.preventDefault();
    set(Math.min(max, Math.max(min, value + (e.key === 'PageUp' ? big : -big))));
    oninput?.(value);
    onchange?.(value);
  }
</script>

<div class="slider field" class:disabled>
  <label for={id}>{label}</label>
  <input
    {id}
    type="range"
    {min}
    {max}
    {step}
    {disabled}
    value={value}
    aria-valuetext={format(value)}
    oninput={onInput}
    onchange={onChange}
    onkeydown={onKeydown}
  />
  <span aria-hidden="true"><Num {value} {digits} /></span>
</div>

<style>
  .slider {
    grid-template-columns: 96px minmax(0, 1fr) auto;
  }
  label {
    color: var(--muted);
    cursor: pointer;
  }
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
    border-radius: 4px;
  }
  .disabled label {
    opacity: 0.5;
    cursor: default;
  }
  .disabled input {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
