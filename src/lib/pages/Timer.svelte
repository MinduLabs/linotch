<script lang="ts">
  // Timer: pick minutes (stepper or a preset) and start; while running, a countdown with its progress ring.
  // When it ends the existing alarm rings until it is clicked.
  import { island, fmt } from "$lib/island.svelte";
  import Ring from "$lib/ui/Ring.svelte";

  const PRESETS = [5, 10, 25, 45];
  let minutes = $state(25);
  const running = $derived(!!island.timerEnd);
  const frac = $derived(island.timerTotal ? island.timerLeft / island.timerTotal : 1);
  const step = (d: number) => (minutes = Math.max(1, Math.min(180, minutes + d)));
</script>

<div class="cols">
  <!-- The big ring is the main button, like Nusa's: start when idle, cancel while running. -->
  <button class="dial" class:running onclick={() => (running ? island.cancelTimer() : island.startTimer(minutes))} aria-label={running ? "Cancel timer" : "Start timer"}>
    <Ring value={running ? frac : 1} size={92} stroke={7} color="#ff453a" track="rgba(255,69,58,0.18)">
      {#if running}
        <svg class="icon" viewBox="0 0 24 24"><rect x="7.5" y="7.5" width="9" height="9" rx="2" fill="currentColor" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" /></svg>
      {:else}
        <!-- Triangle drawn around its centroid (x = 12), so it sits in the middle of the ring instead of leaning right. -->
        <svg class="icon" viewBox="0 0 24 24"><path d="M8.6 6.2v11.6L18.4 12z" fill="currentColor" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" /></svg>
      {/if}
    </Ring>
  </button>

  <div class="main">
    {#if running}
      <div class="big">{fmt(island.timerLeft)}</div>
      <div class="label">{Math.round(island.timerTotal / 60)} min timer · tap the ring to stop</div>
    {:else}
      <div class="stepper">
        <button onclick={() => step(-1)} aria-label="One minute less">−</button>
        <div class="big">{String(minutes).padStart(2, "0")}:00</div>
        <button onclick={() => step(1)} aria-label="One minute more">+</button>
      </div>
      <div class="row">
        {#each PRESETS as p}
          <button class="chip" class:on={minutes === p} onclick={() => (minutes = p)}>{p}m</button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .cols { display: flex; align-items: center; gap: 26px; height: 100%; padding: 0 8px; }
  .dial { flex: none; border-radius: 50%; transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), filter 0.15s; }
  .dial:hover { filter: brightness(1.15); transform: scale(1.04); }
  .dial:active { transform: scale(0.92); }
  .icon { width: 30px; height: 30px; color: #ff453a; display: block; }
  .main { flex: 1; min-width: 0; display: flex; flex-direction: column; align-items: center; gap: 12px; }
  .big { font-size: 44px; font-weight: 700; letter-spacing: -1.2px; font-variant-numeric: tabular-nums; line-height: 1; }
  .label { font-size: 12.5px; color: #8e8e93; }
  .stepper { display: flex; align-items: center; justify-content: space-between; gap: 18px; width: 100%; }
  .stepper .big { flex: 1; text-align: center; }
  .stepper button { width: 30px; height: 30px; border-radius: 50%; background: #1c1c1e; color: #e5e5ea; font-size: 18px; line-height: 1; transition: background 0.15s, transform 0.15s; }
  .stepper button:hover { background: #2c2c2e; }
  .stepper button:active { transform: scale(0.88); }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; width: 100%; }
  .chip { flex: 1; background: #1c1c1e; border-radius: 14px; padding: 6px 0; font-size: 13px; font-weight: 600; color: #e5e5ea; transition: background 0.15s, color 0.15s; }
  .chip:hover { background: #2c2c2e; }
  .chip.on { background: rgba(255, 69, 58, 0.22); color: #ff6961; }
</style>
