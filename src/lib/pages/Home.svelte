<script lang="ts">
  // Home: the player while something is playing; otherwise a glance (big clock, date, status chips).
  import { island, fmt } from "$lib/island.svelte";
  import Cover from "$lib/Cover.svelte";
  import Ring from "$lib/ui/Ring.svelte";

  const m = $derived(island.media);
  const pos = $derived(m && m.length > 0 ? m.position / m.length : 0);
  const now = $derived(new Date(island.now));
  const nextTodo = $derived(
    island.todos.filter((t) => !t.done).sort((a, b) => (a.due ?? Infinity) - (b.due ?? Infinity))[0] ?? null,
  );

  function seekClick(e: MouseEvent) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    island.seek((e.clientX - r.left) / r.width);
  }
</script>

{#if m}
  <div class="player">
    <div class="top">
      <Cover src={m.art} />
      <div class="meta">
        <strong>{m.title}</strong>
        <span>{m.artist}</span>
      </div>
    </div>
    <div class="seekrow">
      {#if m.length > 0}
        <span class="t">{fmt(m.position / 1e6)}</span>
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div class="seek" onclick={seekClick}><div class="track"><i style:transform="scaleX({pos})"></i></div></div>
        <span class="t">-{fmt(Math.max(0, (m.length - m.position) / 1e6))}</span>
      {:else}
        <span class="live"><b></b>LIVE</span>
        <div class="seek"><div class="track"><i class="full" style:transform="scaleX(1)"></i></div></div>
      {/if}
    </div>
    <div class="ctrl">
      <button class:dim={!m.canPrev} onclick={() => island.control("prev")} aria-label="Previous"><svg viewBox="0 0 24 24"><path d="M11 6.5v11L3.5 12zm9 0v11L12.5 12z" fill="currentColor" /></svg></button>
      <button class="big" onclick={() => island.control("toggle")} aria-label="Play/Pause">
        {#if m.playing}<svg viewBox="0 0 24 24"><rect x="6" y="5" width="4" height="14" rx="1.2" fill="currentColor" /><rect x="14" y="5" width="4" height="14" rx="1.2" fill="currentColor" /></svg>
        {:else}<svg viewBox="0 0 24 24"><path d="M8 5.5v13a1 1 0 0 0 1.5.86l10.4-6.5a1 1 0 0 0 0-1.72L9.5 4.64A1 1 0 0 0 8 5.5z" fill="currentColor" /></svg>{/if}
      </button>
      <button class:dim={!m.canNext} onclick={() => island.control("next")} aria-label="Next"><svg viewBox="0 0 24 24"><path d="M13 6.5v11l7.5-5.5zm-9 0v11l7.5-5.5z" fill="currentColor" /></svg></button>
    </div>
  </div>
{:else}
  <div class="glance">
    <div class="clock">{now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</div>
    <div class="date">{now.toLocaleDateString([], { weekday: "long", day: "numeric", month: "long" })}</div>
    <div class="chips">
      {#if island.power}
        <span class="chip fixed">
          <Ring value={island.power.percent / 100} size={13} stroke={2.2} color={island.power.plugged ? "#30d158" : "#e5e5ea"} />
          <span>{island.power.percent}%</span>
          {#if island.power.plugged}<svg viewBox="0 0 24 24"><path d="M13 2 4 14h7l-1 8 9-12h-7z" fill="#30d158" /></svg>{/if}
        </span>
      {/if}
      <!-- Two chips at most: the battery, then the next to-do, or else a connected Bluetooth device. -->
      {#each nextTodo ? [] : island.btDevices.slice(0, 1) as d}
        <span class="chip"><svg viewBox="0 0 24 24"><path d="M7 7l10 10-5 5V2l5 5L7 17" fill="none" stroke="#0a84ff" stroke-width="2.2" stroke-linejoin="round" /></svg><span>{d.name}</span></span>
      {/each}
      {#if nextTodo}
        <span class="chip"><svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="8" fill="none" stroke="#30d158" stroke-width="2.2" /></svg><span>{nextTodo.text}</span></span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .player { display: flex; flex-direction: column; gap: 9px; height: 100%; justify-content: center; }
  .top { display: flex; align-items: center; gap: 13px; }
  .meta { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .meta strong { font-size: 15px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta span { font-size: 12.5px; color: #8e8e93; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .seekrow { display: flex; align-items: center; gap: 10px; }
  .t { font-size: 10.5px; color: #8e8e93; font-variant-numeric: tabular-nums; min-width: 32px; }
  .t:last-child { text-align: right; }
  .seek { flex: 1; padding: 5px 0; cursor: pointer; }
  .track { height: 4px; border-radius: 2px; background: #3a3a3c; overflow: hidden; }
  /* scaleX, not width: a compositor-only change every position tick */
  .track i { display: block; height: 100%; background: #e5e5ea; transform-origin: 0 50%; transition: transform 0.5s linear; will-change: transform; }
  .track i.full { background: #ff375f; }
  .live { display: inline-flex; align-items: center; gap: 5px; color: #ff375f; font-size: 10.5px; font-weight: 700; letter-spacing: 0.5px; }
  .live b { width: 6px; height: 6px; border-radius: 50%; background: #ff375f; animation: pulse 1.6s ease-in-out infinite; }
  .ctrl { display: flex; justify-content: center; align-items: center; gap: 34px; }
  .ctrl button { width: 22px; height: 22px; color: #fff; opacity: 0.92; transition: transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.15s; }
  .ctrl button.dim { opacity: 0.35; }
  .ctrl button:hover { opacity: 1; transform: scale(1.14); }
  .ctrl button:active { transform: scale(0.88); }
  .ctrl .big { width: 28px; height: 28px; }
  .ctrl svg { width: 100%; height: 100%; display: block; }

  .glance { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; }
  .clock { font-size: 40px; font-weight: 700; letter-spacing: -1px; font-variant-numeric: tabular-nums; line-height: 1.05; }
  .date { font-size: 12.5px; color: #8e8e93; }
  .chips { display: flex; justify-content: center; gap: 6px; margin-top: 10px; width: 100%; }
  /* Chips share the row and shrink with an ellipsis; the battery chip is short and keeps its size. */
  .chip { display: inline-flex; align-items: center; gap: 5px; flex: 0 1 auto; min-width: 0; background: #1c1c1e; border-radius: 11px; padding: 4px 10px; font-size: 11.5px; color: #e5e5ea; white-space: nowrap; }
  .chip.fixed { flex: none; }
  .chip > span { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .chip svg { width: 12px; height: 12px; flex: none; }
  @keyframes pulse { 50% { opacity: 0.45; } }
</style>
