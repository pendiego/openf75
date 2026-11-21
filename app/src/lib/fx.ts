import { CAPS } from './f75art';

export type RGB = [number, number, number]; // sRGB, each 0..1
export type FxOptions = {
  name: string; // Effect.name from the backend table
  colour: RGB | null; // null = multicolour
  speed: number; // 0..4
  brightness: number; // 0..9
  perkey?: RGB[]; // Self-define colours indexed by matrix
};
export interface Fx {
  reactive: boolean; // driven by key presses
  animated: boolean; // colour changes with time on its own (false: static until a press)
  color(m: number, t: number): RGB; // t in seconds, monotonic
  press(m: number, t: number): void;
}

type Geo = { x: number; y: number; row: number; path: number }; // x, y in key pitches; path = serpentine index
type Press = { m: number; u: number }; // u = press time in effect units
type Hit = { v: number; hue: number };
type Shape = (g: Geo, m: number, tt: number) => RGB; // tt in effect units

const BLACK: RGB = [0, 0, 0];
const WHITE: RGB = [1, 1, 1];
const PITCH = 103; // px per key pitch in CAPS space
const ROW_PX = [104, 242, 343, 445, 548, 647]; // row centres in CAPS space
// Firmware speed steps; the 0.9 (speed 2) step maps to one effect unit per second.
// shortcut: speed steps are approximated from the UI, not measured on firmware, revisit on real keyboard
const RATE = [0.45, 0.65, 0.9, 1.25, 1.7];
const LIFE = 3; // press history kept for this many effect units
const PHI = 0.618034; // golden ratio conjugate, spreads per-key hues
// Reactive effects stay dark on the keyboard until a key is pressed (confirmed for Flash away, Ripples shining
// and Shadow disappear on a 2.4G F75).
const REACTIVE: Record<string, true> = {
  'Flash away': true,
  'Ripples shining': true,
  'Shadow disappear': true,
  Reaction: true,
};
export const isReactive = (name: string) => REACTIVE[name] === true;

const cells = Object.entries(CAPS).map(([k, [x1, y1, x2, y2]]) => {
  const cy = (y1 + y2) / 2;
  let row = 0;
  for (let i = 1; i < ROW_PX.length; i++) if (Math.abs(ROW_PX[i] - cy) < Math.abs(ROW_PX[row] - cy)) row = i;
  return { m: Number(k), x: (x1 + x2) / 2 / PITCH, y: cy / PITCH, row };
});
// Serpentine order: rows top to bottom, alternating direction.
cells.sort((a, b) => a.row - b.row || (a.row % 2 === 0 ? a.x - b.x : b.x - a.x));
const GEO = new Map<number, Geo>(cells.map((c, path) => [c.m, { x: c.x, y: c.y, row: c.row, path }]));
const N = cells.length;
const KEYS = cells.map((c) => c.m);
const xs = cells.map((c) => c.x);
const ys = cells.map((c) => c.y);
const MINX = Math.min(...xs);
const MAXX = Math.max(...xs);
const CX = (MINX + MAXX) / 2;
const CY = (Math.min(...ys) + Math.max(...ys)) / 2;

const clamp01 = (v: number): number => (v > 0 ? (v < 1 ? v : 1) : 0); // NaN -> 0
const fract = (v: number): number => v - Math.floor(v);
const mul = (c: RGB, v: number): RGB => [c[0] * v, c[1] * v, c[2] * v];

// Full-saturation hue wheel; h in turns.
function hsv(h: number): RGB {
  const f = fract(h) * 6;
  return [clamp01(Math.abs(f - 3) - 1), clamp01(2 - Math.abs(f - 2)), clamp01(2 - Math.abs(f - 4))];
}

// Pure integer hash to [0, 1), so random spawns are reproducible for any t.
function hash(a: number, b: number): number {
  let h = Math.imul(a, 0x9e3779b1) ^ Math.imul(b, 0x85ebca77);
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

// Random spawns: slot s fires on a key at (s + jitter) / perTt; env gets age as a fraction of life.
function sparkle(m: number, tt: number, perTt: number, salt: number, life: number, env: (a: number) => number): Hit {
  const last = Math.floor(tt * perTt);
  const span = Math.ceil(life * perTt) + 1;
  let best: Hit = { v: 0, hue: 0 };
  for (let s = last - span; s <= last; s++) {
    if (KEYS[Math.floor(hash(s, salt) * N)] !== m) continue;
    const a = (tt - (s + hash(s, salt + 1)) / perTt) / life;
    if (a < 0 || a >= 1) continue;
    const v = env(a);
    if (v > best.v) best = { v, hue: hash(s, salt + 2) };
  }
  return best;
}

// Strongest press contribution at tt; f returns the value for press p at age a.
function hit(presses: Press[], tt: number, f: (p: Press, a: number) => number): Hit {
  let best: Hit = { v: 0, hue: 0 };
  for (const p of presses) {
    const a = tt - p.u;
    if (a < 0) continue;
    const v = f(p, a);
    if (v > best.v) best = { v, hue: fract(p.m * PHI) };
  }
  return best;
}

export function makeFx(o: FxOptions): Fx {
  const scale = (RATE[Math.round(o.speed)] ?? RATE[2]) / RATE[2];
  const gain = (o.brightness + 1) / 10;
  const colour = o.colour;
  // shortcut: press history is a plain list pruned lazily on press, upgrade if many presses per second
  let presses: Press[] = [];

  // Fixed colour if set, otherwise the multicolour hue; v scales brightness.
  const pick = (hue: number, v: number): RGB => mul(colour ?? hsv(hue), v);

  const fixed: Shape = (g) => pick(g.x / 17, 1);
  const table: Record<string, Shape> = {
    'Fixed on': fixed,
    Respire: (_g, _m, tt) => {
      const env = 0.525 - 0.475 * Math.cos((Math.PI * tt) / 2); // 4-unit period, 0.05..1
      return pick(Math.floor(tt / 4) * PHI, env);
    },
    Rainbow: (g, _m, tt) => hsv((g.x - tt * 1.5) / 17),
    'Flash away': (g, _m, tt) => {
      const h = hit(presses, tt, (p, a) => {
        const pg = GEO.get(p.m);
        if (!pg || pg.row !== g.row) return 0;
        const dx = Math.abs(g.x - pg.x);
        const fade = Math.max(0, 1 - a / 2.5);
        // Front moves 7 keys per unit both ways along the row; the pressed key stays lit while fading.
        return Math.max(0, 1 - Math.abs(dx - 7 * a) / 1.2) * fade + (dx < 0.5 ? fade : 0);
      });
      return pick(h.hue, h.v);
    },
    Raindrops: (_g, m, tt) => {
      const h = sparkle(m, tt, 6, 11, 1, (a) => Math.min(1, a / 0.15, (1 - a) / 0.85));
      return pick(h.hue, h.v);
    },
    'Rainbow wheel': (g, _m, tt) => {
      const th = Math.atan2(g.y - CY, g.x - CX);
      const beam = ((Math.cos(th - 1.5 * tt) + 1) / 2) ** 6;
      return pick(th / (2 * Math.PI) + tt * 0.15, colour ? 0.08 + 0.92 * beam : 1);
    },
    'Ripples shining': (g, _m, tt) => {
      const h = hit(presses, tt, (p, a) => {
        const pg = GEO.get(p.m);
        if (!pg) return 0;
        const d = Math.hypot(g.x - pg.x, g.y - pg.y);
        // Ring expands 6 keys per unit, 0.8 key wide, dying out by age 2.5.
        return Math.max(0, 1 - Math.abs(d - 6 * a) / 0.4) * Math.max(0, 1 - a / 2.5);
      });
      return pick(h.hue, h.v);
    },
    'Stars twinkle': (_g, m, tt) => {
      const h = sparkle(m, tt, 4, 23, 1.5, (a) => Math.sin(Math.PI * a));
      const bg = mul(colour ?? WHITE, 0.08);
      const star = colour ?? hsv(h.hue);
      return [0, 1, 2].map((i) => bg[i] + (star[i] - bg[i]) * h.v) as RGB;
    },
    // Observed on the keyboard: dark until a key is pressed. What follows a press (the key lights and fades
    // slowly, faint halo on its neighbours) is an approximation.
    'Shadow disappear': (g, _m, tt) => {
      const h = hit(presses, tt, (p, a) => {
        const pg = GEO.get(p.m);
        if (!pg) return 0;
        const d = Math.hypot(g.x - pg.x, g.y - pg.y);
        const fade = Math.max(0, 1 - a / 2);
        return (d < 0.5 ? 1 : Math.max(0, 0.35 * (1 - (d - 0.5) / 0.9))) * fade;
      });
      return pick(h.hue, h.v);
    },
    'Retro snake': (g, _m, tt) => {
      const head = (tt * 5) % N;
      let d = head - g.path; // distance behind the head along the path
      if (d < 0) d += N;
      return pick(g.path / N, Math.max(0, 1 - d / 6));
    },
    'Neon stream': (g, _m, tt) => {
      if (!colour) return hsv(g.x * 0.04 - tt * 0.2);
      const band = (Math.cos(2 * Math.PI * (g.x / 6 - 0.6 * tt)) + 1) / 2;
      return mul(colour, 0.1 + 0.9 * band ** 4);
    },
    Reaction: (_g, m, tt) => {
      const h = hit(presses, tt, (p, a) => {
        if (p.m !== m) return 0;
        return a < 0.6 ? 1 : Math.max(0, 1 - (a - 0.6) / 0.8);
      });
      return pick(h.hue, h.v);
    },
    'Sine wave': (g, _m, tt) => {
      const yc = 3.6 + 2 * Math.sin(0.45 * g.x - 1.6 * tt);
      return pick(g.x / 16 + 0.1 * tt, clamp01((1 - Math.abs(g.y - yc)) / 0.5));
    },
    'Retinue scanning': (g, _m, tt) => {
      const L = MAXX - MINX;
      const u = (tt * 5) % (2 * L);
      const dir = u < L ? 1 : -1;
      const bar = MINX + (u < L ? u : 2 * L - u);
      const dx = g.x - bar;
      const behind = -dir * dx; // > 0 on the trailing side
      let v = 0;
      if (Math.abs(dx) <= 0.5) v = 1;
      else if (behind > 0.5) {
        const f = Math.max(0, 1 - (behind - 0.5) / 4);
        v = f * f;
      }
      return pick(0.08 * tt, v);
    },
    'Rotating windmill': (g, _m, tt) => {
      const rel = Math.atan2(g.y - CY, g.x - CX) - 1.2 * tt;
      const blade = Math.round(rel / (Math.PI / 2));
      const off = rel - blade * (Math.PI / 2);
      const v = clamp01(0.5 + (Math.PI / 8 - Math.abs(off)) / 0.15);
      return mul(hsv(((((blade % 4) + 4) % 4) / 4) + 0.04 * tt), v);
    },
    'Colorful waterfall': (g, _m, tt) => hsv((g.y - tt) / 7),
    Blossoming: (g, _m, tt) => {
      const d = Math.hypot(g.x - CX, g.y - CY);
      const ring = ((Math.cos((2 * Math.PI * (d - 2 * tt)) / 3) + 1) / 2) ** 4;
      return mul(hsv(d / 10 + 0.05 * tt), ring);
    },
    'Rotating storm': (g, _m, tt) => {
      const th = Math.atan2(g.y - CY, g.x - CX);
      const r = Math.hypot(g.x - CX, g.y - CY);
      const ph = 3 * th + 0.8 * r - 1.5 * tt; // three spiral arms
      const v = 0.1 + 0.9 * ((Math.cos(ph) + 1) / 2) ** 3;
      return mul(colour ?? hsv(ph / (2 * Math.PI)), v);
    },
    'Self-define': (_g, m) => o.perkey?.[m] ?? BLACK,
    Off: () => BLACK,
  };
  // Own-property check so names like 'constructor' fall back to Fixed on.
  const shape = Object.prototype.hasOwnProperty.call(table, o.name) ? table[o.name] : fixed;

  return {
    reactive: REACTIVE[o.name] === true,
    // Shapes that ignore time: no need to redraw every frame (unknown names fall back to Fixed on).
    animated: shape !== fixed && o.name !== 'Self-define' && o.name !== 'Off',
    color(m, t) {
      const g = GEO.get(m);
      if (!g) return BLACK;
      const c = shape(g, m, t * scale);
      return [clamp01(c[0] * gain), clamp01(c[1] * gain), clamp01(c[2] * gain)];
    },
    press(m, t) {
      if (!GEO.has(m)) return;
      const u = t * scale;
      presses = presses.filter((p) => u - p.u <= LIFE);
      presses.push({ m, u });
    },
  };
}
