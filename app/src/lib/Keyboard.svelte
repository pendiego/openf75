<script lang="ts" module>
  import * as THREE from 'three';
  import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
  import { MeshoptDecoder } from 'three/addons/libs/meshopt_decoder.module.js';
  import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';
  import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
  import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
  import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
  import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
  import modelUrl from './f75.glb?url';

  type Cap = {
    legends: THREE.MeshStandardMaterial[];
    glow: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshBasicMaterial>;
    corners: THREE.Vector3[];
    mark: THREE.Vector3;
    // Cap and legend meshes, moved together for key travel; userData.z0 is the rest height (mm).
    parts: THREE.Object3D[];
  };
  type Stage = {
    renderer: THREE.WebGLRenderer;
    scene: THREE.Scene;
    camera: THREE.PerspectiveCamera;
    caps: Map<number, Cap>;
    // Pointer picking is a raycast, so at low angles a click lands on the cap actually in front.
    keyOf: Map<THREE.Object3D, number>;
    glows: Set<THREE.Object3D>;
    // Bloom pass for lit LEDs; plain renderer.render() when everything is dark.
    composer: EffectComposer;
  };

  const TAPER = Math.tan((13 * Math.PI) / 180);
  const FOV = 24;
  const HOME = { az: 0, el: 1 };
  // Shared by every mount, so the angle survives tab switches.
  let view = { ...HOME };
  // Theme thumbnails, rendered once per session on first open of the picker.
  const thumbCache = new Map<string, string>();

  // The node or group name a theme colour is keyed by (per-key parts share their row group's name).
  const groupOf = (o: THREE.Object3D): string => o.userData.group ?? o.name;

  function glowTexture() {
    const c = document.createElement('canvas');
    c.width = c.height = 64;
    const g = c.getContext('2d')!;
    // Flat core (the LED pool under the cap) with a soft edge that spills into the gaps.
    const grad = g.createRadialGradient(32, 32, 0, 32, 32, 32);
    grad.addColorStop(0, '#fff');
    grad.addColorStop(0.6, '#fff');
    grad.addColorStop(1, 'rgba(255,255,255,0)');
    g.fillStyle = grad;
    g.fillRect(0, 0, 64, 64);
    return new THREE.CanvasTexture(c);
  }

  // f75.glb comes split per key (docs/keyboard-model/build_glb.py): every cap and legend part carries
  // extras {group, key}. Each gets its own material, so it can take its own LED colour.
  function indexKeys(root: THREE.Object3D, s: Stage) {
    const tex = glowTexture();
    const parts: THREE.Mesh[] = [];
    const legends = new Map<number, THREE.MeshStandardMaterial[]>();
    const partsOf = new Map<number, THREE.Object3D[]>();
    root.traverse((o) => {
      if (o instanceof THREE.Mesh && o.userData.key !== undefined) parts.push(o);
    });
    for (const part of parts) {
      const m: number = part.userData.key;
      const mat = (part.material as THREE.MeshStandardMaterial).clone();
      part.material = mat;
      s.keyOf.set(part, m);
      part.userData.z0 = part.position.z;
      if (!partsOf.has(m)) partsOf.set(m, []);
      partsOf.get(m)!.push(part);
      if (!groupOf(part).startsWith('Caps')) {
        if (!legends.has(m)) legends.set(m, []);
        legends.get(m)!.push(mat);
        continue;
      }

      // Box in the model's mm space: the part's own transform undoes the mesh quantization.
      const parent = part.parent!;
      if (!part.geometry.boundingBox) part.geometry.computeBoundingBox();
      const { min, max } = part.geometry.boundingBox!.clone().applyMatrix4(part.matrix);
      const glow = new THREE.Mesh(
        new THREE.PlaneGeometry(max.x - min.x + 8, max.y - min.y + 8),
        new THREE.MeshBasicMaterial({ map: tex, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, toneMapped: false }),
      );
      // Just above the plate, hidden under the cap: only the light leaking into the gaps shows.
      glow.position.set((min.x + max.x) / 2, (min.y + max.y) / 2, min.z + 0.2);
      glow.visible = false;
      parent.add(glow);
      s.glows.add(glow);

      const inset = (max.z - min.z) * TAPER;
      const corners: THREE.Vector3[] = [];
      for (const [x, y] of [[min.x, min.y], [max.x, min.y], [max.x, max.y], [min.x, max.y]]) {
        corners.push(parent.localToWorld(new THREE.Vector3(x, y, min.z)));
        const ix = x === min.x ? inset : -inset;
        const iy = y === min.y ? inset : -inset;
        corners.push(parent.localToWorld(new THREE.Vector3(x + ix, y + iy, max.z)));
      }
      // Remap dot: on the top face, back-right corner (legends sit top-left).
      const mark = parent.localToWorld(new THREE.Vector3(max.x - inset - 3.2, max.y - inset - 3.2, max.z));
      s.caps.set(m, { legends: [], glow, corners, mark, parts: [] });
    }
    for (const [m, mats] of legends) s.caps.get(m)?.legends.push(...mats);
    for (const [m, list] of partsOf) s.caps.get(m)?.parts.push(...list);
  }

  // One WebGL context and one model for the whole session: tab switches only move the canvas.
  async function build(): Promise<Stage> {
    // Fetch first: the download runs while the renderer and environment are set up.
    const model = new GLTFLoader().setMeshoptDecoder(MeshoptDecoder).loadAsync(modelUrl);
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(devicePixelRatio);
    renderer.toneMapping = THREE.NeutralToneMapping;
    const scene = new THREE.Scene();
    const pmrem = new THREE.PMREMGenerator(renderer);
    const room = new RoomEnvironment();
    scene.environment = pmrem.fromScene(room, 0.04).texture;
    // The generator keeps its own targets and filter materials; only the returned environment is needed.
    pmrem.dispose();
    room.dispose();
    scene.environmentIntensity = 0.45;
    const sun = new THREE.DirectionalLight(0xffffff, 1.6);
    sun.position.set(-0.3, 0.6, 0.4);
    scene.add(sun);
    const camera = new THREE.PerspectiveCamera(FOV, 2, 0.02, 3);
    const composer = new EffectComposer(renderer);
    composer.addPass(new RenderPass(scene, camera));
    // Only what is brighter than the lit caps blooms: glow planes and shine-through legends.
    composer.addPass(new UnrealBloomPass(new THREE.Vector2(256, 128), 0.55, 0.35, 0.82));
    composer.addPass(new OutputPass());
    const s: Stage = { renderer, scene, camera, caps: new Map(), keyOf: new Map(), glows: new Set(), composer };
    const gltf = await model;
    scene.add(gltf.scene);
    gltf.scene.updateMatrixWorld(true);
    indexKeys(gltf.scene, s);
    // Compile shaders now (in parallel where the driver allows) instead of stalling the first visible frame.
    await renderer.compileAsync(scene, s.camera);
    return s;
  }

  // Started when the app imports this component, in parallel with the first HID reads.
  const stagePromise = build();
  stagePromise.catch(() => {});
  // The canvas fades in once per session; later tab switches only move it.
  let canvasShown = false;
</script>

<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import type { Key } from './api';
  import { THEMES, type Theme } from './f75themes';
  import type { Fx } from './fx';
  import { HID_KEYS } from './hid';
  import { Spring } from 'svelte/motion';
  import { slide } from 'svelte/transition';
  import { dur } from './motion';

  let {
    keys,
    labelOf,
    tint,
    fx,
    selected = new Set<number>(),
    onpick,
  }: {
    keys: Key[];
    // Tooltip text; a second line marks the key as remapped.
    labelOf: (k: Key) => string;
    // LED colour as #rrggbb; #000000 is off.
    tint?: (matrix: number) => string;
    // Animated effect simulation; overrides tint. Keys pressed on the real keyboard or clicked feed it.
    fx?: Fx;
    selected?: Set<number>;
    onpick?: (matrix: number, shift: boolean) => void;
  } = $props();

  // dot: the remap mark is not hidden behind another part.
  type Shape = { pts: string; mx: number; my: number; top: number; dot: boolean };

  const THEME_KEY = 'openf75.theme';

  let host: HTMLDivElement;
  let trigger: HTMLButtonElement;
  let w = $state(0);
  let h = $state(0);
  let stage = $state.raw<Stage | null>(null);
  const ready = $derived(stage !== null);
  let failed = $state(false);
  let shapes = $state<Record<number, Shape>>({});
  let themeName = $state(localStorage.getItem(THEME_KEY) ?? 'Black');
  const theme = $derived(THEMES.find((t) => t.name === themeName) ?? THEMES[0]);
  const ray = new THREE.Raycaster();
  let hover = $state(-1);
  let picker = $state(false);
  let thumbs = $state<Record<string, string>>(Object.fromEntries(thumbCache));
  let raf = 0;
  let viewDirty = true;
  let ledsOn = false;
  const led = new THREE.Color();
  const reduceMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Key travel: keys held down, and each key's current depth 0..1 (eased toward held ? 1 : 0).
  const TRAVEL_MM = 3.2;
  const held = new Set<number>();
  const depth = new Map<number, number>();
  let lastFrame = 0;
  // Camera motion: a glide back to HOME, and the spin left over from a flung orbit (rad/s).
  const GLIDE_S = 0.45;
  let glide: { from: { az: number; el: number }; t: number } | null = null;
  let spin = { az: 0, el: 0 };
  let lastMove = 0;
  const clampEl = (el: number) => Math.max(0.08, Math.min(Math.PI / 2 - 0.01, el));
  // A new effect fades in over the previous one (or from dark) instead of cutting.
  const XFADE_S = 0.3;
  let shownFx: Fx | undefined;
  let prevFx: Fx | undefined;
  let fxSince = -Infinity;
  // Set when the effect changes; cleared only after the frame that paints the fully faded-in colours.
  let fading = false;
  // Physical typing reaches the preview through KeyboardEvent.code -> HID usage -> matrix.
  const matrixOfCode = $derived.by(() => {
    const byUsage = new Map(keys.filter((k) => k.default_usage !== null).map((k) => [k.default_usage!, k.matrix]));
    return new Map(HID_KEYS.filter((k) => byUsage.has(k.usage)).map((k) => [k.code, byUsage.get(k.usage)!]));
  });
  // With no input for a while, reactive effects get ghost keystrokes so the preview never sits dark.
  let lastInput = -Infinity;
  let nextGhost = 0;
  const ghostUp = new Map<number, number>();

  const visible = $derived(keys.filter((k) => shapes[k.matrix]));
  // Convex hull (monotone chain) of the projected cap corners: the clickable outline of the cap.
  function hull(pts: [number, number][]): [number, number][] {
    pts.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    const cross = (o: number[], a: number[], b: number[]) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    const half = (list: [number, number][]) => {
      const out: [number, number][] = [];
      for (const p of list) {
        while (out.length >= 2 && cross(out[out.length - 2], out[out.length - 1], p) <= 0) out.pop();
        out.push(p);
      }
      out.pop();
      return out;
    };
    return [...half(pts), ...half([...pts].reverse())];
  }

  function toScreen(p: THREE.Vector3, v: THREE.Vector3): [number, number] {
    v.copy(p).project(stage!.camera);
    return [((v.x + 1) / 2) * w, ((1 - v.y) / 2) * h];
  }

  // Nearest visible solid along the ray (theme-hidden legends and LED glows don't block).
  function firstHit() {
    return ray.intersectObject(stage!.scene, true).find((i) => i.object.visible && !stage!.glows.has(i.object));
  }

  function project() {
    const { caps, camera, keyOf } = stage!;
    const next: Record<number, Shape> = {};
    const v = new THREE.Vector3();
    for (const [m, cap] of caps) {
      const [mx, my] = toScreen(cap.mark, v);
      const pts = hull(cap.corners.map((c) => toScreen(c, v)));
      const depth = camera.position.distanceTo(cap.mark);
      ray.set(camera.position, v.copy(cap.mark).sub(camera.position).normalize());
      const hit = firstHit();
      const dot = !hit || keyOf.get(hit.object) === m || hit.distance > depth - 0.0005;
      next[m] = { pts: pts.map((p) => p.join(',')).join(' '), mx, my, top: Math.min(...pts.map((p) => p[1])), dot };
    }
    shapes = next;
  }

  function keyAt(e: PointerEvent): number {
    const r = host.getBoundingClientRect();
    ray.setFromCamera(new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, 1 - ((e.clientY - r.top) / r.height) * 2), stage!.camera);
    const hit = firstHit();
    return (hit && stage!.keyOf.get(hit.object)) ?? -1;
  }

  function placeCamera(aspect = w / h) {
    const camera = stage!.camera;
    camera.aspect = aspect;
    const tanV = Math.tan(((FOV / 2) * Math.PI) / 180) / 1.08;
    const tanH = tanV * aspect;
    const ca = Math.cos(view.az), sa = Math.sin(view.az), ce = Math.cos(view.el), se = Math.sin(view.el);
    // Closest distance at which every corner of the case (326 x 34 x 142 mm, world Y up) stays in frame:
    // a corner at depth z toward the camera and offset x needs dist >= z + |x| / tan(half fov).
    let dist = 0;
    for (const x of [-0.163, 0.163])
      for (const y of [-0.013, 0.021])
        for (const z of [-0.071, 0.071]) {
          const depth = x * sa * ce + y * se + z * ca * ce;
          const right = x * ca - z * sa;
          const up = -x * sa * se + y * ce - z * ca * se;
          dist = Math.max(dist, depth + Math.abs(right) / tanH, depth + Math.abs(up) / tanV);
        }
    camera.position.set(dist * sa * ce, 0.005 + dist * se, dist * ca * ce);
    camera.lookAt(0, 0.005, 0);
    camera.updateProjectionMatrix();
  }

  let dead = false;
  function invalidate(moved = false) {
    viewDirty ||= moved;
    if (!raf && !dead) raf = requestAnimationFrame(frame);
  }

  function frame(now = performance.now()) {
    // Direct calls (thumbnails) must not leave a second pending frame behind.
    cancelAnimationFrame(raf);
    raf = 0;
    if (!stage || !w || !h || dead) return;
    const t = now / 1000;
    const dt = Math.min(0.1, t - lastFrame);
    lastFrame = t;
    // Static effects (Off, Fixed on, Self-define) draw once; reactive ones keep running for ghost presses.
    const live = fx && !reduceMotion && (fx.animated || fx.reactive);
    if (fading && (!fx || reduceMotion)) fading = false;
    if (live || fading) {
      if (live && fx!.reactive) ghostType(t);
      applyLeds(true, t);
    }
    // The frame that crosses XFADE_S has just painted k = 1: static effects (Off, Fixed on) stop here.
    const fadeNext = fading && t - fxSince < XFADE_S;
    fading = fadeNext;
    const moving = stepTravel(dt);
    const turning = stepCamera(dt, t);
    placeCamera();
    if (ledsOn) stage.composer.render();
    else stage.renderer.render(stage.scene, stage.camera);
    if (viewDirty) {
      viewDirty = false;
      project();
    }
    // pressKey() inside this frame may already have scheduled the next one: never run two loops.
    if ((live || fadeNext || moving || turning) && !raf && !dead) raf = requestAnimationFrame(frame);
  }

  // Glide toward HOME, else coast on the fling spin; true while the view is still changing.
  function stepCamera(dt: number, t: number): boolean {
    if (glide) {
      const k = Math.min(1, (t - glide.t) / GLIDE_S);
      const e = 1 - (1 - k) ** 3;
      view.az = glide.from.az + (HOME.az - glide.from.az) * e;
      view.el = glide.from.el + (HOME.el - glide.from.el) * e;
      if (k === 1) glide = null;
      viewDirty = true;
      return glide !== null;
    }
    if (orbit || (!spin.az && !spin.el)) return false;
    view.az += spin.az * dt;
    view.el = clampEl(view.el + spin.el * dt);
    const decay = Math.exp(-dt * 5);
    spin = { az: spin.az * decay, el: spin.el * decay };
    if (Math.hypot(spin.az, spin.el) < 0.03) spin = { az: 0, el: 0 };
    viewDirty = true;
    return true;
  }

  function pressKey(m: number, t = performance.now() / 1000) {
    if (!fx || reduceMotion) return;
    fx.press(m, t);
    held.add(m);
    invalidate();
  }

  function ghostType(t: number) {
    for (const [m, up] of ghostUp) if (t >= up) (held.delete(m), ghostUp.delete(m));
    if (t - lastInput < 2.5 || t < nextGhost) return;
    const all = [...stage!.caps.keys()];
    const m = all[Math.floor(Math.random() * all.length)];
    pressKey(m, t);
    ghostUp.set(m, t + 0.09);
    nextGhost = t + 0.15 + Math.random() * 0.35;
  }

  // Eases every key toward its target depth; true while anything is still moving.
  function stepTravel(dt: number): boolean {
    let moving = false;
    for (const m of held) if (!depth.has(m)) depth.set(m, 0);
    for (const [m, d] of depth) {
      const target = held.has(m) ? 1 : 0;
      const next = Math.abs(target - d) < 0.01 ? target : d + (target - d) * Math.min(1, dt * 28);
      for (const p of stage!.caps.get(m)?.parts ?? []) p.position.z = p.userData.z0 - next * TRAVEL_MM;
      if (next === 0 && !held.has(m)) depth.delete(m);
      else depth.set(m, next);
      moving ||= next !== target;
    }
    return moving;
  }

  function realKey(e: KeyboardEvent, down: boolean) {
    if (!fx || e.repeat) return;
    const el = e.target as HTMLElement;
    if (el.closest('input, textarea, select, [contenteditable]')) return;
    const m = matrixOfCode.get(e.code);
    if (m === undefined) return;
    lastInput = performance.now() / 1000;
    if (down) pressKey(m, lastInput);
    else (held.delete(m), invalidate());
  }

  onMount(() => {
    let alive = true;
    let ro: ResizeObserver | undefined;
    stagePromise.then(
      (s) => {
        if (!alive) return;
        const canvas = s.renderer.domElement;
        host.prepend(canvas);
        if (!canvasShown) {
          canvasShown = true;
          canvas.animate([{ opacity: 0 }, { opacity: 1 }], { duration: dur(320), easing: 'ease-out' });
        }
        ro = new ResizeObserver(([e]) => {
          w = e.contentRect.width;
          h = e.contentRect.height;
          s.renderer.setSize(w, h, false);
          s.composer.setPixelRatio(devicePixelRatio);
          s.composer.setSize(w, h);
          invalidate(true);
        });
        ro.observe(host);
        stage = s;
      },
      () => (failed = true),
    );
    return () => {
      alive = false;
      dead = true;
      cancelAnimationFrame(raf);
      ro?.disconnect();
      // The scene is shared with the next mount: put pressed keys back up.
      for (const m of depth.keys()) for (const p of stage?.caps.get(m)?.parts ?? []) p.position.z = p.userData.z0;
      // The next tab's keyboard may already own the canvas.
      const canvas = stage?.renderer.domElement;
      if (canvas?.parentElement === host) canvas.remove();
    };
  });

  function applyTheme(t: Theme) {
    stage!.scene.traverse((o) => {
      if (!(o instanceof THREE.Mesh)) return;
      const group = groupOf(o);
      const hex = t.colors[group];
      if (hex) (o.material as THREE.MeshStandardMaterial).color.set(hex);
      if (group.startsWith('LegendSide')) o.visible = t.side;
      else if (group.startsWith('Legend')) o.visible = !t.side;
    });
  }

  // LEDs: light leaks around the cap and tints it; shine-through legends glow in the LED colour.
  function applyLeds(lit: boolean, t = performance.now() / 1000) {
    const shine = theme.shine;
    let any = false;
    for (const [m, cap] of stage!.caps) {
      if (!lit) led.setRGB(0, 0, 0);
      else if (fx) {
        const tt = reduceMotion ? 1.5 : t;
        const c = fx.color(m, tt);
        // rAF timestamps can predate fxSince by a frame: clamp so the mix never extrapolates.
        const k = reduceMotion ? 1 : Math.max(0, Math.min(1, (t - fxSince) / XFADE_S));
        if (k < 1) {
          const p = prevFx ? prevFx.color(m, tt) : ([0, 0, 0] as const);
          led.setRGB(p[0] + (c[0] - p[0]) * k, p[1] + (c[1] - p[1]) * k, p[2] + (c[2] - p[2]) * k, THREE.SRGBColorSpace);
        } else led.setRGB(...c, THREE.SRGBColorSpace);
      }
      else led.set(tint?.(m) ?? '#000000');
      const on = led.r + led.g + led.b > 1e-4;
      any ||= on;
      cap.glow.visible = on;
      // The LEDs sit under the caps (opaque ABS): light only shows on the plate, through the gaps, and
      // through shine-through legends. The caps themselves are not tinted.
      cap.glow.material.color.copy(led).multiplyScalar(2.6);
      for (const l of cap.legends) {
        if (shine) l.emissive.copy(led);
        else l.emissive.setRGB(0, 0, 0);
      }
    }
    ledsOn = any;
  }

  // Theme: body colours from the f75.sc configuration row; side-printed themes swap top legends for front ones.
  $effect(() => {
    if (!ready) return;
    localStorage.setItem(THEME_KEY, theme.name);
    applyTheme(theme);
    invalidate();
  });

  // Runs before the LED effect below so the crossfade starts on the same paint.
  $effect(() => {
    const f = fx;
    untrack(() => {
      if (f === shownFx) return;
      prevFx = shownFx;
      shownFx = f;
      fxSince = performance.now() / 1000;
      fading = true;
    });
  });

  $effect(() => {
    if (!ready) return;
    applyLeds(true);
    invalidate();
  });

  // Thumbnails right after the model is up, so the "Modelo" button shows its preview before the first open.
  $effect(() => {
    if (!ready) return;
    const id = setTimeout(makeThumbs, 400);
    return () => clearTimeout(id);
  });

  // Renders every theme, LEDs off, from the home view into the same canvas, then restores it within one task.
  function makeThumbs() {
    if (!stage) return;
    const { renderer, scene, camera } = stage;
    if (thumbCache.size < THEMES.length) {
      const saved = view;
      const ratio = renderer.getPixelRatio();
      view = { ...HOME };
      renderer.setPixelRatio(1);
      renderer.setSize(320, 160, false);
      placeCamera(2);
      applyLeds(false);
      for (const t of THEMES) {
        applyTheme(t);
        renderer.render(scene, camera);
        thumbCache.set(t.name, renderer.domElement.toDataURL('image/webp', 0.85));
      }
      view = saved;
      renderer.setPixelRatio(ratio);
      renderer.setSize(w, h, false);
      applyTheme(theme);
      applyLeds(true);
      frame();
    }
    thumbs = Object.fromEntries(thumbCache);
  }

  function togglePicker() {
    picker = !picker;
    if (picker) makeThumbs();
  }

  function choose(name: string) {
    themeName = name;
    picker = false;
    // The chosen card is removed with the grid: keep keyboard focus on the trigger.
    trigger.focus();
  }

  // Left on a key picks (drag paints); left off the keys, or right anywhere, orbits.
  let orbit = $state<{ x: number; y: number } | null>(null);
  let painting = false;
  function hostDown(e: PointerEvent) {
    if ((e.target as Element).closest('.tools') || !ready) return;
    const m = e.button === 0 ? keyAt(e) : -1;
    if (m >= 0) {
      painting = true;
      clicked = m;
      lastInput = performance.now() / 1000;
      pressKey(m, lastInput);
      onpick?.(m, e.shiftKey);
    } else if (e.button === 0 || e.button === 2) {
      orbit = { x: e.clientX, y: e.clientY };
      glide = null;
      spin = { az: 0, el: 0 };
      lastMove = performance.now() / 1000;
    }
  }
  let clicked = -1;
  function pointerUp() {
    painting = false;
    // A drag that stopped before release doesn't coast.
    if (orbit && (reduceMotion || performance.now() / 1000 - lastMove > 0.08)) spin = { az: 0, el: 0 };
    const flung = !!orbit && (spin.az !== 0 || spin.el !== 0);
    orbit = null;
    if (clicked >= 0) (held.delete(clicked), invalidate());
    clicked = -1;
    if (flung) invalidate(true);
  }
  function hostMove(e: PointerEvent) {
    if (orbit) {
      const daz = -(e.clientX - orbit.x) * 0.006;
      const del = (e.clientY - orbit.y) * 0.005;
      view.az += daz;
      view.el = clampEl(view.el + del);
      orbit = { x: e.clientX, y: e.clientY };
      // Smoothed drag velocity, capped, released as spin on pointer up.
      const now = performance.now() / 1000;
      const step = Math.max(now - lastMove, 1 / 240);
      lastMove = now;
      const cap = (v: number) => Math.max(-8, Math.min(8, v));
      spin = { az: cap(spin.az * 0.5 + (daz / step) * 0.5), el: cap(spin.el * 0.5 + (del / step) * 0.5) };
      invalidate(true);
      return;
    }
    const m = ready && host.contains(e.target as Node) ? keyAt(e) : -1;
    if (m === hover) return;
    hover = m;
    if (painting && m >= 0) onpick?.(m, false);
  }
  function resetView() {
    spin = { az: 0, el: 0 };
    // Full-azimuth orbits can leave az several turns away: glide the short way round.
    const az = Math.atan2(Math.sin(view.az), Math.cos(view.az));
    if (reduceMotion) {
      glide = null;
      view = { ...HOME };
    } else {
      view = { az, el: view.el };
      glide = { from: { ...view }, t: performance.now() / 1000 };
    }
    invalidate(true);
  }

  // One tooltip glides between keys: the Spring moves it, CSS fades it in and out.
  const tipAt = new Spring({ x: 0, y: 0 }, { stiffness: 0.25, damping: 0.85 });
  let tipLabel = $state('');
  let tipOn = $state(false);
  let tipWasOn = false;
  $effect(() => {
    const k = keys.find((k) => k.matrix === hover);
    const s = shapes[hover];
    const on = !!k && !!s && !orbit;
    if (k) tipLabel = labelOf(k).split('\n').join(': ');
    tipOn = on;
    // Snap when showing from nothing so it doesn't fly in from the last key; reduced motion never animates.
    if (on) untrack(() => tipAt.set({ x: s.mx, y: s.top }, { instant: !tipWasOn || dur(1) === 0 }));
    tipWasOn = on;
  });
</script>

<svelte:window
  onpointerup={pointerUp}
  onpointermove={hostMove}
  onkeydown={(e) => realKey(e, true)}
  onkeyup={(e) => realKey(e, false)}
/>

<div class="wrap">
  <div
    class="kb"
    class:orbiting={!!orbit}
    class:over-key={hover >= 0}
    bind:this={host}
    role="group"
    aria-label="Teclado"
    onpointerdown={hostDown}
    onpointerleave={() => (hover = -1)}
    oncontextmenu={(e) => e.preventDefault()}
    ondblclick={() => hover < 0 && resetView()}
  >
    {#if failed}
      <p class="hint state">Não foi possível exibir o modelo 3D.</p>
    {:else if !ready}
      <p class="hint state">Carregando modelo 3D…</p>
    {/if}
    {#if fx && ready}
      <p class="live" aria-live="polite">
        <span class="pulse"></span>
        {reduceMotion ? 'Simulação (movimento reduzido)' : 'Simulação ao vivo'}
        {#if fx.reactive && !reduceMotion}<span class="sub">· digite no seu teclado</span>{/if}
      </p>
    {/if}
    <svg viewBox="0 0 {w} {h}">
      <!-- Outlines only; pointer input goes through keyAt(). The groups stay focusable for the keyboard. -->
      <!-- shortcut: 80 tab stops, upgrade to roving tabindex + arrow keys if keyboard navigation matters -->
      {#each visible as k (k.matrix)}
        {@const s = shapes[k.matrix]}
        {@const lines = labelOf(k).split('\n')}
        <g
          class="key"
          class:hover={hover === k.matrix}
          role="button"
          tabindex="0"
          aria-label={k.name}
          aria-pressed={selected.has(k.matrix)}
          onkeydown={(e) => {
            if (e.key !== 'Enter' && e.key !== ' ') return;
            e.preventDefault();
            onpick?.(k.matrix, e.shiftKey);
          }}
        >
          <polygon class="hit" points={s.pts} />
          <polygon class="ring ring-dark" points={s.pts} />
          <polygon class="ring ring-light" points={s.pts} />
          {#if lines.length > 1 && s.dot}<circle class="mark" cx={s.mx} cy={s.my} r="3.5" />{/if}
          <polygon class="focus" points={s.pts} />
        </g>
      {/each}
    </svg>
    <div
      class="tip-anchor"
      aria-hidden="true"
      style:transform="translate3d({tipAt.current.x}px, {tipAt.current.y}px, 0)"
    >
      <span class="tip" class:on={tipOn}>{tipLabel}</span>
    </div>
  </div>
  <div class="tools">
    <span class="label">Modelo</span>
    <button class="current" bind:this={trigger} aria-expanded={picker} disabled={!ready} onclick={togglePicker}>
      {#if thumbs[theme.name]}<img src={thumbs[theme.name]} alt="" />{/if}
      {theme.name}
    </button>
    <span class="hint">Arraste fora das teclas para girar</span>
    <button class="ghost" onclick={resetView}>Vista inicial</button>
  </div>
  {#if picker}
    <div class="themes" role="group" aria-label="Modelo do teclado" transition:slide={{ duration: dur(200) }}>
      {#each THEMES as t (t.name)}
        <button aria-pressed={t.name === themeName} onclick={() => choose(t.name)}>
          {#if thumbs[t.name]}<img src={thumbs[t.name]} alt="" />{/if}
          <span>{t.name}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .wrap {
    width: 100%;
    max-width: 880px;
    margin-inline: auto;
  }
  .kb {
    position: relative;
    aspect-ratio: 2 / 1;
    user-select: none;
    touch-action: none;
    cursor: grab;
  }
  .kb.orbiting {
    cursor: grabbing;
  }
  /* The canvas is the session-wide renderer's, inserted on mount (see build()). */
  .kb :global(canvas),
  svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .kb :global(canvas) {
    pointer-events: none;
    filter: drop-shadow(0 18px 24px var(--lift));
  }
  .current {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .current img {
    width: 48px;
    height: 24px;
    object-fit: cover;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 8px;
    margin-top: 12px;
  }
  .themes button {
    display: grid;
    gap: 4px;
    justify-items: center;
    padding: 6px 6px 8px;
    font-size: 12px;
    text-align: center;
  }
  .themes img {
    width: 100%;
    aspect-ratio: 2 / 1;
  }
  svg {
    overflow: visible;
  }
  .live {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--raised) 80%, transparent);
    font-size: 12px;
    pointer-events: none;
  }
  .live .sub {
    color: var(--muted);
  }
  .pulse {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .pulse {
      animation: none;
    }
  }
  .state {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }
  .tools .hint {
    margin-left: auto;
  }
  .kb.over-key {
    cursor: pointer;
  }
  svg {
    pointer-events: none;
  }
  .key {
    outline: none;
  }
  .hit {
    fill: var(--text);
    fill-opacity: 0;
  }
  .key.hover .hit {
    fill-opacity: 0.14;
  }
  .key[aria-pressed='true'] .hit {
    fill: var(--accent);
    fill-opacity: 0.25;
  }
  .hit,
  .ring,
  .focus {
    stroke-linejoin: round;
  }
  .ring,
  .focus {
    fill: none;
    opacity: 0;
    pointer-events: none;
  }
  .ring-dark {
    stroke: var(--rail);
    stroke-width: 5px;
  }
  .ring-light {
    stroke: var(--accent);
    stroke-width: 2px;
  }
  .key[aria-pressed='true'] .ring {
    opacity: 1;
  }
  .mark {
    fill: var(--accent);
    stroke: var(--rail);
    stroke-width: 1.5;
    pointer-events: none;
  }
  .focus {
    stroke: var(--accent);
    stroke-width: 2px;
  }
  .key:focus-visible .focus {
    opacity: 1;
  }

  .tip-anchor {
    position: absolute;
    left: 0;
    top: 0;
    pointer-events: none;
  }
  .tip {
    position: absolute;
    bottom: 8px;
    left: 0;
    translate: -50% 0;
    padding: 3px 8px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
    color: var(--text);
    font-size: 12px;
    white-space: nowrap;
    opacity: 0;
    transform: scale(0.96);
    transform-origin: bottom center;
    transition:
      opacity var(--dur-quick) var(--ease-out),
      transform var(--dur-quick) var(--ease-out);
  }
  .tip.on {
    opacity: 1;
    transform: scale(1);
  }
</style>
