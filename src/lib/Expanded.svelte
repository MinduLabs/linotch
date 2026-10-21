<script lang="ts">
  // The open island: shared header, the pages side by side in a strip that slides (transform only, so the
  // compositor does it), ‹ › arrows on the sides and page dots at the bottom, like Nusa.
  import { island, PAGES, type Page } from "$lib/island.svelte";
  import Header from "$lib/ui/Header.svelte";
  import Home from "$lib/pages/Home.svelte";
  import Devices from "$lib/pages/Devices.svelte";
  import Timer from "$lib/pages/Timer.svelte";
  import Todo from "$lib/pages/Todo.svelte";

  const META: Record<Page, { title: string; accent: string; color: string }> = {
    home: { title: "Now ", accent: "Playing", color: "#ff375f" },
    devices: { title: "Dev", accent: "ices", color: "#0a84ff" },
    timer: { title: "Tim", accent: "er", color: "#ff453a" },
    todo: { title: "Todo", accent: "List", color: "#30d158" },
  };

  const index = $derived(PAGES.indexOf(island.page));
  const meta = $derived(
    island.page === "home" && !island.media ? { title: "To", accent: "day", color: "#ff375f" } : META[island.page],
  );
  const go = (d: number) => (island.page = PAGES[(index + d + PAGES.length) % PAGES.length]);
</script>

<div class="wrap">
  <Header {...meta} clock={!(island.page === "home" && !island.media)} />

  <div class="body">
    <button class="arrow l" onclick={() => go(-1)} aria-label="Previous page">
      <svg viewBox="0 0 24 24"><path d="m14.5 6-6 6 6 6" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
    </button>
    <div class="viewport">
      <div class="strip" style:transform="translateX({-index * 25}%)">
        {#each PAGES as p, i}
          <section class:active={i === index} aria-hidden={i !== index}>
            {#if p === "home"}<Home />{:else if p === "devices"}<Devices />{:else if p === "timer"}<Timer />{:else}<Todo />{/if}
          </section>
        {/each}
      </div>
    </div>
    <button class="arrow r" onclick={() => go(1)} aria-label="Next page">
      <svg viewBox="0 0 24 24"><path d="m9.5 6 6 6-6 6" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
    </button>
  </div>

  <div class="dots">
    {#each PAGES as p, i}
      <button class:on={i === index} style:--c={META[p].color} onclick={() => (island.page = p)} aria-label={p}></button>
    {/each}
  </div>
</div>

<style>
  .wrap { height: 100%; box-sizing: border-box; padding: 11px 16px 8px; display: flex; flex-direction: column; gap: 6px; }
  .body { position: relative; flex: 1; min-height: 0; display: flex; align-items: center; gap: 6px; }
  .viewport { flex: 1; min-width: 0; height: 100%; overflow: hidden; }
  .strip { display: flex; width: 400%; height: 100%; transition: transform 0.48s cubic-bezier(0.32, 0.72, 0, 1); will-change: transform; }
  section { width: 25%; height: 100%; box-sizing: border-box; padding: 0 4px; overflow: hidden; opacity: 0.25; transform: scale(0.96); transition: opacity 0.4s ease, transform 0.48s cubic-bezier(0.32, 0.72, 0, 1); }
  section.active { opacity: 1; transform: none; }
  .arrow { width: 22px; height: 22px; flex: none; border-radius: 50%; background: #1c1c1e; color: #aeaeb2; display: grid; place-items: center; transition: background 0.15s, color 0.15s, transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1); }
  .arrow:hover { background: #2c2c2e; color: #fff; }
  .arrow:active { transform: scale(0.85); }
  .arrow svg { width: 13px; height: 13px; }
  .dots { display: flex; justify-content: center; gap: 5px; height: 6px; }
  .dots button { width: 5px; height: 5px; border-radius: 3px; background: #48484a; transition: width 0.35s cubic-bezier(0.32, 0.72, 0, 1), background 0.3s; }
  .dots button.on { width: 16px; background: var(--c); }
</style>
