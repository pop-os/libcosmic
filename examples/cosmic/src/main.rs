use cosmic::{iced::Application, settings};

mod window;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    // TODO: Window resize handles not functioning yet
    settings.window.decorations = false;
    Window::run(settings)
}
