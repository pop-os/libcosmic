mod deref_cell;
#[cfg(feature = "layer-shell")]
pub mod wayland;
#[cfg(feature = "layer-shell")]
mod wayland_custom_surface;
#[cfg(feature = "x")]
pub mod x;

#[cfg(feature = "widgets")]
pub use libcosmic_widgets as widgets;

use adw::StyleManager;
use gtk4::{
    gdk,
    gio::{self, FileMonitor, FileMonitorEvent, FileMonitorFlags},
    glib,
    prelude::*,
};

pub fn init() -> (Option<FileMonitor>, Option<FileMonitor>) {
    let _ = gtk4::init();
    adw::init();

    let gtk_user_provider = gtk4::CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk4::StyleContext::add_provider_for_display(
            &display,
            &gtk_user_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    let cosmic_user_provider = gtk4::CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk4::StyleContext::add_provider_for_display(
            &display,
            &cosmic_user_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let path = xdg::BaseDirectories::with_prefix("gtk-4.0")
        .ok()
        .and_then(|xdg_dirs| xdg_dirs.find_config_file("gtk.css"))
        .unwrap_or_else(|| "~/.config/gtk-4.0/gtk.css".into());
    let gtk_file = gio::File::for_path(path);
    let gtk_css_monitor = gtk_file.monitor(FileMonitorFlags::all(), None::<&gio::Cancellable>).ok().map(|monitor| {
        monitor.connect_changed(glib::clone!(@strong gtk_user_provider => move |_monitor, file, _other_file, event| {
            match event {
                FileMonitorEvent::Deleted | FileMonitorEvent::MovedOut | FileMonitorEvent::Renamed => {
                    if adw::is_initialized() {
                        let manager = StyleManager::default();
                        let css = if manager.is_dark() {
                            adw_user_colors_lib::colors::ColorOverrides::dark_default().as_gtk_css()
                        } else {
                            adw_user_colors_lib::colors::ColorOverrides::light_default().as_gtk_css()
                        };
                        gtk_user_provider
                            .load_from_data(css.as_bytes());
                    }
                },
                FileMonitorEvent::ChangesDoneHint | FileMonitorEvent::Created | FileMonitorEvent::MovedIn => {
                    gtk_user_provider.load_from_file(file);
                },
                _ => {} // ignored
            }
        }));
        monitor
    });
    let path = xdg::BaseDirectories::with_prefix("gtk-4.0")
    .ok()
    .and_then(|xdg_dirs| xdg_dirs.find_config_file("cosmic.css"))
    .unwrap_or_else(|| "~/.config/gtk-4.0/cosmic.css".into());

    let cosmic_file = gio::File::for_path(path);
    cosmic_user_provider.load_from_file(&cosmic_file);
    let cosmic_css_monitor = cosmic_file.monitor(FileMonitorFlags::all(), None::<&gio::Cancellable>).ok().map(|monitor| {
            monitor.connect_changed(glib::clone!(@strong cosmic_user_provider => move |_monitor, file, _other_file, event| {
                match event {
                    FileMonitorEvent::Deleted | FileMonitorEvent::MovedOut | FileMonitorEvent::Renamed => {
                        if adw::is_initialized() {
                            let manager = StyleManager::default();
                            let css = if manager.is_dark() {
                                adw_user_colors_lib::colors::ColorOverrides::dark_default().as_gtk_css()
                            } else {
                                adw_user_colors_lib::colors::ColorOverrides::light_default().as_gtk_css()
                            };
                            cosmic_user_provider
                                .load_from_data(css.as_bytes());
                        }
                    },
                    FileMonitorEvent::ChangesDoneHint | FileMonitorEvent::Created | FileMonitorEvent::MovedIn => {
                        cosmic_user_provider.load_from_file(file);
                    },
                    _ => {} // ignored
                }
            }));
            monitor
        });
    (gtk_css_monitor, cosmic_css_monitor)
}
