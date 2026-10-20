import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Media = { title: string; artist: string; art: string | null; artKey: string; playing: boolean; position: number; length: number; canPrev: boolean; canNext: boolean };
export type Alert =
  | { kind: "power"; percent: number; charging: boolean }
  | { kind: "volume"; percent: number; muted: boolean }
  | { kind: "notif"; app: string; summary: string; body: string; desktopEntry: string | null; icon: string | null }
  | { kind: "bluetooth"; name: string; icon: string; connected: boolean; battery: number | null }
  | { kind: "mic"; app: string }
  | { kind: "download"; name: string }
  | { kind: "timer" };

export type Download = { name: string; size: number; speed: number };

export type Mode = "idle" | "compact" | "expanded" | "alert";
export type Size = { w: number; h: number; r: number };

// Logical px. The OS window is a fixed 460x230; only the pill moves.
export const WINDOW_W = 460;
const SIZES = {
  idle: { w: 190, h: 27, r: 10 }, // MacBook notch
  compact: { w: 250, h: 27, r: 13 },
  alertSmall: { w: 300, h: 36, r: 18 },
  alertBt: { w: 340, h: 40, r: 20 },
  alertNotif: { w: 371, h: 84, r: 32 },
  expandedMedia: { w: 371, h: 160, r: 40 },
  expandedIdle: { w: 371, h: 118, r: 36 },
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
  /** Partial files currently growing in the download folder. */
  downloads = $state<Download[]>([]);

  #mediaGone: ReturnType<typeof setTimeout> | undefined;
  #art: { key: string; data: string | null } | null = null;
  #alertTimeout: ReturnType<typeof setTimeout> | undefined;
  #leaveTimeout: ReturnType<typeof setTimeout> | undefined;
  #lastPower: { plugged: boolean } | null = null;

  mode = $derived<Mode>(
    this.alert ? "alert" : this.hovering ? "expanded" : this.media || this.timerEnd || this.micApp || this.downloads.length ? "compact" : "idle",
  );

  size = $derived<Size>(
    this.mode === "alert"
      ? this.alert?.kind === "notif" || this.alert?.kind === "timer"
        ? SIZES.alertNotif
        : this.alert?.kind === "bluetooth"
          ? SIZES.alertBt
          : SIZES.alertSmall
      : this.mode === "expanded"
        ? this.media
          ? SIZES.expandedMedia
          : SIZES.expandedIdle
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
    this.hovering = false;
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
    }, 250);

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
    await listen<Download[]>("downloads", (e) => (this.downloads = e.payload));
    await listen<string>("download-done", (e) => this.pushAlert({ kind: "download", name: e.payload }, 3500));
    await listen<{ kind: string; name: string; icon: string; battery: number | null }>("bluetooth", (e) => {
      const { kind, name, icon, battery } = e.payload;
      const cur = this.alert;
      if (kind === "battery") {
        // Battery levels usually arrive just after the connect event: update the open alert in place.
        if (cur?.kind === "bluetooth" && cur.name === name) this.alert = { ...cur, battery };
        return;
      }
      this.pushAlert({ kind: "bluetooth", name, icon, connected: kind === "connected", battery }, 3500);
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
      this.hovering = false;
    }
  }

  enter() {
    clearTimeout(this.#leaveTimeout);
    if (!this.#suppressHover) this.hovering = true;
  }
  leave() {
    clearTimeout(this.#leaveTimeout);
    this.#suppressHover = false;
    this.#leaveTimeout = setTimeout(() => (this.hovering = false), 350);
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

export const fmtBytes = (n: number) => {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (n >= 1000 && i < units.length - 1) {
    n /= 1000;
    i++;
  }
  return `${n >= 100 || i === 0 ? Math.round(n) : n.toFixed(1)} ${units[i]}`;
};
