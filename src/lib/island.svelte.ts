import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Media = { title: string; artist: string; art: string | null; artKey: string; playing: boolean; position: number; length: number; canPrev: boolean; canNext: boolean };
export type Alert =
  | { kind: "power"; percent: number; charging: boolean }
  | { kind: "volume"; percent: number; muted: boolean }
  | { kind: "notif"; app: string; summary: string; body: string; desktopEntry: string | null; icon: string | null }
  | { kind: "bluetooth"; device: BtDevice; connected: boolean }
  | { kind: "mic"; app: string }
  | { kind: "todo"; text: string }
  | { kind: "timer" };

export type Power = { percent: number; charging: boolean; plugged: boolean };
/** `model` is set for Apple headphones, which get their own drawing. */
export type BtDevice = { name: string; icon: string; model: "airpods" | "airpods-pro" | "airpods-max" | null; battery: number | null };
export type Todo = { id: number; text: string; done: boolean; due: number | null; reminded: boolean };

/** Pages of the open island, in order. The pill keeps one size for all of them. */
export const PAGES = ["home", "devices", "timer", "todo"] as const;
export type Page = (typeof PAGES)[number];

export type Mode = "idle" | "compact" | "expanded" | "alert";
export type Size = { w: number; h: number; r: number };

// Logical px. The OS window is a fixed 420x220 (win.rs); only the pill moves inside it.
const SIZES = {
  idle: { w: 190, h: 27, r: 10 }, // MacBook notch
  compact: { w: 250, h: 27, r: 13 },
  hud: { w: 300, h: 36, r: 18 }, // volume
  alert: { w: 340, h: 56, r: 28 }, // two-line alerts
  alertNotif: { w: 371, h: 84, r: 32 },
  btCard: { w: 371, h: 120, r: 40 },
  expanded: { w: 371, h: 190, r: 40 },
} satisfies Record<string, Size>;

class Island {
  media = $state<Media | null>(null);
  alert = $state<Alert | null>(null);
  hovering = $state(false);
  timerEnd = $state<number | null>(null);
  timerTotal = $state(0);
  now = $state(Date.now());
  /** Do-not-disturb: island stays quiet for notifications. */
  muted = $state(false);
  /** App currently recording from the microphone (a call, Meet, Zoom...). */
  micApp = $state<string | null>(null);
  /** Which page the open island shows; back to home every time it closes. */
  page = $state<Page>("home");
  power = $state<Power | null>(null);
  btDevices = $state<BtDevice[]>([]);
  todos = $state<Todo[]>([]);

  #mediaGone: ReturnType<typeof setTimeout> | undefined;
  #art: { key: string; data: string | null } | null = null;
  #alertTimeout: ReturnType<typeof setTimeout> | undefined;
  #leaveTimeout: ReturnType<typeof setTimeout> | undefined;
  #lastPower: { plugged: boolean } | null = null;

  mode = $derived<Mode>(
    this.alert ? "alert" : this.hovering ? "expanded" : this.media || this.timerEnd || this.micApp ? "compact" : "idle",
  );

  size = $derived<Size>(
    this.mode === "alert"
      ? this.alert?.kind === "notif"
        ? SIZES.alertNotif
        : this.alert?.kind === "bluetooth"
          ? SIZES.btCard
          : this.alert?.kind === "volume"
            ? SIZES.hud
            : SIZES.alert
      : this.mode === "expanded"
        ? SIZES.expanded
        : this.mode === "compact"
          ? SIZES.compact
          : SIZES.idle,
  );

  timerLeft = $derived(this.timerEnd ? Math.max(0, Math.ceil((this.timerEnd - this.now) / 1000)) : 0);

  #coverFor(key: string) {
    return this.#art && this.#art.key === key ? this.#art.data : null;
  }

  toggleMute() {
    this.muted = !this.muted;
    try {
      localStorage.setItem("linotch.muted", this.muted ? "1" : "0");
    } catch {}
  }

  /** Opens the app behind the notification and dismisses the alert. */
  openAlert() {
    const a = this.alert;
    if (a?.kind === "notif") invoke("open_app", { desktopEntry: a.desktopEntry, app: a.app }).catch(() => {});
    this.#suppressHover = true;
    this.#close();
    this.dismissAlert();
  }

  async init() {
    try {
      this.muted = localStorage.getItem("linotch.muted") === "1";
    } catch {}
    setInterval(() => {
      this.now = Date.now();
      if (this.timerEnd && this.now >= this.timerEnd) {
        this.timerEnd = null;
        this.ringAlarm();
      }
      const due = this.todos.find((t) => !t.done && !t.reminded && t.due !== null && t.due <= this.now);
      if (due) {
        this.#updateTodo(due.id, { reminded: true });
        this.pushAlert({ kind: "todo", text: due.text }, 6000);
      }
    }, 250);

    // Current state up front: change events alone leave the Devices page empty until something changes.
    invoke<Power | null>("power_now").then((p) => (this.power = p), () => {});
    invoke<BtDevice[]>("bt_devices").then((d) => (this.btDevices = d), () => {});
    invoke<string>("todos_load").then(
      (json) => {
        try {
          this.todos = JSON.parse(json);
        } catch {}
      },
      () => {},
    );

    // Changing track briefly reports no metadata. Hold the last track for a moment so the
    // island doesn't flip to the clock (and resize) in between.
    // Cover art comes on its own event, only when it changes (it is large); merge it in here.
    // A cover only counts for the track it was sent for; otherwise the UI shows its default cover.
    await listen<{ key: string; data: string | null }>("media-art", (e) => {
      this.#art = e.payload;
      if (this.media) this.media = { ...this.media, art: this.#coverFor(this.media.artKey) };
    });
    await listen<Media | null>("media", (e) => {
      clearTimeout(this.#mediaGone);
      if (e.payload) this.media = { ...e.payload, art: this.#coverFor(e.payload.artKey) };
      else this.#mediaGone = setTimeout(() => (this.media = null), 2500);
    });
    await listen<{ percent: number; charging: boolean; plugged: boolean } | null>("power", (e) => {
      const p = e.payload;
      if (p && this.#lastPower && p.plugged !== this.#lastPower.plugged) {
        this.pushAlert({ kind: "power", percent: p.percent, charging: p.plugged }, 3000);
      }
      this.#lastPower = p;
      this.power = p;
    });
    await listen<{ app: string } | null>("mic", (e) => {
      const started = e.payload && !this.micApp;
      this.micApp = e.payload?.app ?? null;
      if (started) this.pushAlert({ kind: "mic", app: e.payload!.app }, 2500);
    });
    // Crossing events straight from GTK (see win.rs): the DOM's mouseenter/mouseleave alone get lost.
    await listen("pointer-left", () => this.leave());
    await listen<[number, number]>("pointer-entered", (e) => {
      const [x, y] = e.payload;
      this.pointerAt(x, y);
      if (document.elementFromPoint(x, y)?.closest(".pill")) this.enter();
    });
    await listen<BtDevice & { kind: string }>("bluetooth", (e) => {
      const { kind, ...device } = e.payload;
      invoke<BtDevice[]>("bt_devices").then((d) => (this.btDevices = d), () => {});
      const cur = this.alert;
      if (kind === "battery") {
        // Battery levels usually arrive just after the connect event: update the open card in place.
        if (cur?.kind === "bluetooth" && cur.device.name === device.name) this.alert = { ...cur, device };
        return;
      }
      this.pushAlert({ kind: "bluetooth", device, connected: kind === "connected" }, 4500);
    });
    await listen<{ percent: number; muted: boolean }>("volume", (e) =>
      this.pushAlert({ kind: "volume", ...e.payload }, 1600),
    );
    await listen<{ app: string; summary: string; body: string; desktopEntry: string | null; icon: string | null }>("notification", (e) => {
      if (!this.muted) this.pushAlert({ kind: "notif", ...e.payload }, 5000);
    });
  }

  /** Timer done: a ringing "Time's up" that stays until it is clicked (the sound gives up after 2 min). */
  ringAlarm() {
    clearTimeout(this.#alertTimeout);
    this.alert = { kind: "timer" };
    invoke("alarm", { on: true }).catch(() => {});
  }

  pushAlert(a: Alert, ms: number) {
    // A ringing alarm is not pushed aside by a volume change or a notification.
    if (this.alert?.kind === "timer") return;
    this.alert = a;
    clearTimeout(this.#alertTimeout);
    this.#alertTimeout = setTimeout(() => (this.alert = null), ms);
  }

  dismissAlert() {
    if (this.alert?.kind === "timer") invoke("alarm", { on: false }).catch(() => {});
    clearTimeout(this.#alertTimeout);
    this.alert = null;
  }

  // After clicking an alert the cursor is still over the pill; don't pop open the clock until it leaves.
  #suppressHover = false;

  /** Last pointer position seen inside the window (client px). */
  #pointer: { x: number; y: number } | null = null;
  pointerAt(x: number, y: number) {
    this.#pointer = { x, y };
  }
  /**
   * The input region shrinks together with the pill, and a pointer left outside it sends no further events,
   * so no mouseleave ever arrives and the island would stay open. If the last known position is outside the
   * new region the pointer must be outside too (inside it, we would be getting move events): leave now.
   */
  recheckHover(size: Size) {
    if (!this.hovering || !this.#pointer) return;
    const half = (Math.ceil(size.w) + 24) / 2;
    const cx = window.innerWidth / 2;
    const { x, y } = this.#pointer;
    if (Math.abs(x - cx) > half || y > Math.ceil(size.h)) {
      clearTimeout(this.#leaveTimeout);
      this.#suppressHover = false;
      this.#close();
    }
  }

  /** Closing always lands back on the home page next time. */
  #close() {
    this.hovering = false;
    this.page = "home";
  }

  enter() {
    clearTimeout(this.#leaveTimeout);
    if (!this.#suppressHover) this.hovering = true;
  }
  leave() {
    clearTimeout(this.#leaveTimeout);
    this.#suppressHover = false;
    this.#leaveTimeout = setTimeout(() => {
      // Typing a to-do: the pointer drifting off the pill must not throw the text away.
      if (document.activeElement?.closest(".pill input")) return;
      this.#close();
    }, 350);
  }

  // ---------- to-do ----------
  #saveTodos() {
    invoke("todos_save", { data: JSON.stringify(this.todos) }).catch(() => {});
  }
  #updateTodo(id: number, patch: Partial<Todo>) {
    this.todos = this.todos.map((t) => (t.id === id ? { ...t, ...patch } : t));
    this.#saveTodos();
  }
  addTodo(text: string, due: number | null) {
    const t = text.trim();
    if (!t) return;
    this.todos = [...this.todos, { id: Date.now(), text: t, done: false, due, reminded: false }];
    this.#saveTodos();
  }
  toggleTodo(id: number) {
    const t = this.todos.find((x) => x.id === id);
    if (t) this.#updateTodo(id, { done: !t.done });
  }
  removeTodo(id: number) {
    this.todos = this.todos.filter((t) => t.id !== id);
    this.#saveTodos();
  }

  startTimer(minutes: number) {
    this.timerTotal = minutes * 60;
    this.timerEnd = Date.now() + this.timerTotal * 1000;
  }
  cancelTimer() {
    this.timerEnd = null;
  }

  control = (action: "toggle" | "next" | "prev") => invoke("media_cmd", { action });
  seek = (fraction: number) => {
    if (!this.media || this.media.length <= 0) return; // live stream: nothing to seek in
    const target = fraction * this.media.length;
    return invoke("media_seek", { offsetUs: Math.round(target - this.media.position) });
  };
}

export const island = new Island();

/** Click-through: only the pill (plus its side flares) receives input. Not a pixel below it, or browser tabs under the notch stop working. */
export function setHitRect(size: Size) {
  return invoke("set_hit_rect", { w: Math.ceil(size.w) + 24, h: Math.ceil(size.h) }).catch(() => {});
}

export const fmt = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
