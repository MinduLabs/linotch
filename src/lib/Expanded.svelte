<script lang="ts">
  // The open island, like Nusa: each page has the size its content needs, so switching pages morphs the pill
  // (the same spring as every other change) while the old page slides out and the new one slides in from the
  // side you went. Transforms and opacity only. Each page block keeps the size it was built for while it
  // leaves, so nothing reflows mid-transition.
  import { island, PAGES, type Page } from "$lib/island.svelte";
  import { cubicOut } from "svelte/easing";
  import Header from "$lib/ui/Header.svelte";
  import Home from "$lib/pages/Home.svelte";
  import Devices from "$lib/pages/Devices.svelte";
  import Timer from "$lib/pages/Timer.svelte";
  import Todo from "$lib/pages/Todo.svelte";

  // One white word per page; the music page has no title at all.
  const COLOR: Record<Page, string> = { home: "#ff375f", devices: "#0a84ff", timer: "#ff453a", todo: "#30d158" };
  const TITLE: Record<Page, string> = { home: "", devices: "Devices", timer: "Timer", todo: "To-do" };

  const index = $derived(PAGES.indexOf(island.page));
  const idleHome = $derived(island.page === "home" && !island.media);
  const heading = $derived(idleHome ? "Today" : TITLE[island.page]);
  let dir = $state(1);
  const go = (d: number) => {
    dir = d;
    island.page = PAGES[(index + d + PAGES.length) % PAGES.length];
  };
  const jump = (p: Page) => {
    dir = PAGES.indexOf(p) >= index ? 1 : -1;
    island.page = p;
  };

  function fit(node: HTMLElement, size: { w: number; h: number }) {
    const set = ({ w, h }: { w: number; h: number }) => {
      node.style.width = `${w}px`;
      node.style.height = `${h}px`;
      node.style.marginLeft = `${-w / 2}px`;
    };
    set(size);
    return { update: set };
  }

  // Direction is read when the transition starts, so in and out always move the same way.
  const slideIn = (_: Element) => {
    const d = dir;
    return { delay: 70, duration: 300, easing: cubicOut, css: (t: number, u: number) => `opacity: ${t}; transform: translateX(${d * u * 26}px)` };
  };
  const slideOut = (_: Element) => {
    const d = dir;
    return { duration: 150, easing: cubicOut, css: (t: number, u: number) => `opacity: ${t}; transform: translateX(${-d * u * 26}px)` };
  };
</script>

{#key island.page}
  <div class="page" use:fit={island.size} in:slideIn out:slideOut>
    <Header title={heading} clock={!idleHome} />

    <div class="body">
      <button class="arrow" onclick={() => go(-1)} aria-label="Previous page">
        <svg viewBox="0 0 24 24"><path d="m14.5 6-6 6 6 6" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
      </button>
      <section>
        {#if island.page === "home"}<Home />{:else if island.page === "devices"}<Devices />{:else if island.page === "timer"}<Timer />{:else}<Todo />{/if}
      </section>
      <button class="arrow" onclick={() => go(1)} aria-label="Next page">
        <svg viewBox="0 0 24 24"><path d="m9.5 6 6 6-6 6" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
      </button>
    </div>

    <div class="dots">
      {#each PAGES as p, i}
        <button class:on={i === index} style:--c={COLOR[p]} onclick={() => jump(p)} aria-label={p}></button>
      {/each}
    </div>
  </div>
{/key}

<style>
  .page { position: absolute; top: 0; left: 50%; box-sizing: border-box; padding: 11px 16px 8px; display: flex; flex-direction: column; gap: 6px; will-change: transform, opacity; }
  .body { flex: 1; min-height: 0; display: flex; align-items: center; gap: 6px; }
  section { flex: 1; min-width: 0; height: 100%; box-sizing: border-box; padding: 0 4px; overflow: hidden; }
  /* The button is a tall strip (easy to hit); the visible 24px disc is drawn behind the chevron. */
  .arrow { position: relative; width: 38px; height: 100%; min-height: 56px; margin: 0 -8px; flex: none; color: #e5e5ea; display: grid; place-items: center; transition: color 0.15s; }
  .arrow::before { content: ""; position: absolute; width: 24px; height: 24px; border-radius: 50%; background: #3a3a3c; transition: background 0.15s, transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1); }
  .arrow:hover { color: #fff; }
  .arrow:hover::before { background: #5a5a5e; }
  .arrow:active::before { transform: scale(0.85); }
  .arrow svg { position: relative; width: 13px; height: 13px; }
  .dots { display: flex; justify-content: center; gap: 5px; height: 6px; }
  .dots button { width: 5px; height: 5px; border-radius: 3px; background: #48484a; }
  .dots button.on { width: 16px; background: var(--c); }
</style>
