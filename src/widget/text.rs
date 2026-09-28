use crate::{Renderer, config::scaled_text_size};
pub use iced::widget::Text;
use iced_core::text::LineHeight;
use std::borrow::Cow;

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    Text::new(text.into()).font(crate::font::default())
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
pub fn title1<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(35.0))
            .line_height(LineHeight::Absolute(scaled_text_size(52.0).into()))
            .font(crate::font::semibold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Title 2 typography preset.
pub fn title2<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(29.0))
            .line_height(LineHeight::Absolute(scaled_text_size(43.0).into()))
            .font(crate::font::semibold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Title 3 typography preset.
pub fn title3<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(24.0))
            .line_height(LineHeight::Absolute(scaled_text_size(36.0).into()))
            .font(crate::font::bold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Title 4 typography preset.
pub fn title4<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(20.0))
            .line_height(LineHeight::Absolute(scaled_text_size(30.0).into()))
            .font(crate::font::bold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Heading typography preset.
pub fn heading<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(crate::config::font_size())
            .line_height(LineHeight::Absolute(scaled_text_size(21.0).into()))
            .font(crate::font::bold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Caption Heading typography preset.
pub fn caption_heading<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(12.0))
            .line_height(LineHeight::Absolute(scaled_text_size(17.0).into()))
            .font(crate::font::semibold())
    }

    inner(text.into())
}

/// [`Text`] widget with the Body typography preset.
pub fn body<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(crate::config::font_size())
            .line_height(LineHeight::Absolute(scaled_text_size(21.0).into()))
            .font(crate::font::default())
    }

    inner(text.into())
}

/// [`Text`] widget with the Caption typography preset.
pub fn caption<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(scaled_text_size(12.0))
            .line_height(LineHeight::Absolute(scaled_text_size(17.0).into()))
            .font(crate::font::default())
    }

    inner(text.into())
}

/// [`Text`] widget with the Monotext typography preset.
pub fn monotext<'a>(text: impl Into<Cow<'a, str>> + 'a) -> Text<'a, crate::Theme, Renderer> {
    #[inline(never)]
    fn inner(text: Cow<str>) -> Text<crate::Theme, Renderer> {
        Text::new(text)
            .size(crate::config::font_size())
            .line_height(LineHeight::Absolute(scaled_text_size(20.0).into()))
            .font(crate::font::mono())
    }

    inner(text.into())
}
