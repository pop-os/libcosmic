use iced::Color;
use palette::WithAlpha;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
    /// The border [`Color`] of the progress indicator.
    pub border_color: Option<Color>,
    /// The border radius of the progress indicator.
    pub border_radius: f32,
}

/// [`Style`] field overrides
#[derive(Clone, Copy, Debug, Default)]
pub struct Class {
    pub track_color: Option<Color>,
    pub bar_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_radius: Option<f32>,
}

impl Class {
    pub fn track_color(mut self, color: impl Into<Color>) -> Self {
        self.track_color = Some(color.into());
        self
    }

    pub fn bar_color(mut self, color: impl Into<Color>) -> Self {
        self.bar_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    fn resolve(&self, base: Style) -> Style {
        Style {
            track_color: self.track_color.unwrap_or(base.track_color),
            bar_color: self.bar_color.unwrap_or(base.bar_color),
            border_color: self.border_color.or(base.border_color),
            border_radius: self.border_radius.unwrap_or(base.border_radius),
        }
    }
}

/// A set of rules that dictate the style of an indicator.
pub trait Catalog: Sized {
    /// The supported class of the [`Catalog`].
    type Class: Default;

    /// Produces the active [`Style`] of an indicator.
    fn style(&self, class: &Self::Class, is_determinate: bool, is_circular: bool) -> Style;
}

impl Catalog for iced::Theme {
    type Class = Class;

    fn style(&self, class: &Self::Class, _is_determinate: bool, _is_circular: bool) -> Style {
        let palette = self.extended_palette();
        class.resolve(Style {
            track_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
            border_color: None,
            border_radius: 0.0,
        })
    }
}

impl Catalog for crate::Theme {
    type Class = Class;

    fn style(&self, class: &Self::Class, is_determinate: bool, is_circular: bool) -> Style {
        let theme = self.cosmic();

        let (mut track_color, bar_color) = match (theme.is_dark, theme.is_high_contrast) {
            (true, true) => (
                theme.palette.neutral_6.into(),
                theme.accent_text_color().into(),
            ),
            (true, false) => (theme.palette.neutral_5.into(), theme.accent_color().into()),
            (false, true) => (
                theme.palette.neutral_4.into(),
                theme.accent_text_color().into(),
            ),
            (false, false) => (theme.palette.neutral_3.into(), theme.accent_color().into()),
        };

        if !is_determinate && is_circular {
            track_color = Color::TRANSPARENT;
        }

        class.resolve(Style {
            track_color,
            bar_color,
            border_color: if is_determinate && theme.is_high_contrast {
                Some(self.current_container().divider.with_alpha(0.5).into())
            } else {
                None
            },
            border_radius: theme.corner_radii.radius_xl[0],
        })
    }
}
