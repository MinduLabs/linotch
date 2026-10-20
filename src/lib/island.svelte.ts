import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Media = { title: string; artist: string; art: string | null; playing: boolean; position: number; length: number; canPrev: boolean; canNext: boolean };
export type Alert =
  | { kind: "power"; percent: number; charging: boolean }
  | { kind: "volume"; percent: number; muted: boolean }
  | { kind: "notif"; app: string; summary: string; body: string; desktopEntry: string | null; icon: string | null }
  | { kind: "bluetooth"; name: string; icon: string; connected: boolean; battery: number | null }
  | { kind: "mic"; app: string }
  | { kind: "timer" };

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

  #mediaGone: ReturnType<typeof setTimeout> | undefined;
  #alertTimeout: ReturnType<typeof setTimeout> | undefined;
  #leaveTimeout: ReturnType<typeof setTimeout> | undefined;
  #lastPower: { plugged: boolean } | null = null;

  mode = $derived<Mode>(
    this.alert ? "alert" : this.hovering ? "expanded" : this.media || this.timerEnd || this.micApp ? "compact" : "idle",
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
        this.pushAlert({ kind: "timer" }, 6000);
      }
    }, 250);

    // Changing track briefly reports no metadata. Hold the last track for a moment so the
    // island doesn't flip to the clock (and resize) in between.
    await listen<Media | null>("media", (e) => {
      clearTimeout(this.#mediaGone);
      if (e.payload) this.media = e.payload;
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

  pushAlert(a: Alert, ms: number) {
    this.alert = a;
    clearTimeout(this.#alertTimeout);
    this.#alertTimeout = setTimeout(() => (this.alert = null), ms);
  }

  dismissAlert() {
    clearTimeout(this.#alertTimeout);
    this.alert = null;
  }

  // After clicking an alert the cursor is still over the pill; don't pop open the clock until it leaves.
  #suppressHover = false;

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
    if (!this.media) return;
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
