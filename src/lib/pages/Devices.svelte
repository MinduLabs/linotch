<script lang="ts">
  // Devices: the laptop battery on the left; connected Bluetooth devices fill the rest of the page. One device
  // gets a big card (photo, name, battery ring); several share the width as tall cards side by side.
  import { island } from "$lib/island.svelte";
  import Ring from "$lib/ui/Ring.svelte";
  import DeviceArt from "$lib/ui/DeviceArt.svelte";
  import { photoFor } from "$lib/ui/device-art";
  import { tip } from "$lib/ui/tip";

  const p = $derived(island.power);
  const low = $derived(!!p && p.percent <= 20 && !p.plugged);
</script>

<div class="cols" class:solo={!p}>
  {#if p}
    <div class="battery">
      <Ring value={p.percent / 100} size={62} stroke={6} color={low ? "#ff453a" : "#30d158"}>
        <div class="pct"><b>{p.percent}</b><small>%</small></div>
      </Ring>
      <span class="state">
        {#if p.plugged}
          <svg viewBox="0 0 24 24"><path d="M13 2 4 14h7l-1 8 9-12h-7z" fill="#30d158" /></svg>{p.charging ? "Charging" : "Plugged in"}
        {:else}
          On battery
        {/if}
      </span>
    </div>
  {/if}

  <div class="list" class:one={island.btDevices.length === 1}>
    {#each island.btDevices as d (d.name)}
      {@const low = d.battery !== null && d.battery <= 20}
      <div class="dev">
        <span class="glyph" class:photo={!!photoFor(d)}><DeviceArt device={d} /></span>
        <span class="info">
          <span class="name" use:tip={d.name}>{d.name}</span>
          <span class="sub">Connected</span>
        </span>
        {#if d.battery !== null}
          <Ring value={d.battery / 100} size={52} stroke={5} color={low ? "#ff453a" : "#30d158"}>
            <b class="pct" class:low>{d.battery}<small>%</small></b>
          </Ring>
        {/if}
      </div>
    {:else}
      <div class="empty">
        <svg viewBox="0 0 24 24"><path d="M7 7l10 10-5 5V2l5 5L7 17" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" /></svg>
        No Bluetooth devices connected
      </div>
    {/each}
  </div>
</div>

<style>
  .cols { display: grid; grid-template-columns: 80px 1fr; gap: 12px; height: 100%; align-items: center; }
  .cols.solo { grid-template-columns: 1fr; }
  .battery { display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .pct { display: flex; align-items: baseline; font-size: 20px; font-weight: 700; font-variant-numeric: tabular-nums; }
  .pct small { font-size: 11px; color: #8e8e93; margin-left: 1px; }
  .state { display: inline-flex; align-items: center; gap: 4px; font-size: 11.5px; color: #8e8e93; white-space: nowrap; }
  .state svg { width: 12px; height: 12px; }

  /* Cards share the whole height; one device is a wide row, several are tall cards side by side. */
  .list { height: 100%; display: flex; gap: 8px; overflow-x: auto; scrollbar-width: none; }
  .dev { flex: 1 0 118px; min-width: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; padding: 8px 10px; }
  .glyph { width: 52px; height: 52px; flex: none; display: grid; place-items: center; color: #0a84ff; }
  .glyph { position: relative; }
  /* A soft light disc behind the photo only: black earbuds would vanish on the black pill. */
  .glyph.photo::before { content: ""; position: absolute; inset: -20%; border-radius: 50%; background: radial-gradient(circle, #3b3b42 0%, #26262b 52%, rgba(38, 38, 43, 0) 72%); }
  .glyph :global(img) { position: relative; }
  .info { min-width: 0; max-width: 100%; display: flex; flex-direction: column; align-items: center; gap: 1px; }
  .name { max-width: 100%; font-size: 12.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .sub { font-size: 11px; color: #8e8e93; }
  .dev .pct { font-size: 14px; }
  .dev .pct small { font-size: 8.5px; }
  .pct.low { color: #ff453a; }

  .list.one .dev { flex-direction: row; justify-content: flex-start; gap: 14px; padding: 8px 14px 8px 8px; }
  .list.one .glyph { width: 88px; height: 88px; }
  .list.one .info { flex: 1; align-items: flex-start; gap: 3px; }
  .list.one .name { font-size: 17px; font-weight: 700; letter-spacing: -0.2px; }
  .list.one .sub { font-size: 12.5px; }
  .list.one .dev .pct { font-size: 16px; }

  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; color: #8e8e93; font-size: 12px; text-align: center; }
  .empty svg { width: 22px; height: 22px; opacity: 0.6; }
</style>
