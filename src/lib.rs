pub use iced;
pub use iced_lazy;
pub use iced_native;
pub use iced_style;
pub use iced_winit;

pub mod font;
pub mod widget;

#[derive(Clone, Copy, Debug)]
pub enum WindowMsg {
    Close,
    Drag,
    Minimize,
    Maximize,
    ToggleSidebar,
}

pub fn settings<Flags: Default>() -> iced::Settings<Flags> {
    let mut settings = iced::Settings::default();
    settings.default_font = match font::FONT {
        iced::Font::Default => None,
        iced::Font::External { bytes, .. } => Some(bytes),
    };
    settings.default_text_size = 18;
    settings
}
