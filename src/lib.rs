mod deref_cell;
#[cfg(feature = "layer-shell")]
pub mod wayland;
#[cfg(feature = "layer-shell")]
mod wayland_custom_surface;
#[cfg(feature = "x")]
pub mod x;

use adw::StyleManager;
#[cfg(feature = "widgets")]
pub use libcosmic_widgets as widgets;

use gtk4::{gdk, gio::{self, FileMonitorFlags, FileMonitorEvent, FileMonitor}, glib, prelude::*};

pub fn init() -> Option<FileMonitor> {
    let _ = gtk4::init();
    adw::init();

    let user_provider = gtk4::CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk4::StyleContext::add_provider_for_display(
            &display,
            &user_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    let path = xdg::BaseDirectories::with_prefix("gtk-4.0")
    .ok()
    .and_then(|xdg_dirs| xdg_dirs.find_config_file("gtk.css"))
    .unwrap_or_else(|| "~/.config/gtk-4.0/gtk.css".into());
    let file = gio::File::for_path(path);
    if let Ok(monitor) = file.monitor(FileMonitorFlags::all(), None::<&gio::Cancellable>) {
        monitor.connect_changed(glib::clone!(@strong user_provider => move |_monitor, file, _other_file, event| {
            match event {
                FileMonitorEvent::Deleted | FileMonitorEvent::MovedOut | FileMonitorEvent::Renamed => {
                    if adw::is_initialized() {
                        let manager = StyleManager::default();
                        let css = if manager.is_dark() {
                            adw_user_colors_lib::colors::ColorOverrides::dark_default().as_css()
                        } else {
                            adw_user_colors_lib::colors::ColorOverrides::light_default().as_css()
                        };
                        user_provider
                            .load_from_data(css.as_bytes());
                    }
                },
                FileMonitorEvent::ChangesDoneHint | FileMonitorEvent::Created | FileMonitorEvent::MovedIn => {
                    user_provider.load_from_file(file);
                },
                _ => {} // ignored
            }
        }));
        Some(monitor)
    } else {
        None
    }
}