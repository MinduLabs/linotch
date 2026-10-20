mod alarm;
mod bt;
mod downloads;
mod host;
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
    // Measured on the idle island (PSS of app + WebKit processes): 197 MB -> 174 MB.
    // - DMA-BUF renderer off: WebKit keeps GPU compositing but shares frames through shared memory (-20 MB).
    // - JavaScriptCore's optimizing JIT tiers off: the page runs a few lines per frame, the baseline JIT is plenty.
    //   (Turning the JIT off entirely saves another ~5 MB but makes per-frame JS slower; not worth it.)
    #[cfg(target_os = "linux")]
    for (key, value) in [("WEBKIT_DISABLE_DMABUF_RENDERER", "1"), ("JSC_useDFGJIT", "false"), ("JSC_useFTLJIT", "false")] {
        if std::env::var_os(key).is_none() {
            // SAFETY: still before any other thread exists.
            unsafe { std::env::set_var(key, value) };
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .setup(|app| {
            let handle = app.handle();
            settings::install_extension();
            settings::take_over_banners();
            // SIGINT/SIGTERM (logout, `kill`) must still go through RunEvent::Exit so GNOME's banners come back.
            if let Ok(mut signals) = signal_hook::iterator::Signals::new([signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM]) {
                let h = handle.clone();
                std::thread::spawn(move || {
                    if signals.forever().next().is_some() {
                        h.exit(0);
                    }
                });
            }
            tray::build(handle)?;
            win::setup(handle)?;
            media::spawn(handle.clone());
            notif::spawn(handle.clone());
            power::spawn(handle.clone());
            volume::spawn(handle.clone());
            bt::spawn(handle.clone());
            mic::spawn(handle.clone());
            downloads::spawn(handle.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            win::set_hit_rect,
            media::media_cmd,
            media::media_seek,
            notif::open_app,
            alarm::alarm
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                settings::restore_banners();
            }
        });
}
