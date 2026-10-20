mod bt;
mod icons;
mod media;
mod mic;
mod notif;
mod power;
mod settings;
mod tray;
mod volume;
mod win;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // GNOME/Wayland has no layer-shell, and Wayland clients can't position
    // themselves or stay on top. XWayland gives us both.
    #[cfg(target_os = "linux")]
    if std::env::var_os("ISLAND_NATIVE_WAYLAND").is_none() {
        // SAFETY: called before any other thread exists.
        unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .setup(|app| {
            let handle = app.handle();
            settings::install_extension();
            settings::take_over_banners();
            tray::build(handle)?;
            win::setup(handle)?;
            media::spawn(handle.clone());
            notif::spawn(handle.clone());
            power::spawn(handle.clone());
            volume::spawn(handle.clone());
            bt::spawn(handle.clone());
            mic::spawn(handle.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            win::set_hit_rect,
            media::media_cmd,
            media::media_seek,
            notif::open_app
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                settings::restore_banners();
            }
        });
}
