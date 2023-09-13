// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`cosmic::widget::text_input`].

use crate::ext::ColorExt;
use crate::widget::text_input::{Appearance, StyleSheet};
use iced_core::Color;

#[derive(Default)]
pub enum TextInput {
    #[default]
    Default,
    ExpandableSearch,
    Search,
    Inline,
    Custom {
        active: Box<dyn Fn(&crate::Theme) -> Appearance>,
        error: Box<dyn Fn(&crate::Theme) -> Appearance>,
        hovered: Box<dyn Fn(&crate::Theme) -> Appearance>,
        focused: Box<dyn Fn(&crate::Theme) -> Appearance>,
        disabled: Box<dyn Fn(&crate::Theme) -> Appearance>,
    },
}

impl StyleSheet for crate::Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let container = self.current_container();

        let mut background: Color = container.component.base.into();
        background.a = 0.25;

        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;
        match style {
            TextInput::Default => Appearance {
                background: background.into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: container.component.divider.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::ExpandableSearch => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_xl.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search => Appearance {
                background: background.into(),
                border_radius: corner.radius_xl.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: container.component.divider.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { active, .. } => active(self),
        }
    }

    fn error(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let container = self.current_container();

        let mut background: Color = container.component.base.into();
        background.a = 0.25;

        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: background.into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: Color::from(palette.destructive_color()),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search | TextInput::ExpandableSearch => Appearance {
                background: background.into(),
                border_radius: corner.radius_xl.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { error, .. } => error(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let container = self.current_container();

        let mut background: Color = container.component.base.into();
        background.a = 0.25;

        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: background.into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: palette.accent.base.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search => Appearance {
                background: background.into(),
                border_radius: corner.radius_xl.into(),
                border_offset: None,
                border_width: 1.0,
                border_color: palette.accent.base.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::ExpandableSearch => Appearance {
                background: background.into(),
                border_radius: corner.radius_xl.into(),
                border_offset: None,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::from(container.component.hover).into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { hovered, .. } => hovered(self),
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let container = self.current_container();

        let mut background: Color = container.component.base.into();
        background.a = 0.25;

        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: background.into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: palette.accent.base.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search | TextInput::ExpandableSearch => Appearance {
                background: background.into(),
                border_radius: corner.radius_xl.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: palette.accent.base.into(),
                icon_color: container.on.into(),
                text_color: container.on.into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::from(palette.accent.base).into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                icon_color: container.on.into(),
                // TODO use regular text color here after text rendering handles multiple colors
                // in this case, for selected and unselected text
                text_color: palette.on_accent_color().into(),
                placeholder_color: {
                    let color: Color = container.on.into();
                    color.blend_alpha(background, 0.7)
                },
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { focused, .. } => focused(self),
        }
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        if let TextInput::Custom { disabled, .. } = style {
            return disabled(self);
        }

        let mut appearance = self.active(style);

        // TODO: iced will not render alpha itself on text or icon colors.
        let background: Color = self.current_container().component.base.into();
        appearance.text_color = appearance.text_color.blend_alpha(background, 0.5);
        appearance.icon_color = appearance.icon_color.blend_alpha(background, 0.5);

        appearance
    }
}
