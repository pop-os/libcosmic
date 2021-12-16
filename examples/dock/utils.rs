use gdk4::ContentProvider;
use gdk4::Display;
use gio::File;
use gio::Icon;
use gtk4 as gtk;
use gtk4::DragSource;
use gtk4::IconTheme;

use gtk::glib;
use gtk::prelude::*;

pub fn init_drag_controller(drag_controller: &DragSource, app_info: &gio::DesktopAppInfo) {
    if let Some(file) = app_info.filename() {
        let file = File::for_path(file);
        let provider = ContentProvider::for_value(&file.to_value());
        drag_controller.set_content(Some(&provider));
    }
    drag_controller.connect_drag_end(move |_self, _drag, delete_data| {
        dbg!("removing", delete_data);
    });
    let icon = app_info
        .icon()
        .unwrap_or(Icon::for_string("image-missing").expect("Failed to set default icon"));
    drag_controller.connect_drag_begin(glib::clone!(@weak icon, => move |_self, drag| {
        drag.set_selected_action(gdk4::DragAction::MOVE);
        // set drag source icon if possible...
        // gio Icon is not easily converted to a Paintable, but this seems to be the correct method
        if let Some(default_display) = &Display::default() {
            if let Some(icon_theme) = IconTheme::for_display(default_display) {
                if let Some(paintable_icon) = icon_theme.lookup_by_gicon(
                    &icon,
                    64,
                    1,
                    gtk4::TextDirection::None,
                    gtk4::IconLookupFlags::empty(),
                ) {
                    _self.set_icon(Some(&paintable_icon), 32, 32);
                }
            }
        }
    }));
}
