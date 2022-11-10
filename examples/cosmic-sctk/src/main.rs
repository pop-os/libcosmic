use cosmic::{iced::{Application, sctk_settings::InitialSurface}, settings};

mod window;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.initial_surface = InitialSurface::XdgWindow(Default::default());
    Window::run(settings)
}
