use std::borrow::Cow;

pub use iced::widget::Text;

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<'a, Renderer>(text: impl Into<Cow<'a, str>>) -> Text<'a, Renderer>
where
    Renderer: iced_core::text::Renderer,
    Renderer::Theme: iced::widget::text::StyleSheet,
{
    Text::new(text)
}
