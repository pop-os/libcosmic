// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic_theme::Component;
use iced_core::{Background, Color};
use palette::{rgb::Rgb, Alpha};

use crate::{
    app,
    widget::button::{Appearance, StyleSheet},
};

#[derive(Copy, Clone, Debug, Default)]
pub enum Button {
    Destructive,
    Link,
    Icon,
    #[default]
    Standard,
    Suggested,
    Text,
}

pub fn appearance(
    theme: &crate::Theme,
    focused: bool,
    style: &Button,
    color: fn(&Component<Alpha<Rgb, f32>>) -> Color,
) -> Appearance {
    let cosmic = theme.cosmic();
    let mut corner_radii = &cosmic.corner_radii.radius_xl;
    let mut appearance = Appearance::new();

    match style {
        Button::Standard => {
            let component = &theme.current_container().component;
            appearance.background = Some(Background::Color(color(component)));
            appearance.text_color = component.on.into();
        }

        Button::Icon | Button::Text => {
            let component = &cosmic.text_button;
            appearance.background = None;
            appearance.text_color = component.on.into();
        }

        Button::Suggested => {
            let component = &cosmic.accent_button;
            appearance.background = Some(Background::Color(color(component)));
            appearance.icon_color = Some(component.on.into());
            appearance.text_color = component.on.into();
        }

        Button::Destructive => {
            let component = &cosmic.destructive_button;
            appearance.background = Some(Background::Color(color(component)));
            appearance.icon_color = Some(component.on.into());
            appearance.text_color = component.on.into();
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent.base.into());
            appearance.text_color = cosmic.accent.base.into();
            corner_radii = &cosmic.corner_radii.radius_0;
        }
    }

    appearance.border_radius = (*corner_radii).into();

    if focused {
        appearance.outline_width = 1.0;
        appearance.outline_color = cosmic.accent.base.into();
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }

    appearance
}

impl StyleSheet for crate::Theme {
    type Style = Button;

    fn active(&self, focused: bool, style: &Self::Style) -> Appearance {
        appearance(self, focused, style, |component| component.base.into())
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        appearance(self, false, style, |component| {
            let mut color = Color::from(component.base);
            color.a *= 0.5;
            color
        })
    }

    fn drop_target(&self, style: &Self::Style) -> Appearance {
        let mut appearance = self.active(false, style);

        appearance
    }

    fn hovered(&self, focused: bool, style: &Self::Style) -> Appearance {
        appearance(self, focused, style, |component| component.hover.into())
    }

    fn pressed(&self, focused: bool, style: &Self::Style) -> Appearance {
        appearance(self, focused, style, |component| component.pressed.into())
    }

    fn selected(&self, focused: bool, style: &Self::Style) -> Appearance {
        appearance(self, focused, style, |component| component.selected.into())
    }
}
