<script lang="ts">
  // Alerts, Nusa-style: a round icon, a bold title over a grey subtitle, and a big value on the right.
  // Volume stays a one-line HUD with a bar; notifications keep the sender's app icon.
  import type { Alert } from "$lib/island.svelte";

  let { alert: a }: { alert: Exclude<Alert, { kind: "bluetooth" }> } = $props();
</script>

{#if a.kind === "volume"}
  <div class="hud">
    <svg class="vol" viewBox="0 0 24 24"><path d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5z" fill="currentColor" />{#if !a.muted}<path d="M15.5 9a4 4 0 0 1 0 6M18 6.5a7.5 7.5 0 0 1 0 11" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />{:else}<path d="m16 9.5 5 5m0-5-5 5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />{/if}</svg>
    <div class="bar"><i style:transform="scaleX({a.muted ? 0 : Math.min(a.percent, 100) / 100})"></i></div>
    <b class="num">{a.muted ? "Mute" : a.percent}</b>
  </div>
{:else if a.kind === "notif"}
  <div class="notif">
    {#if a.icon}<img class="appicon" src={a.icon} alt="" />{/if}
    <div class="ntext">
      <small>{a.app}</small>
      <strong>{a.summary}</strong>
      <span>{a.body}</span>
    </div>
  </div>
{:else}
  {@const v =
    a.kind === "power"
      ? { color: "#30d158", title: a.charging ? "Charging" : "On battery", sub: a.charging ? "Charger connected" : "Charger disconnected", value: `${a.percent}%` }
      : a.kind === "mic"
        ? { color: "#ff9f0a", title: "Microphone in use", sub: a.app, value: "" }
        : a.kind === "todo"
          ? { color: "#30d158", title: "Reminder", sub: a.text, value: "" }
          : { color: "#ff453a", title: "Time's up", sub: "Click to stop the alarm", value: "" }}
  <div class="two" style:--c={v.color}>
    <span class="badge" class:ring={a.kind === "timer"}>
      {#if a.kind === "power"}
        <svg viewBox="0 0 24 24"><path d="M13 2 4 14h7l-1 8 9-12h-7z" fill="currentColor" /></svg>
      {:else if a.kind === "mic"}
        <svg viewBox="0 0 24 24"><rect x="9" y="3" width="6" height="11" rx="3" fill="currentColor" /><path d="M6 11a6 6 0 0 0 12 0M12 17v3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
      {:else if a.kind === "todo"}
        <svg viewBox="0 0 24 24"><path d="m6 12.5 4 4 8-9" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
      {:else}
        <svg viewBox="0 0 24 24"><circle cx="12" cy="13" r="7.5" fill="none" stroke="currentColor" stroke-width="2.2" /><path d="M12 9.5V13l2.3 1.6M5 3.5 2.5 6m16.5-2.5L21.5 6" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" /></svg>
      {/if}
    </span>
    <div class="text">
      <strong>{v.title}</strong>
      <span>{v.sub}</span>
    </div>
    {#if v.value}<b class="value">{v.value}</b>{/if}
  </div>
{/if}

<style>
  .two { height: 100%; box-sizing: border-box; display: flex; align-items: center; gap: 11px; padding: 0 20px 0 12px; }
  .badge { width: 32px; height: 32px; flex: none; border-radius: 50%; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 20%, transparent); animation: pop 0.5s cubic-bezier(0.34, 1.56, 0.64, 1); }
  .badge svg { width: 17px; height: 17px; }
  .badge.ring svg { animation: ring 0.9s ease-in-out infinite; }
  @keyframes pop { from { transform: scale(0.4); opacity: 0; } }
  @keyframes ring { 0%, 50%, 100% { transform: rotate(0); } 10%, 30% { transform: rotate(-14deg); } 20%, 40% { transform: rotate(14deg); } }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .text strong { font-size: 13.5px; font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .text span { font-size: 11.5px; color: #8e8e93; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .value { font-size: 22px; font-weight: 600; letter-spacing: -0.5px; font-variant-numeric: tabular-nums; }

  .hud { height: 100%; box-sizing: border-box; display: flex; align-items: center; gap: 12px; padding: 0 18px; }
  .vol { width: 19px; height: 19px; flex: none; }
  .bar { flex: 1; height: 6px; border-radius: 3px; background: #3a3a3c; overflow: hidden; }
  .bar i { display: block; height: 100%; background: #fff; border-radius: 3px; transform-origin: 0 50%; transition: transform 0.18s ease-out; }
  .num { width: 34px; text-align: right; font-size: 13px; font-variant-numeric: tabular-nums; }

  .notif { height: 100%; box-sizing: border-box; display: flex; align-items: center; gap: 4px; padding: 0 14px 0 12px; }
  .appicon { width: 40px; height: 40px; border-radius: 9px; object-fit: contain; flex: none; }
  .ntext { display: flex; flex-direction: column; gap: 2px; padding: 12px 10px; min-width: 0; }
  .ntext small { color: #8e8e93; font-size: 11px; }
  .ntext strong { font-size: 14px; }
  .ntext span { color: #d1d1d6; font-size: 12.5px; overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; }
</style>
