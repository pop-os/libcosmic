use cosmic::iced::{wayland::InitialSurface, Application, Settings};

mod window;
pub use window::Window;

pub fn main() -> cosmic::iced::Result {
    cosmic::icon_theme::set_default("Pop");
    let mut settings = Settings::default();
    settings.initial_surface = InitialSurface::XdgWindow(Default::default());
    Window::run(settings)
}
