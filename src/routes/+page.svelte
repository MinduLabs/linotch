<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { scale } from "svelte/transition";
  import Eq from "$lib/Eq.svelte";
  import Cover from "$lib/Cover.svelte";
  import { cubicOut } from "svelte/easing";
  import { island, setHitRect, fmt } from "$lib/island.svelte";
  import Alert from "$lib/Alert.svelte";
  import BtCard from "$lib/BtCard.svelte";
  import Expanded from "$lib/Expanded.svelte";
  import { Spring, OPEN, CLOSE } from "$lib/spring";


  const IN = { start: 0.9, duration: 320, delay: 60, easing: cubicOut };
  const OUT = { start: 0.94, duration: 120, easing: cubicOut };

  // The morph runs on the compositor only: every frame changes nothing but transforms (no layout, no paint).
  // The black shape is put together from pieces, the content sits in a clip box scaled to the shape plus an
  // inner box scaled back by the inverse, so text stays 1:1 and never reflows while the pill moves.
  const BODY_W = 400; // base sizes; shapes only ever scale down from these, so edges stay sharp
  const BODY_H = 240; // tallest shape is 190 (expanded) plus spring overshoot
  const SIDE_W = 48;
  const CORNER = 48; // corner boxes are fixed; only their gradient radius changes (a tiny repaint)
  const FLARE = 10;
  const OVERLAP = 0.5; // pieces overlap by half a pixel so no seam shows between them
  let pillEl: HTMLElement;
  let unclipEl: HTMLElement;
  let bodyEl: HTMLElement, sideL: HTMLElement, sideR: HTMLElement;
  let cornerL: HTMLElement, cornerR: HTMLElement, flareL: HTMLElement, flareR: HTMLElement;
  const sw = new Spring(island.size.w);
  const sh = new Spring(island.size.h);
  const sr = new Spring(island.size.r);
  let raf = 0;
  let last = 0;
  let cfg = OPEN;
  let token = 0;

  const place = (el: HTMLElement, x: number, y: number, sx = 1, sy = 1) =>
    (el.style.transform = `translate(${x}px, ${y}px) scale(${sx}, ${sy})`);

  const apply = () => {
    const W = Math.max(1, sw.x);
    const H = Math.max(1, sh.x);
    const R = Math.max(0, Math.min(sr.x, H, W / 2, CORNER));
    const half = W / 2;
    place(bodyEl, -half + R - OVERLAP, 0, (W - 2 * R + 2 * OVERLAP) / BODY_W, H / BODY_H);
    const sideSy = Math.max(0, H - R + OVERLAP) / BODY_H;
    place(sideL, -half, 0, (R + OVERLAP) / SIDE_W, sideSy);
    place(sideR, half - R - OVERLAP, 0, (R + OVERLAP) / SIDE_W, sideSy);
    place(cornerL, -half, H - R);
    place(cornerR, half - CORNER, H - R);
    cornerL.style.setProperty("--r", `${R}px`);
    cornerR.style.setProperty("--r", `${R}px`);
    place(flareL, -half - FLARE, 0);
    place(flareR, half, 0);
    pillEl.style.transform = `scale(${W / BODY_W}, ${H / BODY_H})`;
    unclipEl.style.transform = `scale(${BODY_W / W}, ${BODY_H / H})`;
  };

  function tick(t: number) {
    const dt = Math.min((t - last) / 1000, 0.1);
    last = t;
    for (const s of [sw, sh, sr]) s.step(dt, cfg);
    apply();
    if (sw.settled && sh.settled && sr.settled) {
      for (const s of [sw, sh, sr]) s.snap();
      apply();
      raf = 0;
      const rest = { w: sw.target, h: sh.target, r: sr.target };
      setHitRect(rest); // shrink the hit area once at rest
      island.recheckHover(rest);
      return;
    }
    raf = requestAnimationFrame(tick);
  }

  $effect(() => {
    const s = island.size;
    const me = ++token;
    const grow = s.w * s.h >= sw.target * sh.target;
    cfg = grow ? OPEN : CLOSE;
    sw.target = s.w;
    sh.target = s.h;
    sr.target = s.r;
    if (grow) setHitRect(s); // widen the hit area at once; narrow it only after the collapse
    if (me !== token || raf) return;
    last = performance.now();
    raf = requestAnimationFrame(tick);
  });

  onMount(() => {
    island.init();
    apply();
    setHitRect(island.size);
  });


  /**
   * Layers keep the size of the shape they belong to instead of following the pill frame by frame, so their
   * content never reflows while the pill morphs; the pill just clips them. An outgoing layer stops updating,
   * so it keeps its old size while it fades.
   */
  function fit(node: HTMLElement, size: { w: number; h: number }) {
    const set = ({ w, h }: { w: number; h: number }) => {
      node.style.width = `${w}px`;
      node.style.height = `${h}px`;
      node.style.marginLeft = `${-w / 2}px`;
    };
    set(size);
    return { update: set };
  }

</script>

<div class="stage">
  <div class="shape" aria-hidden="true">
    <i class="solid body" bind:this={bodyEl}></i>
    <i class="solid side" bind:this={sideL}></i>
    <i class="solid side" bind:this={sideR}></i>
    <i class="corner l" bind:this={cornerL}></i>
    <i class="corner r" bind:this={cornerR}></i>
    <i class="flare l" bind:this={flareL}></i>
    <i class="flare r" bind:this={flareR}></i>
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="pill"
    class:alert={island.mode === "alert"}
    bind:this={pillEl}
    onmouseenter={(e) => {
      island.pointerAt(e.clientX, e.clientY);
      island.enter();
    }}
    onmousemove={(e) => island.pointerAt(e.clientX, e.clientY)}
    onmouseleave={() => island.leave()}
    onclick={() => island.alert && island.openAlert()}
  >
    <div class="unclip" bind:this={unclipEl}>
    <!-- ALERT -->
    {#if island.mode === "alert" && island.alert}
      {@const a = island.alert}
      {#key a.kind === "bluetooth" ? `bt:${a.device.name}:${a.connected}` : a.kind}
        <div class="layer" use:fit={island.size} in:scale={IN} out:scale={OUT}>
          {#if a.kind === "bluetooth"}
            <BtCard device={a.device} connected={a.connected} />
          {:else}
            <Alert alert={a} />
          {/if}
        </div>
      {/key}

    <!-- EXPANDED -->
    {:else if island.mode === "expanded"}
      <div class="layer" use:fit={island.size} in:scale={IN} out:scale={OUT}>
        <Expanded />
      </div>

    <!-- COMPACT -->
    {:else if island.mode === "compact"}
      <div class="layer row split" use:fit={island.size} in:scale={IN} out:scale={OUT}>
        {#if island.media}
          <Cover src={island.media.art} small />
        {:else}
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="13" r="8" fill="none" stroke="currentColor" stroke-width="2" /><path d="M12 9v4l3 2" stroke="currentColor" stroke-width="2" fill="none" /></svg>
        {/if}
        {#if island.timerEnd}
          <b class="orange">{fmt(island.timerLeft)}</b>
        {:else if island.micApp && !island.media}
          <span class="dot" title="{island.micApp} is using the microphone"></span>
        {:else}
          <Eq on={island.media?.playing} />
        {/if}
      </div>
    {/if}
    </div>
  </div>
</div>

<style>
  .stage { position: fixed; top: 0; left: 50%; width: 0; height: 0; }
  .shape > i { position: absolute; top: 0; left: 0; transform-origin: 0 0; will-change: transform; }
  /* Plain colour boxes: WebKit composites them as solid-colour layers, nothing to rasterise. */
  .solid { background: #000; }
  .body { width: 400px; height: 240px; }
  .side { width: 48px; height: 240px; }
  /* Bottom corners: a quarter circle of radius --r, drawn in a fixed 48px box. */
  .corner { --r: 10px; width: 48px; height: 48px; }
  .corner.l { background: radial-gradient(circle at var(--r) 0, #000 calc(var(--r) - 0.25px), transparent calc(var(--r) + 0.5px)); }
  .corner.r { background: radial-gradient(circle at calc(48px - var(--r)) 0, #000 calc(var(--r) - 0.25px), transparent calc(var(--r) + 0.5px)); }
  /* Inverted top corners so the pill melts into the screen edge like the real notch. */
  .flare { width: 10px; height: 10px; }
  .flare.l { background: radial-gradient(circle at 0 100%, transparent 10px, #000 10.5px); }
  .flare.r { background: radial-gradient(circle at 100% 100%, transparent 10px, #000 10.5px); }

  /* Clip box: scaled to the current shape size. Hit-testing follows the transform, so hover matches the shape. */
  .pill {
    position: absolute;
    top: 0;
    left: -200px;
    width: 400px;
    height: 240px;
    transform-origin: 50% 0;
    overflow: hidden;
    cursor: pointer;
    will-change: transform;
  }
  .unclip { position: absolute; inset: 0; transform-origin: 50% 0; will-change: transform; }
  .layer { position: absolute; top: 0; left: 50%; box-sizing: border-box; transform-origin: top center; will-change: transform, opacity; }

  .row { display: flex; align-items: center; gap: 12px; padding: 0 18px; font-size: 13px; }
  .row.split { justify-content: space-between; }
  .ico { width: 20px; height: 20px; flex: none; }
  .orange { color: #ff9f0a; font-variant-numeric: tabular-nums; }
  .dot { width: 9px; height: 9px; border-radius: 50%; background: #ff9f0a; box-shadow: 0 0 8px #ff9f0a; flex: none; animation: pulse 1.6s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.45; } }
</style>
