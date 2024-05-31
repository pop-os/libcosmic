// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::button`].

use cosmic_theme::Component;
use iced_core::{Background, Color};

use crate::{
    theme::TRANSPARENT_COMPONENT,
    widget::button::{Appearance, StyleSheet},
};

#[derive(Default)]
pub enum Button {
    AppletIcon,
    Custom {
        active: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
        disabled: Box<dyn Fn(&crate::Theme) -> Appearance>,
        hovered: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
        pressed: Box<dyn Fn(bool, &crate::Theme) -> Appearance>,
    },
    AppletMenu,
    Destructive,
    HeaderBar,
    Icon,
    IconVertical,
    Image,
    Link,
    MenuItem,
    MenuRoot,
    #[default]
    Standard,
    Suggested,
    Text,
    Transparent,
}

pub fn appearance(
    theme: &crate::Theme,
    focused: bool,
    selected: bool,
    style: &Button,
    color: impl Fn(&Component) -> (Color, Option<Color>, Option<Color>),
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
            if !matches!(style, Button::Standard | Button::Text) {
                appearance.text_color = text;
                appearance.icon_color = icon;
            }
        }

        Button::Icon | Button::IconVertical | Button::HeaderBar => {
            if matches!(style, Button::IconVertical) {
                corner_radii = &cosmic.corner_radii.radius_m;
            }

            let (background, text, icon) = color(&cosmic.icon_button);
            appearance.background = Some(Background::Color(background));
        }

        Button::Image => {
            appearance.background = None;
            appearance.text_color = Some(cosmic.accent.base.into());
            appearance.icon_color = Some(cosmic.accent.base.into());

            corner_radii = &cosmic.corner_radii.radius_s;
            appearance.border_radius = (*corner_radii).into();

            if focused || selected {
                appearance.border_width = 2.0;
                appearance.border_color = cosmic.accent.base.into();
            }

            return appearance;
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent.base.into());
            appearance.text_color = Some(cosmic.accent.base.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::Custom { .. } => (),
        Button::AppletMenu => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }
        Button::AppletIcon => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
        }
        Button::MenuRoot => {
            appearance.background = None;
            appearance.icon_color = None;
            appearance.text_color = None;
        }
        Button::MenuItem => {
            let (background, _, _) = color(&cosmic.background.component);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
            corner_radii = &cosmic.corner_radii.radius_s;
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

    fn active(&self, focused: bool, selected: bool, style: &Self::Style) -> Appearance {
        if let Button::Custom { active, .. } = style {
            return active(focused, self);
        }

        appearance(self, focused, selected, style, move |component| {
            let text_color = if matches!(
                style,
                Button::Icon | Button::IconVertical | Button::HeaderBar
            ) && selected
            {
                Some(self.cosmic().accent_color().into())
            } else {
                Some(component.on.into())
            };

            (component.base.into(), text_color, text_color)
        })
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        if let Button::Custom { disabled, .. } = style {
            return disabled(self);
        }

        appearance(self, false, false, style, |component| {
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
        self.active(false, false, style)
    }

    fn hovered(&self, focused: bool, selected: bool, style: &Self::Style) -> Appearance {
        if let Button::Custom { hovered, .. } = style {
            return hovered(focused, self);
        }

        appearance(
            self,
            focused || matches!(style, Button::Image),
            selected,
            style,
            |component| {
                let text_color = if matches!(
                    style,
                    Button::Icon | Button::IconVertical | Button::HeaderBar
                ) && selected
                {
                    Some(self.cosmic().accent_color().into())
                } else {
                    Some(component.on.into())
                };

                (component.hover.into(), text_color, text_color)
            },
        )
    }

    fn pressed(&self, focused: bool, selected: bool, style: &Self::Style) -> Appearance {
        if let Button::Custom { pressed, .. } = style {
            return pressed(focused, self);
        }

        appearance(self, focused, selected, style, |component| {
            let text_color = if matches!(
                style,
                Button::Icon | Button::IconVertical | Button::HeaderBar
            ) && selected
            {
                Some(self.cosmic().accent_color().into())
            } else {
                Some(component.on.into())
            };

            (component.pressed.into(), text_color, text_color)
        })
    }

    fn selection_background(&self) -> Background {
        Background::Color(self.cosmic().primary.base.into())
    }
}
