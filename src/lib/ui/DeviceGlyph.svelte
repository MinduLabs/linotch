<script lang="ts">
  // Small line icon for a Bluetooth device, picked from BlueZ's icon name (and the name, to tell earbuds
  // from over-ear headphones: BlueZ calls both "audio-headset").
  import type { BtDevice } from "$lib/island.svelte";

  let { device }: { device: BtDevice } = $props();

  const kind = $derived(deviceKind(device));
</script>

<script lang="ts" module>
  import type { BtDevice as Device } from "$lib/island.svelte";
  export type Kind = "earbuds" | "headphones" | "speaker" | "mouse" | "keyboard" | "gamepad" | "phone" | "other";

  export function deviceKind(d: Device): Kind {
    const n = d.name.toLowerCase();
    if (d.model === "airpods" || d.model === "airpods-pro") return "earbuds";
    if (d.model === "airpods-max") return "headphones";
    if (d.icon.startsWith("audio-head") || d.icon === "audio-headphones") {
      return /buds|airpods|earbud|pods|tws|freebuds|\bbud\b/.test(n) ? "earbuds" : "headphones";
    }
    if (d.icon.startsWith("audio")) return "speaker";
    if (d.icon.includes("mouse") || d.icon.includes("tablet")) return "mouse";
    if (d.icon.includes("keyboard")) return "keyboard";
    if (d.icon.includes("gaming") || d.icon.includes("joystick")) return "gamepad";
    if (d.icon.includes("phone")) return "phone";
    return "other";
  }
</script>

<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {#if kind === "earbuds"}
    <path d="M7.5 4a3 3 0 0 1 3 3v2a3 3 0 0 1-2 2.83V18a1.5 1.5 0 0 1-3 0V7a3 3 0 0 1 2-3z" />
    <path d="M16.5 4a3 3 0 0 0-3 3v2a3 3 0 0 0 2 2.83V18a1.5 1.5 0 0 0 3 0V7a3 3 0 0 0-2-3z" />
  {:else if kind === "headphones"}
    <path d="M4 15v-3a8 8 0 0 1 16 0v3" />
    <rect x="3" y="14" width="4.5" height="6.5" rx="1.8" />
    <rect x="16.5" y="14" width="4.5" height="6.5" rx="1.8" />
  {:else if kind === "speaker"}
    <rect x="6" y="3" width="12" height="18" rx="3" />
    <circle cx="12" cy="14.5" r="3" />
    <circle cx="12" cy="7.5" r="1" />
  {:else if kind === "mouse"}
    <rect x="6.5" y="3" width="11" height="18" rx="5.5" />
    <path d="M12 7v3" />
  {:else if kind === "keyboard"}
    <rect x="2.5" y="6" width="19" height="12" rx="2.5" />
    <path d="M6 10h.01M9.5 10h.01M13 10h.01M16.5 10h.01M8 14h8" />
  {:else if kind === "gamepad"}
    <path d="M7 8h10a4 4 0 0 1 3.9 4.9l-.8 3.6a2.5 2.5 0 0 1-4.4 1L14 15h-4l-1.7 2.5a2.5 2.5 0 0 1-4.4-1l-.8-3.6A4 4 0 0 1 7 8z" />
    <path d="M8 11v3M6.5 12.5h3M15.5 12h.01M17.5 13.5h.01" />
  {:else if kind === "phone"}
    <rect x="6.5" y="2.5" width="11" height="19" rx="2.5" />
    <path d="M11 18.5h2" />
  {:else}
    <path d="M7 7l10 10-5 5V2l5 5L7 17" />
  {/if}
</svg>

<style>
  svg { width: 100%; height: 100%; display: block; }
</style>
