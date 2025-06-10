// From iced_aw, license MIT

//! Change the appearance of menu bars and their menus.
use std::sync::Arc;

use crate::Theme;
use iced_widget::core::Color;

/// The appearance of a menu bar and its menus.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The background color of the menu bar and its menus.
    pub background: Color,
    /// The border width of the menu bar and its menus.
    pub border_width: f32,
    /// The border radius of the menu bar.
    pub bar_border_radius: [f32; 4],
    /// The border radius of the menus.
    pub menu_border_radius: [f32; 4],
    /// The border [`Color`] of the menu bar and its menus.
    pub border_color: Color,
    /// The expand value of the menus' background
    pub background_expand: [u16; 4],
    // /// The highlighted path [`Color`] of the the menu bar and its menus.
    pub path: Color,
}

/// The style sheet of a menu bar and its menus.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a menu bar and its menus.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

/// The style of a menu bar and its menus
#[derive(Default, Clone)]
#[allow(missing_debug_implementations)]
pub enum MenuBarStyle {
    /// The default style.
    #[default]
    Default,
    /// A [`Theme`] that uses a `Custom` palette.
    Custom(Arc<dyn StyleSheet<Style = Theme> + Send + Sync>),
}

impl From<fn(&Theme) -> Appearance> for MenuBarStyle {
    fn from(f: fn(&Theme) -> Appearance) -> Self {
        Self::Custom(Arc::new(f))
    }
}

impl StyleSheet for fn(&Theme) -> Appearance {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        (self)(style)
    }
}

impl StyleSheet for Theme {
    type Style = MenuBarStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let cosmic = self.cosmic();
        let component = &cosmic.background.component;

        match style {
            MenuBarStyle::Default => Appearance {
                background: component.base.into(),
                border_width: 1.0,
                bar_border_radius: cosmic.corner_radii.radius_xl,
                menu_border_radius: cosmic.corner_radii.radius_s.map(|x| x + 2.0),
                border_color: component.divider.into(),
                background_expand: [1; 4],
                path: component.hover.into(),
            },
            MenuBarStyle::Custom(c) => c.appearance(self),
        }
    }
}
