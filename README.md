# Linotch

A Dynamic Island / MacBook-notch style pill for Linux, built with Tauri 2, Svelte 5 and Rust.
It sits at the top-centre of the screen, shrinks to a notch when idle and springs open for what is happening.

## Features

- **Now playing** (any MPRIS player: Spotify, browsers, VLC): cover art, seek bar, previous / play / next.
- **Notifications** shown inside the island with the sending app's icon; click one to open that app. GNOME's own banners are switched off while Linotch runs and restored on exit.
- **Alerts**: volume, charger plugged / unplugged, Bluetooth device connected / disconnected with battery level.
- **Microphone in use** (calls in Meet, Zoom, ...): orange dot and an alert naming the app.
- **Timer** with 1 / 5 / 10 / 25 minute presets and a clock when nothing else is going on.
- Do-not-disturb bell, tray menu with *Enable / Disable notch bar* and *Enable / Disable auto start*.

## Requirements

Developed on Ubuntu 22.04 (GNOME 42, Wayland). Needs `pactl` (PulseAudio / PipeWire), `dbus-monitor` and `gtk-launch`.
The window runs through XWayland (`GDK_BACKEND=x11`) because GNOME has no layer-shell.

The GNOME top bar is drawn above normal windows, so *Enable notch bar* installs a small extension
(`gnome-extension/`) that makes the bar transparent, moves the clock to the right and lets clicks pass
through. The first time, GNOME only loads it after you log out and in again. The extension targets
GNOME Shell 42 only for now.

## Development

```bash
pnpm install
pnpm tauri dev
```

Build an AppImage and a `.deb`:

```bash
NO_STRIP=true pnpm tauri build
```

Files end up in `src-tauri/target/release/bundle/`.

## Status

Early work in progress. Known gaps: GNOME 45+ extension, multiple monitors, camera detection.
