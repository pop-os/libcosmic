use cosmic::{settings, iced::Application};
mod window;
use window::*;

fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    
    App::run(settings)
}
