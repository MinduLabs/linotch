use serde::Serialize;
use std::{collections::HashMap, sync::Mutex, thread, time::Duration};
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
    #[serde(flatten)]
    device: Dev,
}

#[derive(Serialize, Clone, PartialEq)]
pub struct Dev {
    name: String,
    /// BlueZ icon name, e.g. "audio-headset", "input-keyboard"
    icon: String,
    /// "airpods" | "airpods-pro" | "airpods-max" for Apple headphones, so the UI can draw them; None otherwise.
    model: Option<&'static str>,
    #[serde(skip)]
    connected: bool,
    battery: Option<u8>,
}

/// Devices connected right now, for the UI to ask for when it opens (events only fire on change).
static CONNECTED: Mutex<Vec<Dev>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn bt_devices() -> Vec<Dev> {
    CONNECTED.lock().unwrap().clone()
}

/// Apple headphones by Bluetooth product id (vendor 0x004C). Ids from The Apple Wiki, "Bluetooth PIDs".
/// The name is the fallback, since a renamed pair still says "AirPods" in most cases and some ids are missing here.
fn apple_model(modalias: Option<&str>, name: &str) -> Option<&'static str> {
    let by_id = modalias.and_then(|m| {
        // "bluetooth:v004Cp2014d0F12" -> vendor 004C, product 2014
        let rest = m.split(':').nth(1)?;
        let vendor = rest.strip_prefix('v')?.get(..4)?;
        let product = rest.get(5..)?.strip_prefix('p')?.get(..4)?;
        if !vendor.eq_ignore_ascii_case("004C") {
            return None;
        }
        match u16::from_str_radix(product, 16).ok()? {
            0x2002 | 0x200F | 0x2013 | 0x201B => Some("airpods"),
            0x200E | 0x2014 | 0x2024 => Some("airpods-pro"),
            0x200A | 0x201F => Some("airpods-max"),
            _ => None,
        }
    });
    by_id.or_else(|| {
        let n = name.to_ascii_lowercase();
        if n.contains("airpods max") {
            Some("airpods-max")
        } else if n.contains("airpods pro") {
            Some("airpods-pro")
        } else if n.contains("airpods") {
            Some("airpods")
        } else {
            None
        }
    })
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
        let name = s("Alias").or_else(|| s("Name")).unwrap_or_else(|| "Bluetooth device".into());
        out.insert(
            path.to_string(),
            Dev {
                model: apple_model(s("Modalias").as_deref(), &name),
                name,
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
                let mut connected: Vec<Dev> = now.values().filter(|d| d.connected).cloned().collect();
                connected.sort_by(|a, b| a.name.cmp(&b.name));
                *CONNECTED.lock().unwrap() = connected;

                if let Some(before) = &prev {
                    for (path, d) in &now {
                        let was = before.get(path);
                        let kind = match (was.map(|w| w.connected).unwrap_or(false), d.connected) {
                            (false, true) => "connected",
                            (true, false) => "disconnected",
                            (true, true) if was.map(|w| w.battery) != Some(d.battery) => "battery",
                            _ => continue,
                        };
                        let _ = app.emit("bluetooth", &BtEvent { kind, device: d.clone() });
                    }
                }
                prev = Some(now);
            }
            thread::sleep(Duration::from_millis(1500));
        }
    });
}
