<script lang="ts">
  // Our own drawing of Apple's headphones (not Apple artwork), animated like the iPhone pairing card:
  // the case lid lifts and the buds float up out of it; AirPods Max swing into place.
  let { model, open = true }: { model: "airpods" | "airpods-pro" | "airpods-max"; open?: boolean } = $props();
  const uid = Math.random().toString(36).slice(2, 8);
</script>

<svg viewBox="0 0 100 100" class:open aria-hidden="true">
  <defs>
    <linearGradient id="w-{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff" />
      <stop offset="1" stop-color="#d4d4da" />
    </linearGradient>
    <linearGradient id="s-{uid}" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="#c9c9cf" />
      <stop offset="0.5" stop-color="#ffffff" />
      <stop offset="1" stop-color="#c9c9cf" />
    </linearGradient>
    <linearGradient id="in-{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#e9e9ee" />
      <stop offset="1" stop-color="#9c9ca3" />
    </linearGradient>
    <radialGradient id="glow-{uid}" cx="0.5" cy="0.6" r="0.5">
      <stop offset="0" stop-color="rgba(10,132,255,0.35)" />
      <stop offset="1" stop-color="rgba(10,132,255,0)" />
    </radialGradient>
  </defs>

  <circle cx="50" cy="58" r="44" fill="url(#glow-{uid})" class="glow" />

  {#if model === "airpods-max"}
    <g class="max">
      <path d="M24 56V44a26 26 0 0 1 52 0v12" fill="none" stroke="url(#s-{uid})" stroke-width="6" stroke-linecap="round" />
      <path d="M30 40a20 20 0 0 1 40 0" fill="none" stroke="#9a9aa0" stroke-width="1.5" stroke-dasharray="1.6 1.8" />
      <rect x="15" y="52" width="20" height="30" rx="9" fill="url(#w-{uid})" />
      <rect x="65" y="52" width="20" height="30" rx="9" fill="url(#w-{uid})" />
      <rect x="18" y="56" width="5" height="22" rx="2.5" fill="#b9b9bf" />
      <rect x="77" y="56" width="5" height="22" rx="2.5" fill="#b9b9bf" />
    </g>
  {:else}
    {@const pro = model === "airpods-pro"}
    {@const x = pro ? 18 : 24}
    {@const w = pro ? 64 : 52}
    <!-- Seen from the front: the open lid stands up behind the buds, the buds rise out of the case. -->
    <g class="lid-open">
      <rect {x} y="24" width={w} height="34" rx={pro ? 14 : 12} fill="url(#in-{uid})" />
      <rect x={x + 3} y="27" width={w - 6} height="28" rx={pro ? 11 : 9} fill="none" stroke="rgba(255,255,255,0.35)" stroke-width="0.8" />
    </g>
    <g class="bud l">
      {#if pro}
        <rect x="31" y="43" width="7" height="17" rx="3.5" fill="url(#w-{uid})" />
        <ellipse cx="38" cy="40" rx="9.5" ry="10" fill="url(#w-{uid})" />
        <ellipse cx="41.5" cy="36" rx="4" ry="4.6" fill="#a8a8ae" />
      {:else}
        <rect x="32.5" y="42" width="6.5" height="24" rx="3.25" fill="url(#w-{uid})" />
        <ellipse cx="38" cy="39" rx="8.5" ry="9" fill="url(#w-{uid})" />
        <circle cx="41" cy="36" r="2.6" fill="#8e8e93" />
      {/if}
    </g>
    <g class="bud r">
      {#if pro}
        <rect x="62" y="43" width="7" height="17" rx="3.5" fill="url(#w-{uid})" />
        <ellipse cx="62" cy="40" rx="9.5" ry="10" fill="url(#w-{uid})" />
        <ellipse cx="58.5" cy="36" rx="4" ry="4.6" fill="#a8a8ae" />
      {:else}
        <rect x="61" y="42" width="6.5" height="24" rx="3.25" fill="url(#w-{uid})" />
        <ellipse cx="62" cy="39" rx="8.5" ry="9" fill="url(#w-{uid})" />
        <circle cx="59" cy="36" r="2.6" fill="#8e8e93" />
      {/if}
    </g>
    <rect {x} y="54" width={w} height={pro ? 36 : 40} rx={pro ? 15 : 13} fill="url(#w-{uid})" />
    <rect class="lid-closed" {x} y="46" width={w} height="12" rx={pro ? 8 : 7} fill="url(#s-{uid})" />
    <line x1={x + 2} y1="58.5" x2={x + w - 2} y2="58.5" stroke="#bdbdc3" stroke-width="0.8" />
    <circle class="led" cx="50" cy="73" r="1.6" fill="#30d158" />
  {/if}
</svg>

<style>
  svg { width: 100%; height: 100%; display: block; overflow: visible; }
  .glow { opacity: 0; transition: opacity 0.8s ease 0.2s; }
  .open .glow { opacity: 1; }

  /* The lid flips up from its hinge: the closed lid fades while the open one grows upward behind the buds. */
  .lid-open { transform-box: fill-box; transform-origin: 50% 100%; transform: scaleY(0); transition: transform 0.5s cubic-bezier(0.34, 1.3, 0.64, 1) 0.08s; }
  .open .lid-open { transform: scaleY(1); }
  .lid-closed { transition: opacity 0.18s ease 0.08s; }
  .open .lid-closed { opacity: 0; }

  .bud { transform-box: view-box; transition: transform 0.7s cubic-bezier(0.34, 1.45, 0.64, 1), opacity 0.3s ease; opacity: 0; transform: translateY(16px); }
  .bud.l { transition-delay: 0.32s; transform-origin: 38px 50px; }
  .bud.r { transition-delay: 0.42s; transform-origin: 62px 50px; }
  .open .bud { opacity: 1; }
  .open .bud.l { transform: translate(-2px, -6px) rotate(-8deg); }
  .open .bud.r { transform: translate(2px, -6px) rotate(8deg); }
  .open .bud { animation: float 3.2s ease-in-out 1.2s infinite; }
  @keyframes float { 50% { translate: 0 -2px; } }

  .led { animation: led 1.6s ease-in-out infinite; }
  @keyframes led { 50% { opacity: 0.35; } }

  .max { transform-box: fill-box; transform-origin: 50% 20%; transform: rotate(-12deg) translateY(10px); opacity: 0; transition: transform 0.75s cubic-bezier(0.34, 1.45, 0.64, 1) 0.1s, opacity 0.4s; }
  .open .max { transform: none; opacity: 1; }
</style>
