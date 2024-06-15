mod window;

use cosmic::app::Settings;
use window::Window;

fn main() -> cosmic::iced::Result {
    let settings = Settings::default();

    cosmic::app::run::<Window>(settings, ())
}
