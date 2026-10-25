<script lang="ts">
  // To-do: done count on the left, the list on the right (tick, delete), and an add row with an optional
  // reminder time; at that time the island shows the task as an alert.
  import { island } from "$lib/island.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { tick } from "svelte";
  import { tip } from "$lib/ui/tip";

  let text = $state("");
  // Reminder presets, in minutes from now. Nothing is chosen by default; tap a chip to pick one, again to clear.
  // (No native time picker: it showed a stray "12:30 PM" and looked out of place.)
  const REMIND = [15, 30, 60, 120, 240];
  let remind = $state<number | null>(null);
  const label = (m: number) => (m < 60 ? `${m}m` : `${m / 60}h`);
  let input: HTMLInputElement | undefined = $state();

  const done = $derived(island.todos.filter((t) => t.done).length);
  const sorted = $derived([...island.todos].sort((a, b) => Number(a.done) - Number(b.done) || (a.due ?? Infinity) - (b.due ?? Infinity)));

  // The island is a dock window, which never gets keyboard focus on its own: ask for it before typing.
  const focusWindow = () => getCurrentWindow().setFocus().catch(() => {});

  async function openAdd() {
    island.todoAdding = true;
    await focusWindow();
    await tick();
    input?.focus();
  }

  function save() {
    island.addTodo(text, remind ? Date.now() + remind * 60_000 : null);
    text = "";
    remind = null;
    island.todoAdding = false;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") save();
    if (e.key === "Escape") {
      island.todoAdding = false;
      text = "";
      remind = null;
    }
  }

  const dueLabel = (ms: number) => new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
</script>

<div class="cols">
  <div class="side">
    <div class="count"><b>{done}</b><span>/{island.todos.length}</span></div>
    <div class="label">Done</div>
    <button class="chip add" class:on={island.todoAdding} onclick={() => (island.todoAdding ? (island.todoAdding = false) : openAdd())}>
      <svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" /></svg>
      Add
    </button>
  </div>

  <div class="list">
    {#if island.todoAdding}
      <div class="newbox">
      <div class="new">
        <input
          bind:this={input}
          bind:value={text}
          placeholder="New task"
          onpointerdown={focusWindow}
          onkeydown={onKey}
          onblur={() => island.recheckHover(island.size)}
        />
        <button class="ok" onclick={save} aria-label="Save">
          <svg viewBox="0 0 24 24"><path d="m5 12.5 4.5 4.5L19 7.5" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </button>
      </div>
      <div class="remind">
        <svg viewBox="0 0 24 24"><path d="M6 17h12l-1.6-2.2V10.5a4.4 4.4 0 0 0-8.8 0v4.3zm4 2.5a2 2 0 0 0 4 0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
        {#each REMIND as m}
          <button class="r" class:on={remind === m} onclick={() => (remind = remind === m ? null : m)}>{label(m)}</button>
        {/each}
      </div>
      </div>
    {/if}
    {#each sorted as t (t.id)}
      <div class="item" class:done={t.done}>
        <button class="check" onclick={() => island.toggleTodo(t.id)} aria-label={t.done ? "Mark as not done" : "Mark as done"}>
          {#if t.done}<svg viewBox="0 0 24 24"><path d="m6.5 12.5 3.5 3.5 7.5-8" fill="none" stroke="#000" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" /></svg>{/if}
        </button>
        <span class="textcol">
          <span class="text" use:tip={t.text}>{t.text}</span>
          {#if t.due && !t.done}<span class="due"><svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="2.4" /><path d="M12 8v4l2.5 1.5" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" /></svg>{dueLabel(t.due)}</span>{/if}
        </span>
        <button class="del" onclick={() => island.removeTodo(t.id)} aria-label="Delete">
          <svg viewBox="0 0 24 24"><path d="M7 7l10 10M17 7 7 17" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" /></svg>
        </button>
      </div>
    {:else}
      {#if !island.todoAdding}<div class="empty">Nothing to do. Add a task.</div>{/if}
    {/each}
  </div>
</div>

<style>
  .cols { display: grid; grid-template-columns: 62px 1fr; gap: 12px; height: 100%; align-items: center; }
  .side { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; }
  .count { display: flex; align-items: baseline; line-height: 1; }
  .count b { font-size: 34px; font-weight: 700; letter-spacing: -1px; font-variant-numeric: tabular-nums; }
  .count span { font-size: 15px; color: #8e8e93; font-weight: 600; }
  .label { font-size: 11px; color: #8e8e93; margin-bottom: 8px; }
  .chip { display: inline-flex; align-items: center; gap: 4px; background: #1c1c1e; border-radius: 11px; padding: 4px 10px; font-size: 11.5px; color: #e5e5ea; transition: background 0.15s, color 0.15s; }
  .chip svg { width: 11px; height: 11px; }
  .chip:hover { background: #2c2c2e; }
  .chip.on { background: rgba(48, 209, 88, 0.2); color: #30d158; }

  .list { display: flex; flex-direction: column; gap: 5px; max-height: 100%; overflow-y: auto; scrollbar-width: none; padding-right: 2px; }
  .item { display: flex; align-items: center; gap: 8px; background: #1c1c1e; border-radius: 11px; padding: 5px 9px; animation: rise 0.28s cubic-bezier(0.22, 1, 0.36, 1); }
  @keyframes rise { from { opacity: 0; transform: translateY(6px); } }
  .check { width: 16px; height: 16px; flex: none; border-radius: 50%; border: 1.8px solid #48484a; display: grid; place-items: center; transition: background 0.2s, border-color 0.2s, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1); }
  .check:active { transform: scale(0.8); }
  .check svg { width: 12px; height: 12px; }
  .done .check { background: #30d158; border-color: #30d158; }
  .textcol { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .text { font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; transition: color 0.2s; }
  .done .text { color: #636366; text-decoration: line-through; }
  .due { display: inline-flex; align-items: center; gap: 3px; font-size: 10px; color: #30d158; font-variant-numeric: tabular-nums; }
  .due svg { width: 9px; height: 9px; }
  .del { width: 14px; height: 14px; flex: none; color: #636366; opacity: 0; transition: opacity 0.15s, color 0.15s; }
  .item:hover .del { opacity: 1; }
  .del:hover { color: #ff453a; }
  .del svg { width: 100%; height: 100%; display: block; }

  .new { display: flex; align-items: center; gap: 6px; background: #1c1c1e; border-radius: 12px; padding: 3px 4px 3px 11px; transition: background 0.15s; }
  .new:focus-within { background: #26262a; }
  .new input { flex: 1; min-width: 0; background: none; border: 0; outline: none; color: #fff; font: inherit; font-size: 12.5px; padding: 4px 0; caret-color: #30d158; }
  .new input::placeholder { color: #636366; }
  .newbox { display: flex; flex-direction: column; gap: 5px; }
  .remind { display: flex; align-items: center; gap: 5px; padding-left: 2px; color: #636366; }
  .remind svg { width: 14px; height: 14px; flex: none; margin-right: 2px; }
  .r { flex: 1; height: 22px; border-radius: 11px; background: #1c1c1e; color: #aeaeb2; font-size: 11px; font-variant-numeric: tabular-nums; transition: background 0.15s, color 0.15s; }
  .r:hover { background: #2c2c2e; }
  .r.on { background: rgba(48, 209, 88, 0.2); color: #30d158; }
  .ok { width: 22px; height: 22px; flex: none; border-radius: 50%; background: #30d158; color: #000; display: grid; place-items: center; }
  .ok svg { width: 13px; height: 13px; }
  .empty { color: #636366; font-size: 12px; text-align: center; }
</style>
