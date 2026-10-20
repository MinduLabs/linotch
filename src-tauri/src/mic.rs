use serde::Serialize;
use std::{collections::HashSet, process::Command, thread, time::Duration};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, PartialEq)]
struct Mic {
    app: String,
}

fn pactl(args: &[&str]) -> String {
    Command::new("pactl")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// The app currently recording from a real microphone (e.g. Meet/Zoom in a browser), if any.
/// Streams reading a ".monitor" source (screen/system audio capture) don't count.
fn recording_app() -> Option<String> {
    let monitors: HashSet<String> = pactl(&["list", "short", "sources"])
        .lines()
        .filter_map(|l| {
            let mut c = l.split('\t');
            let (idx, name) = (c.next()?, c.next()?);
            name.ends_with(".monitor").then(|| idx.to_string())
        })
        .collect();

    for block in pactl(&["list", "source-outputs"]).split("Source Output #").skip(1) {
        let source = block.lines().find_map(|l| l.trim().strip_prefix("Source: ")).unwrap_or("").trim();
        if monitors.contains(source) {
            continue;
        }
        let prop = |k: &str| {
            block.lines().find_map(|l| {
                l.trim().strip_prefix(k)?.trim_start().strip_prefix('=').map(|v| v.trim().trim_matches('"').to_string())
            })
        };
        let app = prop("application.name").or_else(|| prop("application.process.binary")).unwrap_or_default();
        if app.is_empty() || app == "speech-dispatcher" {
            continue;
        }
        return Some(app);
    }
    None
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut last: Option<Mic> = None;
        loop {
            let now = recording_app().map(|app| Mic { app });
            if now != last {
                let _ = app.emit("mic", &now);
                last = now;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
