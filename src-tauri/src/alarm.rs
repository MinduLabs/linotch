//! Timer alarm: rings the freedesktop "alarm-clock-elapsed" sound over and over until the alert is dismissed.
use crate::host;
use std::{
    process::Child,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const SOUND_ID: &str = "alarm-clock-elapsed";
const SOUND_FILE: &str = "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";
/// Stop ringing on its own after this long, like a phone alarm that times out.
const MAX_RING: Duration = Duration::from_secs(120);

/// Bumped on every start/stop; a ringing loop exits as soon as it no longer owns the current value.
static GENERATION: AtomicU64 = AtomicU64::new(0);
static PLAYING: Mutex<Option<Child>> = Mutex::new(None);

/// One play of the sound, with whichever player the system has.
fn play_once() -> Option<Child> {
    host::command("canberra-gtk-play")
        .args(["-i", SOUND_ID])
        .spawn()
        .or_else(|_| host::command("pw-play").arg(SOUND_FILE).spawn())
        .or_else(|_| host::command("paplay").arg(SOUND_FILE).spawn())
        .ok()
}

fn stop_current() {
    if let Some(mut child) = PLAYING.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[tauri::command]
pub fn alarm(on: bool) {
    let me = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    stop_current();
    if !on {
        return;
    }
    thread::spawn(move || {
        let started = Instant::now();
        while GENERATION.load(Ordering::SeqCst) == me && started.elapsed() < MAX_RING {
            let Some(child) = play_once() else { return };
            *PLAYING.lock().unwrap() = Some(child);
            // Wait for this play to end (or be killed by a stop), then a short pause before the next ring.
            loop {
                thread::sleep(Duration::from_millis(100));
                let mut guard = PLAYING.lock().unwrap();
                match guard.as_mut().map(|c| c.try_wait()) {
                    Some(Ok(None)) => continue,
                    _ => {
                        guard.take();
                        break;
                    }
                }
            }
            thread::sleep(Duration::from_millis(600));
        }
    });
}

/// A single soft "message" sound, for a to-do reminder (the timer alarm rings until dismissed instead).
#[tauri::command]
pub fn chime() {
    const FILE: &str = "/usr/share/sounds/freedesktop/stereo/message.oga";
    let _ = host::command("canberra-gtk-play")
        .args(["-i", "message"])
        .spawn()
        .or_else(|_| host::command("pw-play").arg(FILE).spawn())
        .or_else(|_| host::command("paplay").arg(FILE).spawn());
}
