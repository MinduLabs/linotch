<script lang="ts">
  // Bars share a horizontal axis and wander to random heights while `on`, like Apple's now-playing indicator.
  let { on = false }: { on?: boolean } = $props();

  const MIN = 4;
  const MAX = 16;
  let heights = $state([MIN, MIN, MIN, MIN]);

  $effect(() => {
    // Never read `heights` in here: this effect would then re-run on every update and restart the timer.
    if (!on) {
      heights = [MIN, MIN, MIN, MIN];
      return;
    }
    const step = () => (heights = Array.from({ length: 4 }, () => MIN + Math.random() * (MAX - MIN)));
    step();
    const id = setInterval(step, 260);
    return () => clearInterval(id);
  });
</script>

<!-- scaleY instead of height: a compositor-only change, no layout on every step -->
<div class="eq">{#each heights as h}<i style:transform="scaleY({h / MAX})"></i>{/each}</div>

<style>
  .eq { display: flex; align-items: center; gap: 2.5px; height: 16px; }
  i { width: 3px; height: 16px; border-radius: 2px; background: #ff375f; transition: transform 0.26s ease-in-out; will-change: transform; }
</style>
