use crate::settings;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

fn autostart_label() -> &'static str {
    if settings::autostart() { "Disable auto start" } else { "Enable auto start" }
}

fn notch_label() -> &'static str {
    if settings::topbar() { "Disable notch bar" } else { "Enable notch bar" }
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let autostart = MenuItem::with_id(app, "autostart", autostart_label(), true, None::<&str>)?;
    let notch = MenuItem::with_id(app, "notch", notch_label(), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&notch, &autostart, &PredefinedMenuItem::separator(app)?, &quit])?;

    let (autostart_i, notch_i) = (autostart.clone(), notch.clone());
    TrayIconBuilder::with_id("linotch")
        .icon(tauri::include_image!("icons/tray.png"))
        .tooltip("Linotch")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "autostart" => {
                let _ = settings::set_autostart(!settings::autostart());
                let _ = autostart_i.set_text(autostart_label());
            }
            "notch" => {
                // The first time, GNOME only picks the extension up after the next login.
                settings::set_topbar(!settings::topbar());
                let _ = notch_i.set_text(notch_label());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
