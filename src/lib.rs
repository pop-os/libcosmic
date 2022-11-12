pub use iced;
pub use iced_lazy;
pub use iced_native;
pub use iced_style;
pub use iced_winit;

#[cfg(feature = "applet")]
pub mod applet;
pub mod font;
pub mod theme;
pub mod widget;

pub use theme::Theme;
pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;

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
