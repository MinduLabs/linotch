# Linotch roadmap

Planned features and known problems. Nothing here is implemented yet unless it says so.
Reference for the look and feel: [Nusa](https://rama2555.github.io/nusa/) (a notch for Windows; no source, but its feature list is clear).

Target platform: Ubuntu 24.04+ / GNOME 45+ (and newer). Ubuntu 22.04 (GNOME 42) is not a goal.

## 1. Tabs in the expanded view

Hovering the island opens it on the **Home** tab; a row of icons at the bottom switches tabs. Every tab uses the
same pill size (about 371 x 190), so switching tabs never morphs the pill. The do-not-disturb bell sits in the same row.

| Tab | Content |
|---|---|
| **Home** (default) | Music player when something is playing (cover, seek bar, previous / play / next). Otherwise a glance: clock, date, and chips for charging, connected Bluetooth devices, next to-do. |
| **Devices** | Battery row (charging or on battery, % and bar) and the list of connected Bluetooth devices with their battery. |
| **Timer** | Minutes stepper plus presets (5 / 10 / 25 / 45), Start. While running: countdown, progress bar, Cancel. Existing alarm keeps ringing until dismissed. |
| **To-do** | Task list (tick, delete), add field with an optional reminder time. At the due time an alert shows the task. Stored locally. |

Notes from a first attempt:

- The window is a dock window, which gets no keyboard focus. The to-do input only worked after calling the window's
  `setFocus()` on pointer-down (needs the `core:window:allow-set-focus` permission).
- The window height has to grow (about 220 px) to fit the taller expanded view.
- Battery and Bluetooth state must be readable on demand (commands), not only through change events: the UI can
  miss events that fire before it starts listening.

## 2. Resting state (not decided yet)

What the closed island shows when nothing is happening. A plain clock plus battery is **not** wanted by default.

Proposed priority for what the closed pill shows:

1. Ringing alarm
2. Microphone in use (call)
3. Running timer (countdown)
4. Music playing (cover and wave)
5. To-do due soon
6. Nothing: a plain black notch, or Nusa-style "eyes" that doze off and wave when the pointer comes back (to decide)

## 3. Alerts (already there, keep)

Notifications with the app's icon (click opens the app), volume, charger plugged / unplugged, Bluetooth connected /
disconnected with battery, microphone in use, timer finished.

## 4. Removed / not wanted

- **Download detection** (progress and "downloaded" alert for browser downloads): did not work well, to be removed.
- **File shelf**: not wanted.

## 5. Known problems

- Cover art is missing when the app starts while a track is already playing: it is sent once on a `media-art`
  event that the UI can miss. It should be re-sent when the UI is ready.
- Quitting the app can leave a second instance behind; cause not found yet.
- Clicks on browser tabs just under the notch were blocked at one point; needs a recheck after the input region
  changes.
- Hover open / close is unreliable on Ubuntu 22.04 (pointer enter / leave events); not planned to fix.

## 6. Maybe later (from Nusa)

- Synced lyrics (the notch splits in two, current line highlighted)
- Water reminder with a daily goal
- Brightness, Caps Lock and keyboard-layout overlays
- Pomodoro (work / break cycles) in the Timer tab
- Settings that are remembered (which alerts to show, resting style)
