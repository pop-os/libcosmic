use cosmic::{
    iced::{wayland::InitialSurface, Application},
    settings,
};

mod window;
pub use window::Window;

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.initial_surface = InitialSurface::XdgWindow(Default::default());
    Window::run(settings)
}
