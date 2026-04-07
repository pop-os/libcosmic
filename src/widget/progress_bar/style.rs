use iced::Color;

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
    /// The border [`Color`] of the progress indicator.
    pub border_color: Option<Color>,
    /// The border radius of the progress indicator.
    pub border_radius: f32,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
            border_color: None,
            border_radius: 0.0,
        }
    }
}

/// A set of rules that dictate the style of an indicator.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a indicator.
    fn appearance(
        &self,
        style: &Self::Style,
        is_determinate: bool,
        is_circular: bool,
    ) -> Appearance;
}

impl StyleSheet for iced::Theme {
    type Style = ();

    fn appearance(
        &self,
        _style: &Self::Style,
        _is_determinate: bool,
        _is_circular: bool,
    ) -> Appearance {
        let palette = self.extended_palette();

        Appearance {
            track_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
            border_color: None,
            border_radius: 0.0,
        }
    }
}

impl StyleSheet for crate::Theme {
    type Style = ();

    fn appearance(
        &self,
        _style: &Self::Style,
        is_determinate: bool,
        is_circular: bool,
    ) -> Appearance {
        let cur = self.current_container();
        let mut cur_divider = cur.divider;
        cur_divider.alpha = 0.5;
        let theme = self.cosmic();

        let (mut track_color, bar_color) = if theme.is_dark && theme.is_high_contrast {
            (
                theme.palette.neutral_6.into(),
                theme.accent_text_color().into(),
            )
        } else if theme.is_dark {
            (theme.palette.neutral_5.into(), theme.accent_color().into())
        } else if theme.is_high_contrast {
            (
                theme.palette.neutral_4.into(),
                theme.accent_text_color().into(),
            )
        } else {
            (theme.palette.neutral_3.into(), theme.accent_color().into())
        };

        if !is_determinate && is_circular {
            track_color = Color::TRANSPARENT;
        }

        Appearance {
            track_color,
            bar_color,
            border_color: if is_determinate && theme.is_high_contrast {
                Some(cur_divider.into())
            } else {
                None
            },
            border_radius: theme.corner_radii.radius_xl[0],
        }
    }
}
