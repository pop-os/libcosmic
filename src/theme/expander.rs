use iced_core::{Background, Color};

/// The appearance of a [`Expander`](crate::native::expander::Expander).
#[derive(Clone, Copy, Debug)]
pub struct Appearance {
    /// The background of the [`Expander`](crate::native::expander::Expander).
    pub background: Background,

    /// The border radius of the [`Expander`](crate::native::expander::Expander).
    pub border_radius: f32,

    /// The border width of the [`Expander`](crate::native::expander::Expander).
    pub border_width: f32,

    /// The border color of the [`Expander`](crate::native::expander::Expander).
    pub border_color: Color,

    /// The background of the head of the [`Expander`](crate::native::expander::Expander).
    pub head_background: Background,

    /// The text color of the head of the [`Expander`](crate::native::expander::Expander).
    pub head_text_color: Color,

    /// The background of the body of the [`Expander`](crate::native::expander::Expander).
    pub body_background: Background,

    /// The text color of the body of the [`Expander`](crate::native::expander::Expander).
    pub body_text_color: Color,

    /// The color of the close icon of the [`Expander`](crate::native::expander::Expander).
    pub toggle_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            background: Color::WHITE.into(),
            border_radius: 10.0, //32.0,
            border_width: 1.0,
            border_color: [0.87, 0.87, 0.87].into(), //Color::BLACK.into(),
            head_background: Background::Color([0.87, 0.87, 0.87].into()),
            head_text_color: Color::BLACK,
            body_background: Color::TRANSPARENT.into(),
            body_text_color: Color::BLACK,
            toggle_color: Color::BLACK,
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a container.
pub trait StyleSheet {
    type Style: Default + Copy;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: Self::Style) -> Appearance;
}
