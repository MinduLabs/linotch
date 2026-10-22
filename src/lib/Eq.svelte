<script lang="ts">
  // Now-playing bars. Pure CSS: no JS timer, nothing for the main thread to do. Every bar steps on the same
  // 0.2 s grid (delays are multiples of it), so the whole wave changes in at most 5 frames a second; measured,
  // each WebKit frame costs about the same whatever its size, and a smooth 60 fps wave burned ~25% of a core.
  let { on = false }: { on?: boolean } = $props();
</script>

<div class="eq" class:on><i></i><i></i><i></i><i></i></div>

<style>
  .eq { display: flex; align-items: center; gap: 2.5px; height: 16px; }
  i { width: 3px; height: 16px; border-radius: 2px; background: #ff375f; transform: scaleY(0.25); will-change: transform; }
  .on i { animation: bar 1.2s steps(1, end) infinite; }
  .on i:nth-child(2) { animation-delay: -0.4s; }
  .on i:nth-child(3) { animation-delay: -0.8s; }
  .on i:nth-child(4) { animation-delay: -0.2s; }
  @keyframes bar {
    0% { transform: scaleY(0.3); }
    16.6% { transform: scaleY(1); }
    33.3% { transform: scaleY(0.55); }
    50% { transform: scaleY(0.85); }
    66.6% { transform: scaleY(0.4); }
    83.3% { transform: scaleY(0.7); }
  }
</style>
