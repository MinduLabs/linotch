use tauri::{AppHandle, Manager, PhysicalPosition};

/// Fixed window size (logical px): big enough for the largest pill plus spring overshoot.
/// It never resizes, because resizing a transparent X11 window mid-animation glitches.
pub const WIDTH: f64 = 420.0;
pub const HEIGHT: f64 = 180.0;

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let Some(w) = app.get_webview_window("main") else {
        return Ok(());
    };
    // A DOCK-typed window isn't clamped below the GNOME top bar, so the pill can sit at y=0.
    #[cfg(target_os = "linux")]
    if let Ok(gw) = w.gtk_window() {
        use gtk::prelude::{GtkWindowExt, WidgetExt, WidgetExtManual};
        gw.hide();
        gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
        gw.set_keep_above(true);
        gw.show_all();

        // WebKit regularly misses the pointer leaving through the edge of the input region: the DOM never gets
        // mouseleave and the island stays open until the pointer comes back. GTK's own crossing events on the
        // toplevel are reliable, so forward them. (Inferior = moving into a child window of ours, not a real exit.)
        use gtk::gdk::{EventMask, NotifyType};
        use tauri::Emitter;
        gw.add_events(EventMask::ENTER_NOTIFY_MASK | EventMask::LEAVE_NOTIFY_MASK);
        let a = app.clone();
        gw.connect_leave_notify_event(move |_, ev| {
            if ev.detail() != NotifyType::Inferior {
                let _ = a.emit("pointer-left", ());
            }
            gtk::glib::Propagation::Proceed
        });
        let a = app.clone();
        gw.connect_enter_notify_event(move |_, ev| {
            if ev.detail() != NotifyType::Inferior {
                let _ = a.emit("pointer-entered", ev.position());
            }
            gtk::glib::Propagation::Proceed
        });
    }
    #[cfg(target_os = "linux")]
    let _ = w.with_webview(|wv| tune_webkit(&wv.inner()));
    w.set_size(tauri::LogicalSize::new(WIDTH, HEIGHT))?;
    if let Some(m) = w.primary_monitor()? {
        let sf = m.scale_factor();
        let x = m.position().x + (m.size().width as i32 - (WIDTH * sf) as i32) / 2;
        w.set_position(PhysicalPosition::new(x, m.position().y))?;
    }
    w.set_always_on_top(true)?;
    w.set_skip_taskbar(true)?;
    Ok(())
}

/// WebKit defaults to a full web browser profile. The island is one small local page, so drop the browser
/// caches (WebKitGTK docs: DOCUMENT_VIEWER "can substantially reduce memory use" for apps without browsing)
/// and features it never uses.
#[cfg(target_os = "linux")]
fn tune_webkit(view: &webkit2gtk::WebView) {
    use webkit2gtk::{CacheModel, SettingsExt, WebContextExt, WebViewExt};
    if let Some(ctx) = view.context() {
        ctx.set_cache_model(CacheModel::DocumentViewer);
    }
    if let Some(s) = WebViewExt::settings(view) {
        s.set_enable_page_cache(false); // back/forward cache: there is no navigation
        s.set_enable_webgl(false);
        s.set_enable_developer_extras(cfg!(debug_assertions));
    }
}

/// Limits the input region to the pill (`w` x `h`, logical px, centred on the top edge) so the
/// transparent rest of the window lets clicks through to what is underneath.
#[tauri::command]
pub fn set_hit_rect(window: tauri::WebviewWindow, w: i32, h: i32) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::WidgetExt;
        let gw = window.gtk_window().map_err(|e| e.to_string())?;
        if let Some(gdk_win) = gw.window() {
            let x = (WIDTH as i32 - w) / 2;
            let region = gtk::cairo::Region::create_rectangle(&gtk::cairo::RectangleInt::new(x, 0, w, h));
            gdk_win.input_shape_combine_region(&region, 0, 0);
        }
    }
    let _ = (window, w, h);
    Ok(())
}
