<script lang="ts">
  // Devices: the laptop battery on the left, connected Bluetooth devices with their battery on the right.
  import { island } from "$lib/island.svelte";
  import Ring from "$lib/ui/Ring.svelte";
  import DeviceGlyph from "$lib/ui/DeviceGlyph.svelte";

  const p = $derived(island.power);
  const low = $derived(!!p && p.percent <= 20 && !p.plugged);
</script>

<div class="cols">
  <div class="battery">
    {#if p}
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
    {:else}
      <span class="state">No battery</span>
    {/if}
  </div>

  <div class="list">
    {#each island.btDevices as d (d.name)}
      <div class="dev">
        <span class="glyph"><DeviceGlyph device={d} /></span>
        <span class="info">
          <span class="name">{d.name}</span>
          {#if d.battery !== null}
            <span class="b"><Ring value={d.battery / 100} size={11} stroke={2} color={d.battery <= 20 ? "#ff453a" : "#30d158"} />{d.battery}%</span>
          {:else}
            <span class="b dim">Connected</span>
          {/if}
        </span>
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
  .battery { display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .pct { display: flex; align-items: baseline; }
  .pct b { font-size: 20px; font-weight: 700; font-variant-numeric: tabular-nums; }
  .pct small { font-size: 11px; color: #8e8e93; margin-left: 1px; }
  .state { display: inline-flex; align-items: center; gap: 4px; font-size: 11.5px; color: #8e8e93; white-space: nowrap; }
  .state svg { width: 12px; height: 12px; }
  .list { display: flex; flex-direction: column; gap: 6px; max-height: 100%; overflow-y: auto; scrollbar-width: none; }
  .dev { display: flex; align-items: center; gap: 10px; background: #1c1c1e; border-radius: 13px; padding: 6px 11px; }
  .glyph { width: 22px; height: 22px; flex: none; color: #0a84ff; }
  .info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .name { font-size: 12.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .b { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; font-variant-numeric: tabular-nums; color: #aeaeb2; }
  .b.dim { color: #8e8e93; }
  .empty { display: flex; flex-direction: column; align-items: center; gap: 6px; color: #8e8e93; font-size: 12px; text-align: center; }
  .empty svg { width: 22px; height: 22px; opacity: 0.6; }
</style>
