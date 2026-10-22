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
#[serde(rename_all = "camelCase")]
struct Media {
    title: String,
    artist: String,
    /// Cover art as a data URI, often 100+ KB. Sent on its own "media-art" event only when it changes:
    /// shipping it with every 500 ms position update stalled the webview.
    #[serde(skip)]
    art: Option<String>,
    /// Which cover belongs to this track (the player's artUrl, "" for none). Lets the UI show its default
    /// cover instead of the previous track's while the new one is on its way.
    art_key: String,
    playing: bool,
    /// microseconds
    position: i64,
    length: i64,
    can_prev: bool,
    can_next: bool,
}

#[derive(Serialize)]
struct ArtMsg {
    key: String,
    data: Option<String>,
}

fn art_to_data_uri(url: &str) -> Option<String> {
    let Some(path) = url.strip_prefix("file://") else {
        return Some(url.to_string());
    };
    let bytes = std::fs::read(path).ok()?;
    let png = bytes.starts_with(b"\x89PNG\r\n\x1a\n");
    // Chromium-based browsers (Brave, Chrome) fill the gap while switching tracks with their own app icon
    // (seen: a 256x256 RGBA PNG, with an empty title). Real artwork arrives as an opaque PNG, so an image with
    // an alpha channel from Chromium's temp files is the browser logo: treat it as no art.
    // PNG colour type sits at byte 25 of the IHDR chunk; 4 and 6 are the types with alpha.
    if png && path.contains(".org.chromium.Chromium.") && matches!(bytes.get(25), Some(4 | 6)) {
        return None;
    }
    let mime = if png { "image/png" } else { "image/jpeg" };
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

/// The url, plus size and mtime for a local file, so a file rewritten under the same name counts as new art.
fn art_identity(url: &str) -> String {
    let Some(path) = url.strip_prefix("file://") else { return url.to_string() };
    let Ok(meta) = std::fs::metadata(path) else { return url.to_string() };
    let secs = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
    format!("{url}#{}-{secs}", meta.len())
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
        // Live streams report no length, 0, or a sentinel like i64::MAX. Normalise all of those to 0 = unknown.
        let length = if length <= 0 || length > 7 * 24 * 3600 * 1_000_000 { 0 } else { length };
        let art_url = md.get("mpris:artUrl").and_then(|v| v.downcast_ref::<&str>().ok());
        // Chromium rewrites the same temp file in place (first its own logo, later the real cover), so the url
        // alone is not a key: a stale "no art" would stick for the whole track. Add the file's size and mtime.
        let art_key = art_url.map(art_identity).unwrap_or_default();
        let art = art_url
            .and_then(|u| {
                // Browsers hand out a new temp file per track, so the cache would grow forever: keep a few.
                if !art_cache.contains_key(&art_key) && art_cache.len() >= 8 {
                    art_cache.clear();
                }
                art_cache.entry(art_key.clone()).or_insert_with(|| art_to_data_uri(u)).clone()
            });

        let playing = status == "Playing";
        let m = Media {
            title,
            artist,
            art,
            art_key,
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
        let mut last_key: Option<String> = None;
        loop {
            let now = snapshot(&conn, &mut cache).unwrap_or(None);
            let key = now.as_ref().map(|m| m.art_key.clone());
            if key != last_key {
                if let Some(m) = &now {
                    let _ = app.emit("media-art", &ArtMsg { key: m.art_key.clone(), data: m.art.clone() });
                }
                last_key = key;
            }
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
