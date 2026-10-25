<script lang="ts">
  // Now-playing bars. Pure CSS: no JS timer, nothing for the main thread to do. Each 0.25 s move between two
  // heights is cut into 6 steps = 24 frames a second. A smooth tween would run at the display rate (60 fps),
  // and every WebKit frame costs about the same whatever its size.
  let { on = false }: { on?: boolean } = $props();
</script>

<div class="eq" class:on><i></i><i></i><i></i><i></i></div>

<style>
  .eq { display: flex; align-items: center; gap: 2.5px; height: 16px; }
  i { width: 3px; height: 16px; border-radius: 2px; background: #ff375f; transform: scaleY(0.25); will-change: transform; }
  .on i { animation: bar 1.5s linear infinite; }
  .on i:nth-child(2) { animation-delay: -0.5s; }
  .on i:nth-child(3) { animation-delay: -1s; }
  .on i:nth-child(4) { animation-delay: -0.25s; }
  /* The timing function of a keyframe shapes the move to the next one. */
  @keyframes bar {
    0%, 16.6%, 33.3%, 50%, 66.6%, 83.3% { animation-timing-function: steps(6, end); }
    0% { transform: scaleY(0.3); }
    16.6% { transform: scaleY(1); }
    33.3% { transform: scaleY(0.55); }
    50% { transform: scaleY(0.85); }
    66.6% { transform: scaleY(0.4); }
    83.3% { transform: scaleY(0.7); }
    100% { transform: scaleY(0.3); }
  }
</style>
