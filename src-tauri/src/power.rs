use serde::Serialize;
use std::{fs, thread, time::Duration};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, PartialEq)]
struct Power {
    percent: u8,
    charging: bool,
    plugged: bool,
}

fn read() -> Option<Power> {
    let mut percent = None;
    let mut plugged = false;
    let mut charging = false;
    for e in fs::read_dir("/sys/class/power_supply").ok()?.flatten() {
        let p = e.path();
        let kind = fs::read_to_string(p.join("type")).unwrap_or_default();
        match kind.trim() {
            "Battery" => {
                percent = fs::read_to_string(p.join("capacity")).ok()?.trim().parse().ok();
                charging = fs::read_to_string(p.join("status")).unwrap_or_default().trim() == "Charging";
            }
            "Mains" | "USB" => {
                plugged |= fs::read_to_string(p.join("online")).unwrap_or_default().trim() == "1";
            }
            _ => {}
        }
    }
    Some(Power { percent: percent?, charging, plugged: plugged || charging })
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let mut last: Option<Power> = None;
        loop {
            let now = read();
            if now != last {
                let _ = app.emit("power", &now);
                last = now;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
