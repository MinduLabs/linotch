<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { scale } from "svelte/transition";
  import Eq from "$lib/Eq.svelte";
  import { cubicOut } from "svelte/easing";
  import { island, setHitRect, fmt, fmtBytes } from "$lib/island.svelte";
  import { Spring, OPEN, CLOSE } from "$lib/spring";


  const IN = { start: 0.8, duration: 340, delay: 90, easing: cubicOut };
  const OUT = { start: 0.9, duration: 130, easing: cubicOut };
  let pillEl: HTMLElement;
  const sw = new Spring(island.size.w);
  const sh = new Spring(island.size.h);
  const sr = new Spring(island.size.r);
  let raf = 0;
  let last = 0;
  let cfg = OPEN;
  let token = 0;

  const apply = () => {
    pillEl.style.width = `${sw.x}px`;
    pillEl.style.height = `${Math.max(0, sh.x)}px`;
    const r = Math.max(0, sr.x);
    pillEl.style.borderRadius = `0 0 ${r}px ${r}px`;
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

  const pos = $derived(island.media ? (island.media.length ? island.media.position / island.media.length : 0) : 0);
  const dlSize = $derived(island.downloads.reduce((n, d) => n + d.size, 0));
  const dlSpeed = $derived(island.downloads.reduce((n, d) => n + d.speed, 0));
  const timerFrac = $derived(island.timerTotal ? island.timerLeft / island.timerTotal : 0);

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

  function seekClick(e: MouseEvent) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    island.seek((e.clientX - r.left) / r.width);
  }
</script>

<div class="stage">
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
    <div class="clip">
    <!-- ALERT -->
    {#if island.mode === "alert" && island.alert}
      {@const a = island.alert}
      <div class="layer row" use:fit={island.size} in:scale={IN} out:scale={OUT}>
        {#if a.kind === "power"}
          <svg class="ico {a.charging ? 'green' : ''}" viewBox="0 0 24 24"><path d="M13 2 4 14h7l-1 8 9-12h-7z" fill="currentColor" /></svg>
          <span class="grow">{a.charging ? "Charging" : "On battery"}</span>
          <b>{a.percent}%</b>
        {:else if a.kind === "volume"}
          <svg class="ico" viewBox="0 0 24 24"><path d="M3 9v6h4l5 4V5L7 9zm13.5 3a4.5 4.5 0 0 0-2.5-4v8a4.5 4.5 0 0 0 2.5-4z" fill="currentColor" /></svg>
          <div class="bar grow"><i style:width="{a.muted ? 0 : Math.min(a.percent, 100)}%"></i></div>
          <b class="w36">{a.muted ? "Mute" : a.percent}</b>
        {:else if a.kind === "bluetooth"}
          <svg class="ico {a.connected ? 'blue' : ''}" viewBox="0 0 24 24">
            {#if a.icon.includes("head") || a.icon.includes("audio")}
              <path d="M12 3a8 8 0 0 0-8 8v7a2 2 0 0 0 2 2h2v-8H6v-1a6 6 0 0 1 12 0v1h-2v8h2a2 2 0 0 0 2-2v-7a8 8 0 0 0-8-8z" fill="currentColor" />
            {:else if a.icon.includes("keyboard")}
              <path d="M3 6h18a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1zm2 3v2h2V9zm4 0v2h2V9zm4 0v2h2V9zm4 0v2h2V9zM7 13v2h10v-2z" fill="currentColor" />
            {:else if a.icon.includes("mouse")}
              <path d="M12 2a6 6 0 0 0-6 6v8a6 6 0 0 0 12 0V8a6 6 0 0 0-6-6zm-1 2.1V9H7.1A5 5 0 0 1 11 4.1z" fill="currentColor" />
            {:else}
              <path d="M7 7l10 10-5 5V2l5 5L7 17" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
            {/if}
          </svg>
          <span class="grow ellipsis">{a.name}</span>
          <span class="dim-text">{a.connected ? "Connected" : "Disconnected"}</span>
          {#if a.connected && a.battery !== null}
            <b class="batt" class:low={a.battery <= 20}>{a.battery}%</b>
          {/if}
        {:else if a.kind === "download"}
          <svg class="ico green" viewBox="0 0 24 24"><path d="M12 3v11m0 0-4.5-4.5M12 14l4.5-4.5M5 19h14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
          <span class="grow ellipsis">{a.name}</span>
          <span class="dim-text">Downloaded</span>
        {:else if a.kind === "mic"}
          <span class="dot"></span>
          <span class="grow ellipsis">{a.app}</span>
          <span class="dim-text">is using the microphone</span>
        {:else if a.kind === "notif"}
          {#if a.icon}<img class="appicon" src={a.icon} alt="" />{/if}
          <div class="notif" class:has-icon={a.icon}>
            <small>{a.app}</small>
            <strong>{a.summary}</strong>
            <span>{a.body}</span>
          </div>
        {:else}
          <svg class="ico orange ring" viewBox="0 0 24 24"><circle cx="12" cy="13" r="8" fill="none" stroke="currentColor" stroke-width="2" /><path d="M12 9v4l3 2M5 3 2 6m17-3 3 3" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" /></svg>
          <div class="notif grow">
            <small>Timer</small>
            <strong>Time's up</strong>
          </div>
          <span class="dim-text">Click to stop</span>
        {/if}
      </div>

    <!-- EXPANDED -->
    {:else if island.mode === "expanded"}
      {#if island.media}
        {@const m = island.media}
        <div class="layer expanded" use:fit={island.size} in:scale={IN} out:scale={OUT}>
          <div class="top">
            {#if m.art}<img class="art" src={m.art} alt="" />{:else}<div class="art ph"></div>{/if}
            <div class="meta">
              <strong>{m.title}</strong>
              <span>{m.artist}</span>
            </div>
            <Eq on={m.playing} />
          </div>
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="seek" class:live={m.length <= 0} onclick={seekClick}>
            <div class="track"><i style:width="{m.length > 0 ? pos * 100 : 100}%"></i></div>
          </div>
          {#if m.length > 0}
            <div class="times"><span>{fmt(m.position / 1e6)}</span><span>-{fmt(Math.max(0, (m.length - m.position) / 1e6))}</span></div>
          {:else}
            <div class="times"><span class="livetag"><b></b>LIVE</span></div>
          {/if}
          <div class="ctrl">
            <button class:dim={!m.canPrev} onclick={() => island.control("prev")} aria-label="Previous"><svg viewBox="0 0 24 24"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z" fill="currentColor" /></svg></button>
            <button class="big" onclick={() => island.control("toggle")} aria-label="Play/Pause">
              {#if m.playing}<svg viewBox="0 0 24 24"><path d="M6 5h4v14H6zm8 0h4v14h-4z" fill="currentColor" /></svg>
              {:else}<svg viewBox="0 0 24 24"><path d="M8 5v14l11-7z" fill="currentColor" /></svg>{/if}
            </button>
            <button class:dim={!m.canNext} onclick={() => island.control("next")} aria-label="Next"><svg viewBox="0 0 24 24"><path d="M16 6h2v12h-2zM6 18l8.5-6L6 6z" fill="currentColor" /></svg></button>
          </div>
          <button class="bell" class:off={island.muted} onclick={() => island.toggleMute()} aria-label="Do not disturb" title={island.muted ? "Notifications off" : "Notifications on"}>
            <svg viewBox="0 0 24 24"><path d="M12 22a2 2 0 0 0 2-2h-4a2 2 0 0 0 2 2zm6-6V11c0-3.1-1.6-5.6-4.5-6.3V4a1.5 1.5 0 0 0-3 0v.7C7.6 5.4 6 7.9 6 11v5l-2 2v1h16v-1z" fill="currentColor" />{#if island.muted}<path d="M4 3 20 21" stroke="#ff9f0a" stroke-width="2.2" stroke-linecap="round" />{/if}</svg>
          </button>
        </div>
      {:else}
        <div class="layer expanded" use:fit={island.size} in:scale={IN} out:scale={OUT}>
          <div class="clock">
            {#if island.timerEnd}
              <div class="big-time">{fmt(island.timerLeft)}</div>
              <button class="chip" onclick={() => island.cancelTimer()}>Cancel</button>
            {:else}
              <div class="big-time">{new Date(island.now).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</div>
              <div class="chips">
                {#each [1, 5, 10, 25] as min}
                  <button class="chip" onclick={() => island.startTimer(min)}>{min}m</button>
                {/each}
              </div>
            {/if}
          </div>
          <button class="bell" class:off={island.muted} onclick={() => island.toggleMute()} aria-label="Do not disturb" title={island.muted ? "Notifications off" : "Notifications on"}>
            <svg viewBox="0 0 24 24"><path d="M12 22a2 2 0 0 0 2-2h-4a2 2 0 0 0 2 2zm6-6V11c0-3.1-1.6-5.6-4.5-6.3V4a1.5 1.5 0 0 0-3 0v.7C7.6 5.4 6 7.9 6 11v5l-2 2v1h16v-1z" fill="currentColor" />{#if island.muted}<path d="M4 3 20 21" stroke="#ff9f0a" stroke-width="2.2" stroke-linecap="round" />{/if}</svg>
          </button>
        </div>
      {/if}

    <!-- COMPACT -->
    {:else if island.mode === "compact"}
      <div class="layer row split" use:fit={island.size} in:scale={IN} out:scale={OUT}>
        <!-- A download outranks media: it is short-lived, and a paused browser tab still counts as media. -->
        {#if island.downloads.length && !island.timerEnd}
          <svg class="ico dl" viewBox="0 0 24 24"><path d="M12 3v11m0 0-4.5-4.5M12 14l4.5-4.5M5 19h14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
        {:else if island.media}
          {#if island.media.art}<img class="art sm" src={island.media.art} alt="" />{:else}<div class="art sm ph"></div>{/if}
        {:else}
          <svg class="ico" viewBox="0 0 24 24"><circle cx="12" cy="13" r="8" fill="none" stroke="currentColor" stroke-width="2" /><path d="M12 9v4l3 2" stroke="currentColor" stroke-width="2" fill="none" /></svg>
        {/if}
        {#if island.timerEnd}
          <b class="orange">{fmt(island.timerLeft)}</b>
        {:else if island.downloads.length}
          <span class="dltext" title={island.downloads.map((d) => d.name).join(", ")}>
            {island.downloads.length > 1 ? `${island.downloads.length} files · ` : ""}{fmtBytes(dlSize)}{dlSpeed > 0 ? ` · ${fmtBytes(dlSpeed)}/s` : ""}
          </span>
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
  .stage { position: fixed; inset: 0 0 auto 0; display: flex; justify-content: center; }
  .pill {
    --flare: 10px;
    position: relative;
    background: #000;
    overflow: visible;
    cursor: pointer;
  }
  /* Inverted top corners so the pill melts into the screen edge like the real notch. */
  .pill::before,
  .pill::after {
    content: "";
    position: absolute;
    top: 0;
    width: var(--flare);
    height: var(--flare);
  }
  .pill::before { left: calc(var(--flare) * -1); background: radial-gradient(circle at 0 100%, transparent var(--flare), #000 calc(var(--flare) + 0.5px)); }
  .pill::after { right: calc(var(--flare) * -1); background: radial-gradient(circle at 100% 100%, transparent var(--flare), #000 calc(var(--flare) + 0.5px)); }

  .clip { position: absolute; inset: 0; overflow: hidden; border-radius: inherit; }
  .layer { position: absolute; top: 0; left: 50%; box-sizing: border-box; transform-origin: top center; }

  .row { display: flex; align-items: center; gap: 12px; padding: 0 18px; font-size: 13px; }
  .row.split { justify-content: space-between; }
  .grow { flex: 1; min-width: 0; }
  .w36 { width: 36px; text-align: right; }
  .ico { width: 20px; height: 20px; flex: none; }
  .ico.green { color: #30d158; }
  .orange { color: #ff9f0a; font-variant-numeric: tabular-nums; }

  .art { width: 52px; height: 52px; border-radius: 12px; object-fit: cover; flex: none; background: #222; }
  .art.sm { width: 19px; height: 19px; border-radius: 5px; }
  .art.ph { background: linear-gradient(135deg, #444, #1c1c1e); }

  .bar { height: 6px; border-radius: 3px; background: #3a3a3c; overflow: hidden; }
  .bar i { display: block; height: 100%; background: #fff; border-radius: 3px; transition: width 0.15s; }

  .notif { display: flex; flex-direction: column; gap: 2px; padding: 12px 22px; min-width: 0; }
  .notif small { color: #8e8e93; font-size: 11px; }
  .notif strong { font-size: 14px; }
  .notif span { color: #d1d1d6; font-size: 12.5px; overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }

  .expanded { padding: 14px 24px 12px; display: flex; flex-direction: column; gap: 6px; }
  .top { display: flex; align-items: center; gap: 14px; }
  .meta { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .meta strong { font-size: 15px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta span { font-size: 13px; color: #8e8e93; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .seek { padding: 6px 0 2px; cursor: pointer; }
  .track { height: 5px; border-radius: 3px; background: #3a3a3c; overflow: hidden; }
  .track i { display: block; height: 100%; background: #e5e5ea; }
  .times { display: flex; justify-content: space-between; font-size: 10.5px; color: #8e8e93; font-variant-numeric: tabular-nums; }
  .ctrl { display: flex; justify-content: center; align-items: center; gap: 30px; }
  .ctrl button { width: 24px; height: 24px; opacity: 0.9; transition: transform 0.15s, opacity 0.15s; }
  .ctrl button.dim { opacity: 0.4; }
  .ctrl button:hover { opacity: 1; transform: scale(1.12); }
  .ctrl button:active { transform: scale(0.92); }
  .ctrl .big { width: 30px; height: 30px; }
  .ctrl svg { width: 100%; height: 100%; }

  .bell { position: absolute; right: 22px; bottom: 12px; width: 20px; height: 20px; color: #8e8e93; transition: color 0.15s, transform 0.15s; }
  .bell:hover { color: #fff; transform: scale(1.12); }
  .bell.off { color: #ff9f0a; }
  .bell svg { width: 100%; height: 100%; }

  .ellipsis { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dim-text { color: #8e8e93; font-size: 12px; white-space: nowrap; }
  .ico.blue { color: #0a84ff; }
  .ico.ring { animation: ring 0.9s ease-in-out infinite; }
  @keyframes ring { 0%, 50%, 100% { transform: rotate(0); } 10%, 30% { transform: rotate(-14deg); } 20%, 40% { transform: rotate(14deg); } }
  .ico.dl { color: #0a84ff; animation: nudge 1s ease-in-out infinite; }
  @keyframes nudge { 50% { transform: translateY(2px); } }
  .dltext { font-size: 12px; font-variant-numeric: tabular-nums; color: #d1d1d6; white-space: nowrap; }
  .seek.live { cursor: default; }
  .seek.live .track i { background: #ff375f; }
  .livetag { display: inline-flex; align-items: center; gap: 5px; color: #ff375f; font-weight: 600; letter-spacing: 0.5px; }
  .livetag b { width: 6px; height: 6px; border-radius: 50%; background: #ff375f; animation: pulse 1.6s ease-in-out infinite; }
  .batt { font-variant-numeric: tabular-nums; color: #30d158; }
  .batt.low { color: #ff453a; }
  .dot { width: 9px; height: 9px; border-radius: 50%; background: #ff9f0a; box-shadow: 0 0 8px #ff9f0a; flex: none; animation: pulse 1.6s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.45; } }
  .appicon { width: 40px; height: 40px; border-radius: 9px; object-fit: contain; flex: none; margin-left: 4px; }
  .notif.has-icon { padding-left: 4px; }

  .clock { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; }
  .big-time { font-size: 34px; font-weight: 600; letter-spacing: -0.5px; font-variant-numeric: tabular-nums; }
  .chips { display: flex; gap: 8px; }
  .chip { background: #2c2c2e; border-radius: 14px; padding: 5px 14px; font-size: 12.5px; transition: background 0.15s; }
  .chip:hover { background: #3a3a3c; }
</style>
