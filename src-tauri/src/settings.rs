//! Things the tray menu toggles: autostart, the GNOME top-bar helper extension, and GNOME's own banners.
use std::{fs, path::PathBuf, process::Command};

const UUID: &str = "linotch-helper@mindulabs";
/// Names used before the project was called Linotch; cleaned up so upgrades leave nothing behind.
const OLD_UUID: &str = "island-helper@mbw";
const EXT_JS: &str = include_str!("../../gnome-extension/extension.js");
const EXT_META: &str = include_str!("../../gnome-extension/metadata.json");

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ---------- autostart ----------

fn autostart_file() -> PathBuf {
    home().join(".config/autostart/linotch.desktop")
}

pub fn autostart() -> bool {
    autostart_file().exists()
}

pub fn set_autostart(on: bool) -> std::io::Result<()> {
    let path = autostart_file();
    if !on {
        return match fs::remove_file(path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    // Inside an AppImage the running binary is a temporary mount; $APPIMAGE is the file to start.
    let exec = std::env::var("APPIMAGE")
        .ok()
        .or_else(|| std::env::current_exe().ok().map(|p| p.to_string_lossy().into_owned()))
        .unwrap_or_default();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(
        path,
        format!(
            "[Desktop Entry]\nType=Application\nName=Linotch\nComment=Dynamic Island for Linux\nExec=\"{exec}\"\nX-GNOME-Autostart-enabled=true\nTerminal=false\n"
        ),
    )
}

// ---------- GNOME top bar helper ----------

/// Copies the bundled extension into the user's extensions dir (refreshed when it changed).
pub fn install_extension() {
    let _ = fs::remove_dir_all(home().join(".local/share/gnome-shell/extensions").join(OLD_UUID));
    let _ = fs::remove_file(home().join(".config/autostart/island.desktop"));
    let dir = home().join(".local/share/gnome-shell/extensions").join(UUID);
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    for (name, content) in [("extension.js", EXT_JS), ("metadata.json", EXT_META)] {
        let path = dir.join(name);
        if fs::read_to_string(&path).ok().as_deref() != Some(content) {
            let _ = fs::write(path, content);
        }
    }
}

fn enabled_list() -> Vec<String> {
    run("gsettings", &["get", "org.gnome.shell", "enabled-extensions"])
        .map(|s| {
            s.trim_matches(|c| c == '[' || c == ']' || c == ' ')
                .split(',')
                .map(|x| x.trim().trim_matches('\'').to_string())
                .filter(|x| !x.is_empty() && !x.starts_with('@') && x != OLD_UUID)
                .collect()
        })
        .unwrap_or_default()
}

pub fn topbar() -> bool {
    enabled_list().iter().any(|e| e == UUID)
}

/// Returns true when GNOME Shell already knows the extension (so the change is live);
/// false means it only gets picked up after the next login.
pub fn set_topbar(on: bool) -> bool {
    install_extension();
    let mut list = enabled_list();
    list.retain(|e| e != UUID);
    if on {
        list.push(UUID.to_string());
    }
    let value = format!("[{}]", list.iter().map(|e| format!("'{e}'")).collect::<Vec<_>>().join(", "));
    let _ = run("gsettings", &["set", "org.gnome.shell", "enabled-extensions", &value]);

    let known = run("gnome-extensions", &["info", UUID]).is_some();
    if known {
        let _ = run("gnome-extensions", &[if on { "enable" } else { "disable" }, UUID]);
    }
    known
}

// ---------- GNOME's own notification banners ----------
// While Island runs it owns notifications, so GNOME's banners are switched off and restored on exit.

fn banners_marker() -> PathBuf {
    home().join(".config/linotch/banners-were-on")
}

fn set_banners(on: bool) {
    let _ = run("gsettings", &["set", "org.gnome.desktop.notifications", "show-banners", if on { "true" } else { "false" }]);
}

pub fn take_over_banners() {
    let on = run("gsettings", &["get", "org.gnome.desktop.notifications", "show-banners"]).as_deref() == Some("true");
    if on {
        let marker = banners_marker();
        let _ = fs::create_dir_all(marker.parent().unwrap());
        let _ = fs::write(marker, "");
        set_banners(false);
    }
}

pub fn restore_banners() {
    let legacy = home().join(".config/island/banners-were-on");
    if fs::remove_file(banners_marker()).is_ok() | fs::remove_file(legacy).is_ok() {
        set_banners(true);
    }
}
