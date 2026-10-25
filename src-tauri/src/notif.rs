use crate::host;
use serde::Serialize;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};
use crate::icons;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Notification {
    app: String,
    summary: String,
    body: String,
    desktop_entry: Option<String>,
    /// data: URI of the sending app's icon
    icon: Option<String>,
}

/// Value of a `string "..."` or `variant string "..."` dbus-monitor line.
fn quoted(line: &str) -> Option<String> {
    let t = line.trim();
    let s = t.strip_prefix("variant").map(str::trim_start).unwrap_or(t).strip_prefix("string \"")?;
    Some(s.strip_suffix('"').unwrap_or(s).replace("\\\"", "\""))
}

/// Eavesdrops on org.freedesktop.Notifications.Notify via dbus-monitor.
/// Args: app_name, replaces_id, icon, summary, body, actions[], hints{}, expire_timeout.
pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let Ok(mut child) = host::command("dbus-monitor")
            .args(["interface='org.freedesktop.Notifications',member='Notify'"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let Some(out) = child.stdout.take() else { return };

        let mut collecting = false;
        let mut depth = 0i32;
        let mut strings: Vec<String> = Vec::new();
        let mut desktop_entry: Option<String> = None;
        let mut want_entry = false;
        // The shell re-sends each Notify to itself, so the same call shows up twice.
        let mut last: Option<(String, Instant)> = None;

        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if line.starts_with("method call") || line.starts_with("signal") || line.starts_with("method return") {
                collecting = line.contains("member=Notify");
                depth = 0;
                strings.clear();
                desktop_entry = None;
                want_entry = false;
                continue;
            }
            if !collecting {
                continue;
            }
            let t = line.trim();
            let is_string = quoted(t).is_some();

            if depth == 0 && is_string && strings.len() < 4 {
                strings.extend(quoted(t));
            } else if want_entry && is_string {
                desktop_entry = quoted(t);
                want_entry = false;
            } else if t == "string \"desktop-entry\"" {
                want_entry = true;
            }

            if !is_string {
                if t.ends_with('[') || t.ends_with('(') || t.ends_with('{') {
                    depth += 1;
                } else if t.starts_with(']') || t.starts_with(')') || t.starts_with('}') {
                    depth -= 1;
                }
            }

            // The last top-level argument (expire_timeout) closes the message.
            if depth == 0 && t.starts_with("int32") && strings.len() >= 4 {
                collecting = false;
                let key = format!("{}|{}|{}", strings[0], strings[2], strings[3]);
                if last.as_ref().is_some_and(|(k, at)| *k == key && at.elapsed() < Duration::from_millis(500)) {
                    continue;
                }
                last = Some((key, Instant::now()));
                if !crate::settings::notifications() {
                    continue;
                }
                let entry = desktop_entry.take();
                let icon = icons::resolve(&strings[1]).or_else(|| app_icon(entry.as_deref(), &strings[0]));
                let _ = app.emit(
                    "notification",
                    &Notification {
                        app: strings[0].clone(),
                        summary: strings[2].clone(),
                        body: strings[3].clone(),
                        desktop_entry: entry,
                        icon,
                    },
                );
            }
        }
    });
}

fn app_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    vec![
        "/usr/share/applications".into(),
        "/var/lib/snapd/desktop/applications".into(),
        "/var/lib/flatpak/exports/share/applications".into(),
        home.join(".local/share/applications"),
        home.join(".local/share/flatpak/exports/share/applications"),
    ]
}

/// Candidate .desktop ids for a notification, best first: the `desktop-entry` hint, then files
/// whose name, Name= or StartupWMClass= equals the app name, then looser substring matches.
fn desktop_candidates(entry: Option<&str>, app: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |id: String| {
        if !out.contains(&id) {
            out.push(id);
        }
    };
    if let Some(e) = entry.filter(|e| !e.is_empty()) {
        push(e.trim_end_matches(".desktop").to_string());
    }
    let want = app.to_lowercase();
    if want.is_empty() {
        return out;
    }
    let mut loose = Vec::new();
    for dir in app_dirs() {
        let Ok(rd) = fs::read_dir(dir) else { continue };
        for f in rd.flatten() {
            let path = f.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let content = fs::read_to_string(&path).unwrap_or_default();
            if content.lines().any(|l| l.trim() == "NoDisplay=true") {
                continue;
            }
            let field = |k: &str| content.lines().find_map(|l| l.strip_prefix(k)).map(str::to_lowercase);
            let stem_l = stem.to_lowercase();
            if stem_l == want || field("Name=").as_deref() == Some(&want) || field("StartupWMClass=").as_deref() == Some(&want) {
                push(stem.to_string());
            } else if stem_l.contains(&want) {
                loose.push(stem.to_string());
            }
        }
    }
    for id in loose {
        push(id);
    }
    out
}

/// Icon of the app behind a notification, read from its .desktop file.
fn app_icon(entry: Option<&str>, app: &str) -> Option<String> {
    for id in desktop_candidates(entry, app) {
        for dir in app_dirs() {
            let Ok(content) = fs::read_to_string(dir.join(format!("{id}.desktop"))) else { continue };
            if let Some(icon) = content.lines().find_map(|l| l.strip_prefix("Icon=")).and_then(|n| icons::resolve(n.trim())) {
                return Some(icon);
            }
        }
    }
    None
}

/// Opens (or focuses, for single-instance apps) the app that sent a notification.
#[tauri::command]
pub fn open_app(desktop_entry: Option<String>, app: String) -> Result<(), String> {
    let candidates = desktop_candidates(desktop_entry.as_deref(), &app);
    if candidates.is_empty() {
        return Err(format!("no desktop entry for {app}"));
    }
    // gtk-launch exits non-zero for an unknown id, so fall through to the next candidate.
    thread::spawn(move || {
        for id in candidates {
            if host::command("gtk-launch").arg(&id).status().is_ok_and(|s| s.success()) {
                break;
            }
        }
    });
    Ok(())
}
