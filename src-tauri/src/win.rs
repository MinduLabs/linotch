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
        use gtk::prelude::{GtkWindowExt, WidgetExt};
        gw.hide();
        gw.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
        gw.set_keep_above(true);
        gw.show_all();
    }
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
