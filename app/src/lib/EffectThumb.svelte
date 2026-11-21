<script lang="ts">
  // Miniature of the keyboard lit by one effect, drawn from the same simulation as the 3D view (fx.ts).
  import { CAPS } from './f75art';
  import { makeFx } from './fx';
  import { dur } from './motion';
  import { theme } from './theme.svelte';

  let { name, live = false }: { name: string; live?: boolean } = $props();

  const W = 1760, H = 766; // CAPS coordinate space
  const S = 0.1;
  const rects = Object.entries(CAPS).map(([m, [x1, y1, x2, y2]]) => ({ m: +m, x: x1 * S, y: y1 * S, w: (x2 - x1) * S, h: (y2 - y1) * S }));
  const mids = rects.filter((r) => r.y > 15 && r.y < 50).map((r) => r.m); // keys for simulated presses

  let canvas: HTMLCanvasElement;

  $effect(() => {
    void theme.mode; // repaint the unlit keys with the new theme's token
    const fx = makeFx({ name, colour: null, speed: 2, brightness: 9 });
    const ctx = canvas.getContext('2d')!;
    const dpr = devicePixelRatio || 1;
    canvas.width = W * S * dpr;
    canvas.height = H * S * dpr;
    ctx.scale(dpr, dpr);
    const off = getComputedStyle(canvas).getPropertyValue('--raised').trim();

    const draw = (t: number) => {
      ctx.clearRect(0, 0, W * S, H * S);
      for (const r of rects) {
        const [cr, cg, cb] = fx.color(r.m, t);
        const lit = cr + cg + cb > 0.02;
        ctx.fillStyle = lit ? `rgb(${(cr * 255) | 0} ${(cg * 255) | 0} ${(cb * 255) | 0})` : off;
        ctx.beginPath();
        ctx.roundRect(r.x + 0.6, r.y + 0.6, r.w - 1.2, r.h - 1.2, 1.5);
        ctx.fill();
      }
    };

    // Reactive effects stay dark until a press: seed one so the still frame shows the effect.
    if (fx.reactive) fx.press(mids[Math.floor(mids.length / 2)] ?? 0, 0.2);
    if (!live || dur(1) === 0) {
      draw(fx.reactive ? 0.6 : 1.5);
      return;
    }
    let raf = 0, nextPress = 0;
    const t0 = performance.now();
    const frame = (now: number) => {
      const t = (now - t0) / 1000;
      if (fx.reactive && t >= nextPress) {
        fx.press(mids[Math.floor(Math.random() * mids.length)] ?? 0, t);
        nextPress = t + 0.9;
      }
      draw(t);
      raf = requestAnimationFrame(frame);
    };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
</script>

<canvas bind:this={canvas} class="thumb" aria-hidden="true"></canvas>

<style>
  .thumb { display: block; width: 100%; aspect-ratio: 1760 / 766; border-radius: var(--r-s); background: var(--bg); }
</style>
