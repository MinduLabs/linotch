//! Running host programs (pactl, dbus-monitor, gsettings, ...) from inside the AppImage.
//!
//! The AppImage runtime points LD_LIBRARY_PATH, GIO_MODULE_DIR, PATH etc. at the libraries bundled from the
//! build distro. Child processes inherit that, so host binaries load the wrong libdbus/libgio and die
//! (dbus-monitor: "LIBDBUS_PRIVATE_1.16.2 not found"). Strip everything that points into the AppImage.
use std::{ffi::OsStr, process::Command};

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    let Some(appdir) = std::env::var_os("APPDIR").map(|d| d.to_string_lossy().trim_end_matches('/').to_string()) else {
        return cmd;
    };
    for (key, value) in std::env::vars_os() {
        let value = value.to_string_lossy();
        if !value.contains(&appdir) {
            continue;
        }
        let kept: Vec<&str> = value.split(':').filter(|p| !p.is_empty() && !p.starts_with(&appdir)).collect();
        if kept.is_empty() {
            cmd.env_remove(&key);
        } else {
            cmd.env(&key, kept.join(":"));
        }
    }
    cmd
}
