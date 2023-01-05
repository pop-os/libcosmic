use cosmic::{
    iced::{wayland::InitialSurface, Application},
    settings,
};

mod window;
pub use window::Window;

pub fn main() -> cosmic::iced::Result {
    settings::set_default_icon_theme("Pop");
    let mut settings = settings();
    settings.initial_surface = InitialSurface::XdgWindow(Default::default());
    Window::run(settings)
}
