/* GNOME Shell 42 (Ubuntu 22.04). The top bar is drawn above client windows, so to let the
 * Linotch window show in the middle we empty the centre, make the bar transparent and let
 * pointer events fall through the bar's empty space. */
const Main = imports.ui.main;

class LinotchHelper {
    enable() {
        const panel = Main.panel;
        this._dm = panel.statusArea.dateMenu;
        this._wasReactive = panel.reactive;

        panel._centerBox.remove_child(this._dm.container);
        panel._rightBox.insert_child_at_index(this._dm.container, 0);

        panel.set_style('background-color: transparent; background-gradient-direction: none;');
        for (const c of [panel._leftCorner, panel._rightCorner]) c?.hide();
        panel.reactive = false; // children (menus, tray icons) stay clickable
    }

    disable() {
        const panel = Main.panel;
        panel.reactive = this._wasReactive;
        panel.set_style(null);
        for (const c of [panel._leftCorner, panel._rightCorner]) c?.show();

        panel._rightBox.remove_child(this._dm.container);
        panel._centerBox.add_child(this._dm.container);
        this._dm = null;
    }
}

function init() {
    return new LinotchHelper();
}
