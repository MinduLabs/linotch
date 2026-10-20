use base64::Engine;
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex, thread, time::Duration};
use tauri::{AppHandle, Emitter};
use zbus::{
    blocking::{Connection, Proxy},
    zvariant::OwnedValue,
};

const PREFIX: &str = "org.mpris.MediaPlayer2.";
static CURRENT: Mutex<Option<String>> = Mutex::new(None);

#[derive(Serialize, Clone, PartialEq, Default)]
struct Media {
    title: String,
    artist: String,
    art: Option<String>,
    playing: bool,
    /// microseconds
    position: i64,
    length: i64,
    can_prev: bool,
    can_next: bool,
}

fn art_to_data_uri(url: &str) -> Option<String> {
    let Some(path) = url.strip_prefix("file://") else {
        return Some(url.to_string());
    };
    let bytes = std::fs::read(path).ok()?;
    let mime = if path.ends_with(".png") { "image/png" } else { "image/jpeg" };
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn snapshot(conn: &Connection, art_cache: &mut HashMap<String, Option<String>>) -> zbus::Result<Option<Media>> {
    let dbus = Proxy::new(conn, "org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus")?;
    let names: Vec<String> = dbus.call("ListNames", &())?;

    let mut best: Option<(String, Media)> = None;
    for name in names.iter().filter(|n| n.starts_with(PREFIX)) {
        let p = Proxy::new(conn, name.as_str(), "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player")?;
        let Ok(status) = p.get_property::<String>("PlaybackStatus") else { continue };
        let md: HashMap<String, OwnedValue> = p.get_property("Metadata").unwrap_or_default();

        let title = md.get("xesam:title").and_then(|v| v.downcast_ref::<&str>().ok()).unwrap_or("").to_string();
        let artist = md
            .get("xesam:artist")
            .and_then(|v| v.try_clone().ok())
            .and_then(|v| Vec::<String>::try_from(v).ok())
            .map(|a| a.join(", "))
            .unwrap_or_default();
        let length = md
            .get("mpris:length")
            .and_then(|v| v.downcast_ref::<i64>().ok().or_else(|| v.downcast_ref::<u64>().ok().map(|x| x as i64)))
            .unwrap_or(0);
        let art = md
            .get("mpris:artUrl")
            .and_then(|v| v.downcast_ref::<&str>().ok())
            .and_then(|u| art_cache.entry(u.to_string()).or_insert_with(|| art_to_data_uri(u)).clone());

        let playing = status == "Playing";
        let m = Media {
            title,
            artist,
            art,
            playing,
            position: p.get_property("Position").unwrap_or(0),
            length,
            can_prev: p.get_property("CanGoPrevious").unwrap_or(false),
            can_next: p.get_property("CanGoNext").unwrap_or(false),
        };
        if playing || best.is_none() {
            let stop = playing;
            best = Some((name.clone(), m));
            if stop {
                break;
            }
        }
    }

    let found = best.as_ref().map(|(n, _)| n.clone());
    *CURRENT.lock().unwrap() = found;
    Ok(best.map(|(_, m)| m).filter(|m| !m.title.is_empty()))
}

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        let Ok(conn) = Connection::session() else { return };
        let mut cache = HashMap::new();
        let mut last: Option<Media> = None;
        loop {
            let now = snapshot(&conn, &mut cache).unwrap_or(None);
            if now != last {
                let _ = app.emit("media", &now);
                last = now;
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}

fn call(method: &str, args: &(impl serde::Serialize + zbus::zvariant::DynamicType)) -> zbus::Result<()> {
    let Some(name) = CURRENT.lock().unwrap().clone() else { return Ok(()) };
    let conn = Connection::session()?;
    let p = Proxy::new(&conn, name, "/org/mpris/MediaPlayer2", "org.mpris.MediaPlayer2.Player")?;
    p.call_method(method, args)?;
    Ok(())
}

#[tauri::command]
pub fn media_cmd(action: String) -> Result<(), String> {
    let method = match action.as_str() {
        "toggle" => "PlayPause",
        "next" => "Next",
        "prev" => "Previous",
        _ => return Err("unknown action".into()),
    };
    call(method, &()).map_err(|e| e.to_string())
}

/// Relative seek, in microseconds.
#[tauri::command]
pub fn media_seek(offset_us: i64) -> Result<(), String> {
    call("Seek", &offset_us).map_err(|e| e.to_string())
}
