<script lang="ts">
  // Bluetooth connect / disconnect card, iPhone-style: the device comes in with a spring, the battery ring
  // sweeps up to its level. Apple headphones get their own drawing; every other device one shared
  // illustration (a glowing disc with the device type and a Bluetooth ripple).
  import type { BtDevice } from "$lib/island.svelte";
  import AirPodsArt from "$lib/ui/AirPodsArt.svelte";
  import DeviceGlyph from "$lib/ui/DeviceGlyph.svelte";
  import Ring from "$lib/ui/Ring.svelte";
  import { onMount } from "svelte";

  let { device, connected }: { device: BtDevice; connected: boolean } = $props();

  // Start closed / empty and flip on the next frame, so the CSS transitions run as an entrance.
  let shown = $state(false);
  onMount(() => {
    const id = requestAnimationFrame(() => requestAnimationFrame(() => (shown = true)));
    return () => cancelAnimationFrame(id);
  });
</script>

<div class="card" class:shown class:off={!connected}>
  <div class="art">
    {#if device.model}
      <AirPodsArt model={device.model} open={shown && connected} />
    {:else}
      <div class="disc">
        {#if connected}<i class="ripple"></i><i class="ripple two"></i>{/if}
        <span class="glyph"><DeviceGlyph {device} /></span>
      </div>
    {/if}
  </div>

  <div class="text">
    <strong>{device.name}</strong>
    <span class="status"><b></b>{connected ? "Connected" : "Disconnected"}</span>
  </div>

  {#if connected && device.battery !== null}
    <div class="batt">
      <Ring value={shown ? device.battery / 100 : 0} size={46} stroke={4.5} color={device.battery <= 20 ? "#ff453a" : "#30d158"}>
        <b class="pct">{device.battery}<small>%</small></b>
      </Ring>
    </div>
  {/if}
</div>

<style>
  .card { height: 100%; box-sizing: border-box; display: flex; align-items: center; gap: 14px; padding: 8px 22px 10px 16px; }
  .art { width: 92px; height: 92px; flex: none; transform: scale(0.6) translateY(-8px); opacity: 0; transition: transform 0.7s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.35s ease; }
  .shown .art { transform: none; opacity: 1; }
  .off .art { filter: grayscale(1) brightness(0.7); }

  .disc { position: relative; width: 100%; height: 100%; display: grid; place-items: center; }
  .disc::before { content: ""; position: absolute; inset: 12px; border-radius: 50%; background: radial-gradient(circle at 35% 30%, #5ac8fa, #0a84ff 55%, #0040dd); box-shadow: 0 6px 22px rgba(10, 132, 255, 0.45), inset 0 1px 1px rgba(255, 255, 255, 0.45); }
  .glyph { position: relative; width: 34px; height: 34px; color: #fff; }
  .ripple { position: absolute; inset: 12px; border-radius: 50%; border: 2px solid rgba(90, 200, 250, 0.6); animation: ripple 2s cubic-bezier(0.22, 1, 0.36, 1) 0.4s infinite; opacity: 0; }
  .ripple.two { animation-delay: 1.4s; }
  @keyframes ripple { 0% { transform: scale(1); opacity: 0.8; } 100% { transform: scale(1.45); opacity: 0; } }

  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; transform: translateX(14px); opacity: 0; transition: transform 0.55s cubic-bezier(0.22, 1, 0.36, 1) 0.12s, opacity 0.4s ease 0.12s; }
  .shown .text { transform: none; opacity: 1; }
  strong { font-size: 16px; font-weight: 700; letter-spacing: -0.2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .status { display: inline-flex; align-items: center; gap: 6px; font-size: 12.5px; color: #8e8e93; }
  .status b { width: 7px; height: 7px; border-radius: 50%; background: #30d158; box-shadow: 0 0 8px rgba(48, 209, 88, 0.8); }
  .off .status b { background: #636366; box-shadow: none; }

  .batt { flex: none; transform: scale(0.7); opacity: 0; transition: transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1) 0.2s, opacity 0.4s ease 0.2s; }
  .shown .batt { transform: none; opacity: 1; }
  .pct { font-size: 13px; font-weight: 700; font-variant-numeric: tabular-nums; }
  .pct small { font-size: 8.5px; color: #8e8e93; margin-left: 0.5px; }
</style>
