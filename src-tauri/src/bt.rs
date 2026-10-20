use serde::Serialize;
use std::{collections::HashMap, thread, time::Duration};
use tauri::{AppHandle, Emitter};
use zbus::{
    blocking::{Connection, Proxy},
    zvariant::{OwnedObjectPath, OwnedValue},
};

type Managed = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

#[derive(Serialize, Clone)]
struct BtEvent {
    /// "connected" | "disconnected" | "battery"
    kind: &'static str,
    name: String,
    /// BlueZ icon name, e.g. "audio-headphones", "input-keyboard"
    icon: String,
    battery: Option<u8>,
}

#[derive(Clone, PartialEq)]
struct Dev {
    name: String,
    icon: String,
    connected: bool,
    battery: Option<u8>,
}

fn read(conn: &Connection) -> Option<HashMap<String, Dev>> {
    let dbus = Proxy::new(conn, "org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus").ok()?;
    // Don't call BlueZ unless it is already running: that would try to activate it and block.
    if !dbus.call::<_, _, bool>("NameHasOwner", &"org.bluez").ok()? {
        return None;
    }
    let om = Proxy::new(conn, "org.bluez", "/", "org.freedesktop.DBus.ObjectManager").ok()?;
    let objs: Managed = om.call("GetManagedObjects", &()).ok()?;

    let mut out = HashMap::new();
    for (path, ifaces) in objs {
        let Some(d) = ifaces.get("org.bluez.Device1") else { continue };
        let s = |k: &str| d.get(k).and_then(|v| v.downcast_ref::<&str>().ok()).map(str::to_string);
        out.insert(
            path.to_string(),
            Dev {
                name: s("Alias").or_else(|| s("Name")).unwrap_or_else(|| "Bluetooth device".into()),
                icon: s("Icon").unwrap_or_default(),
                connected: d.get("Connected").and_then(|v| v.downcast_ref::<bool>().ok()).unwrap_or(false),
                battery: ifaces
                    .get("org.bluez.Battery1")
                    .and_then(|b| b.get("Percentage"))
                    .and_then(|v| v.downcast_ref::<u8>().ok()),
            },
        );
    }
    Some(out)
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let Ok(conn) = Connection::system() else { return };
        let mut prev: Option<HashMap<String, Dev>> = None;
        loop {
            if let Some(now) = read(&conn) {
                if let Some(before) = &prev {
                    for (path, d) in &now {
                        let was = before.get(path);
                        let kind = match (was.map(|w| w.connected).unwrap_or(false), d.connected) {
                            (false, true) => "connected",
                            (true, false) => "disconnected",
                            (true, true) if was.map(|w| w.battery) != Some(d.battery) => "battery",
                            _ => continue,
                        };
                        let _ = app.emit("bluetooth", &BtEvent { kind, name: d.name.clone(), icon: d.icon.clone(), battery: d.battery });
                    }
                }
                prev = Some(now);
            }
            thread::sleep(Duration::from_millis(1500));
        }
    });
}
