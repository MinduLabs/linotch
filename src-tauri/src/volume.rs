use crate::host;
use serde::Serialize;
use std::{
    io::{BufRead, BufReader},
    process::Stdio,
    thread,
};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, PartialEq)]
struct Volume {
    percent: u32,
    muted: bool,
}

fn pactl(args: &[&str]) -> String {
    host::command("pactl")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn read() -> Option<Volume> {
    let vol = pactl(&["get-sink-volume", "@DEFAULT_SINK@"]);
    let percent = vol.split('/').nth(1)?.trim().trim_end_matches('%').trim().parse().ok()?;
    let muted = pactl(&["get-sink-mute", "@DEFAULT_SINK@"]).contains("yes");
    Some(Volume { percent, muted })
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let Ok(mut child) = host::command("pactl").arg("subscribe").stdout(Stdio::piped()).spawn() else {
            return;
        };
        let Some(out) = child.stdout.take() else { return };
        let mut last = read();
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if !line.contains("on sink #") || !line.contains("'change'") {
                continue;
            }
            let now = read();
            if now != last {
                if let Some(v) = &now {
                    let _ = app.emit("volume", v);
                }
                last = now;
            }
        }
    });
}
