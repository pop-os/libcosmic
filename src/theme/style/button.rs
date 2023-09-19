// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::button`].

use cosmic_theme::Component;
use iced_core::{Background, Color};
use palette::{rgb::Rgb, Alpha};

use crate::{
    theme::TRANSPARENT_COMPONENT,
    widget::button::{Appearance, StyleSheet},
};

#[derive(Default)]
pub enum Button {
    Custom {
        active: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
        disabled: Box<dyn Fn(&crate::Theme) -> Appearance>,
        hovered: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
        pressed: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
    },
    Destructive,
    Link,
    Icon,
    IconVertical,
    #[default]
    Standard,
    Suggested,
    Text,
    Transparent,
}

pub fn appearance(
    theme: &crate::Theme,
    focused: bool,
    style: &Button,
    color: impl Fn(&Component<Alpha<Rgb, f32>>) -> (Color, Option<Color>, Option<Color>),
) -> Appearance {
    let cosmic = theme.cosmic();
    let mut corner_radii = &cosmic.corner_radii.radius_xl;
    let mut appearance = Appearance::new();

    match style {
        Button::Standard
        | Button::Text
        | Button::Suggested
        | Button::Destructive
        | Button::Transparent => {
            let style_component = match style {
                Button::Standard => &cosmic.button,
                Button::Text => &cosmic.text_button,
                Button::Suggested => &cosmic.accent_button,
                Button::Destructive => &cosmic.destructive_button,
                Button::Transparent => &TRANSPARENT_COMPONENT,
                _ => return appearance,
            };

            let (background, text, icon) = color(style_component);
            appearance.background = Some(Background::Color(background));
            appearance.text_color = text;
            appearance.icon_color = icon;
        }

        Button::Icon | Button::IconVertical => {
            if let Button::IconVertical = style {
                corner_radii = &cosmic.corner_radii.radius_m;
            }

            let (background, _text, icon) = color(&cosmic.icon_button);
            appearance.background = Some(Background::Color(background));
            // appearance.text_color = text;
            // appearance.icon_color = icon;

            if focused {
                appearance.text_color = Some(cosmic.accent.on.into());
                appearance.icon_color = Some(cosmic.accent.on.into());
            }
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent.base.into());
            appearance.text_color = Some(cosmic.accent.base.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::Custom { .. } => (),
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
        if let Button::Custom { active, .. } = style {
            return active(focused, self);
        }

        appearance(self, focused, style, |component| {
            (
                component.base.into(),
                Some(component.on.into()),
                Some(component.on.into()),
            )
        })
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        if let Button::Custom { disabled, .. } = style {
            return disabled(self);
        }

        appearance(self, false, style, |component| {
            let mut background = Color::from(component.base);
            background.a *= 0.5;
            (
                background,
                Some(component.on_disabled.into()),
                Some(component.on_disabled.into()),
            )
        })
    }

    fn drop_target(&self, style: &Self::Style) -> Appearance {
        self.active(false, style)
    }

    fn hovered(&self, focused: bool, style: &Self::Style) -> Appearance {
        if let Button::Custom { hovered, .. } = style {
            return hovered(focused, self);
        }

        appearance(self, focused, style, |component| {
            (
                component.hover.into(),
                Some(component.on.into()),
                Some(component.on.into()),
            )
        })
    }

    fn pressed(&self, focused: bool, style: &Self::Style) -> Appearance {
        if let Button::Custom { pressed, .. } = style {
            return pressed(focused, self);
        }

        appearance(self, focused, style, |component| {
            (
                component.pressed.into(),
                Some(component.on.into()),
                Some(component.on.into()),
            )
        })
    }
}
