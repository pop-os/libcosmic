use crate::Renderer;
pub use iced::widget::Text;
use std::borrow::Cow;

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text)
}

/// Available presets for text typography
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Typography {
    Body,
    Caption,
    CaptionHeading,
    Heading,
    Monotext,
    Title1,
    Title2,
    Title3,
    Title4,
}

/// [`Text`] widget with the Title 1 typography preset.
pub fn title1<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text)
        .size(32.0)
        .line_height(44.0)
        .font(crate::font::FONT_LIGHT)
}

/// [`Text`] widget with the Title 2 typography preset.
pub fn title2<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text).size(28.0).line_height(36.0)
}

/// [`Text`] widget with the Title 3 typography preset.
pub fn title3<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text).size(24.0).line_height(32.0)
}

/// [`Text`] widget with the Title 4 typography preset.
pub fn title4<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text).size(20.0).line_height(28.0)
}

/// [`Text`] widget with the Heading typography preset.
pub fn heading<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text)
        .size(14.0)
        .line_height(20.0)
        .font(crate::font::FONT_SEMIBOLD)
}

/// [`Text`] widget with the Caption Heading typography preset.
pub fn caption_heading<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text)
        .size(10.0)
        .line_height(14.0)
        .font(crate::font::FONT_SEMIBOLD)
}

/// [`Text`] widget with the Body typography preset.
pub fn body<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text).size(14.0).line_height(20.0)
}

/// [`Text`] widget with the Caption typography preset.
pub fn caption<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text).size(10.0).line_height(14.0)
}

/// [`Text`] widget with the Monotext typography preset.
pub fn monotext<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, Renderer> {
    Text::new(text)
        .size(14.0)
        .line_height(20.0)
        .font(crate::font::FONT_MONO_REGULAR)
}
