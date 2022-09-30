pub use iced;

pub mod font;
pub mod widget;

pub fn settings<Flags: Default>() -> iced::Settings<Flags> {
    let mut settings = iced::Settings::default();
    settings.default_font = match font::FONT {
        iced::Font::Default => None,
        iced::Font::External { bytes, .. } => Some(bytes),
    };
    settings.default_text_size = 18;
    settings
}
