mod dbus;
mod graphics;
mod window;

use cosmic::{
    iced::{sctk_settings::InitialSurface, Application},
    iced_native::window::Settings,
    iced_native::command::platform_specific::wayland::window::SctkWindowSettings,
    settings,
};
use cosmic_panel_config::PanelSize;
use window::*;

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    let pixels = std::env::var("COSMIC_PANEL_SIZE")
        .ok()
        .and_then(|size| match size.parse::<PanelSize>() {
            Ok(PanelSize::XL) => Some(64),
            Ok(PanelSize::L) => Some(48),
            Ok(PanelSize::M) => Some(36),
            Ok(PanelSize::S) => Some(24),
            Ok(PanelSize::XS) => Some(18),
            Err(_) => Some(36),
        })
        .unwrap_or(36);
    settings.initial_surface = InitialSurface::XdgWindow(SctkWindowSettings {
        iced_settings: Settings {
            size: (pixels + 32, pixels + 16),
            min_size: Some((pixels + 32, pixels + 16)),
            max_size: Some((pixels + 32, pixels + 16)),
            ..Default::default()
        },
        ..Default::default()
    });
    Window::run(settings)
}
