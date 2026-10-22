<script lang="ts">
  // The row every page shares, like Nusa: page title, then battery ring +
  // date + time, then the wave (only while music plays) and the do-not-disturb bell.
  import { island } from "$lib/island.svelte";
  import Eq from "$lib/Eq.svelte";
  import Ring from "./Ring.svelte";

  let { title, clock = true }: { title: string; clock?: boolean } = $props();

  const now = $derived(new Date(island.now));
  const date = $derived(now.toLocaleDateString([], { weekday: "short", day: "numeric", month: "short" }));
  const time = $derived(now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }));
  const p = $derived(island.power);
</script>

<header>
  <h2>{title}</h2>
  <div class="status">
    {#if p && clock}
      <Ring value={p.percent / 100} color={p.percent <= 20 && !p.plugged ? "#ff453a" : "#30d158"} />
    {/if}
    {#if clock}
      <span class="date">{date}</span>
      <b class="time">{time}</b>
    {/if}
  </div>
  <div class="right">
    {#if island.media?.playing}<Eq on />{/if}
    <button class="bell" class:off={island.muted} onclick={() => island.toggleMute()} aria-label="Do not disturb" title={island.muted ? "Notifications off" : "Notifications on"}>
      <svg viewBox="0 0 24 24"><path d="M12 22a2 2 0 0 0 2-2h-4a2 2 0 0 0 2 2zm6-6V11c0-3.1-1.6-5.6-4.5-6.3V4a1.5 1.5 0 0 0-3 0v.7C7.6 5.4 6 7.9 6 11v5l-2 2v1h16v-1z" fill="currentColor" />{#if island.muted}<path d="M4 3 20 21" stroke="#ff9f0a" stroke-width="2.4" stroke-linecap="round" />{/if}</svg>
    </button>
  </div>
</header>

<style>
  header { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; height: 22px; }
  h2 { margin: 0; font-size: 14px; font-weight: 700; letter-spacing: -0.2px; white-space: nowrap; }
  .status { display: flex; align-items: center; gap: 7px; font-size: 12px; }
  .date { color: #d1d1d6; }
  .time { font-weight: 700; font-variant-numeric: tabular-nums; }
  .right { display: flex; align-items: center; justify-content: flex-end; gap: 10px; }
  .bell { width: 15px; height: 15px; color: #8e8e93; transition: color 0.15s, transform 0.15s; }
  .bell:hover { color: #fff; transform: scale(1.12); }
  .bell.off { color: #ff9f0a; }
  .bell svg { width: 100%; height: 100%; display: block; }
</style>
