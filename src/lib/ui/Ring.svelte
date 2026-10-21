<script lang="ts">
  // Progress ring (battery, timer). `value` is 0..1. The arc eases to a new value, so a card can open at 0
  // and let it sweep up to the real level.
  let {
    value,
    size = 14,
    stroke = 2.5,
    color = "#30d158",
    track = "rgba(255,255,255,0.14)",
    children,
  }: { value: number; size?: number; stroke?: number; color?: string; track?: string; children?: import("svelte").Snippet } = $props();
  const r = $derived((size - stroke) / 2);
  const c = $derived(2 * Math.PI * r);
  const v = $derived(Math.max(0, Math.min(1, value)));
</script>

<span class="ring" style:width="{size}px" style:height="{size}px">
  <svg viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke={track} stroke-width={stroke} />
    <circle
      class="arc"
      cx={size / 2}
      cy={size / 2}
      {r}
      fill="none"
      stroke={color}
      stroke-width={stroke}
      stroke-linecap="round"
      stroke-dasharray={c}
      stroke-dashoffset={c * (1 - v)}
    />
  </svg>
  {#if children}<span class="inner">{@render children()}</span>{/if}
</span>

<style>
  .ring { position: relative; display: inline-grid; place-items: center; flex: none; }
  svg { position: absolute; inset: 0; transform: rotate(-90deg); }
  .arc { transition: stroke-dashoffset 0.9s cubic-bezier(0.22, 1, 0.36, 1), stroke 0.3s; }
  .inner { position: relative; display: grid; place-items: center; }
</style>
