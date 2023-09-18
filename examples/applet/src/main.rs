use crate::window::Window;

mod window;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Window>(true, ())
}
