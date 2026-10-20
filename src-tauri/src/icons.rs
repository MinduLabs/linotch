use base64::Engine;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

fn find(name: &str) -> Option<PathBuf> {
    let direct = PathBuf::from(name);
    if direct.is_absolute() {
        return direct.is_file().then_some(direct);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let roots = [
        home.join(".local/share/icons"),
        "/usr/share/icons".into(),
        "/var/lib/flatpak/exports/share/icons".into(),
        home.join(".local/share/flatpak/exports/share/icons"),
    ];
    let sizes = ["512x512", "256x256", "128x128", "96x96", "64x64", "48x48", "scalable"];
    for root in &roots {
        for theme in ["hicolor", "Yaru", "Adwaita"] {
            for size in sizes {
                for ext in ["png", "svg"] {
                    let p = root.join(theme).join(size).join("apps").join(format!("{name}.{ext}"));
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }
    for ext in ["png", "svg"] {
        let p = PathBuf::from(format!("/usr/share/pixmaps/{name}.{ext}"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Icon name or absolute path -> `data:` URI the webview can show, cached.
pub fn resolve(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().unwrap().get(name) {
        return hit.clone();
    }
    let uri = find(name).and_then(|p| {
        let bytes = fs::read(&p).ok().filter(|b| b.len() < 600_000)?;
        let mime = match p.extension().and_then(|e| e.to_str()) {
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            _ => return None,
        };
        Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    });
    cache.lock().unwrap().insert(name.to_string(), uri.clone());
    uri
}
