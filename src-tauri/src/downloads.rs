//! Browser downloads, seen from outside: Chrome/Brave write `*.crdownload`, Firefox `*.part`, Opera `*.opdownload`
//! into the download folder while a file is coming in, then rename it. The browser does not tell us the total size,
//! so we only know bytes so far and speed.
use crate::host;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime},
};
use tauri::{AppHandle, Emitter};

const TMP_EXT: [&str; 3] = ["crdownload", "part", "opdownload"];
/// A partial file that has not grown for this long is paused/abandoned, not shown.
const STALL: Duration = Duration::from_secs(15);
const POLL: Duration = Duration::from_millis(700);

#[derive(Serialize, Clone, PartialEq)]
struct Item {
    name: String,
    /// bytes so far
    size: u64,
    /// bytes per second
    speed: u64,
}

struct Track {
    size: u64,
    sampled: Instant,
    changed: Instant,
    speed: f64,
}

fn download_dir() -> PathBuf {
    let from_xdg = host::command("xdg-user-dir")
        .arg("DOWNLOAD")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    from_xdg
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join("Downloads"))
}

/// File name without the temporary extension, if this is a partial download.
fn partial_stem(name: &str) -> Option<&str> {
    let (stem, ext) = name.rsplit_once('.')?;
    (TMP_EXT.contains(&ext) && !stem.is_empty()).then_some(stem)
}

/// Chrome names the partial file "Unconfirmed 123456.crdownload" until it knows the real name.
fn display_name(stem: &str) -> String {
    if stem.starts_with("Unconfirmed ") { "Downloading file".into() } else { stem.to_string() }
}

/// The finished file for a partial that just disappeared; None when the download was cancelled.
fn finished_name(dir: &PathBuf, stem: &str, since_change: Duration) -> Option<String> {
    if dir.join(stem).exists() {
        return Some(stem.to_string());
    }
    if !stem.starts_with("Unconfirmed ") {
        return None;
    }
    // Real name only known after the rename. A rename keeps the mtime, so the finished file is the one last
    // written when the partial file last grew; any other recent file is unrelated (the download was cancelled).
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let age = SystemTime::now().duration_since(e.metadata().ok()?.modified().ok()?).ok()?;
            let apart = if age > since_change { age - since_change } else { since_change - age };
            (e.path().is_file() && partial_stem(&name).is_none() && apart < Duration::from_secs(1)).then_some((age, name))
        })
        .min_by_key(|(age, _)| *age)
        .map(|(_, name)| name)
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let dir = download_dir();
        let mut tracks: HashMap<String, Track> = HashMap::new();
        let mut last: Vec<Item> = Vec::new();
        let mut first_scan = true;

        loop {
            let now = Instant::now();
            let mut seen = Vec::new();
            for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
                let file = entry.file_name().to_string_lossy().into_owned();
                if partial_stem(&file).is_none() {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                let size = meta.len();
                seen.push(file.clone());
                let t = tracks.entry(file).or_insert_with(|| Track {
                    size,
                    sampled: now,
                    // Partial files already lying around at startup count as stale until they grow.
                    changed: if first_scan { now - STALL * 4 } else { now },
                    speed: 0.0,
                });
                if size != t.size {
                    let dt = now.duration_since(t.sampled).as_secs_f64().max(0.001);
                    let inst = size.saturating_sub(t.size) as f64 / dt;
                    t.speed = if t.speed == 0.0 { inst } else { t.speed * 0.6 + inst * 0.4 };
                    t.size = size;
                    t.sampled = now;
                    t.changed = now;
                } else if now.duration_since(t.changed) > Duration::from_secs(2) {
                    t.speed = 0.0;
                }
            }
            first_scan = false;

            // Partial file gone: either renamed to the finished file, or cancelled.
            let gone: Vec<String> = tracks.keys().filter(|k| !seen.contains(k)).cloned().collect();
            for file in gone {
                let t = tracks.remove(&file).unwrap();
                let Some(stem) = partial_stem(&file) else { continue };
                if now.duration_since(t.changed) > STALL {
                    continue;
                }
                if let Some(name) = finished_name(&dir, stem, now.duration_since(t.changed)) {
                    let _ = app.emit("download-done", &name);
                }
            }

            let mut items: Vec<Item> = tracks
                .iter()
                .filter(|(_, t)| now.duration_since(t.changed) <= STALL)
                .map(|(file, t)| Item {
                    name: display_name(partial_stem(file).unwrap_or(file)),
                    size: t.size,
                    speed: t.speed as u64,
                })
                .collect();
            items.sort_by(|a, b| a.name.cmp(&b.name));
            if items != last {
                let _ = app.emit("downloads", &items);
                last = items;
            }
            thread::sleep(POLL);
        }
    });
}
