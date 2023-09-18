use crate::window::Window;

mod window;

fn main() -> cosmic::iced::Result {
    cosmic::app::applet::run::<Window>(true, ())
}
