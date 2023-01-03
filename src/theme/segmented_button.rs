// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::theme::Theme;
use crate::widget::segmented_button;
use iced_core::{Background, BorderRadius, Color};

#[derive(Clone, Copy, Default)]
pub enum SegmentedButton {
    /// A tabbed widget for switching between views in an interface.
    #[default]
    ViewSwitcher,
    /// A widget for multiple choice selection.
    Selection,
    /// Or implement any custom theme of your liking.
    Custom(fn(&Theme) -> segmented_button::Appearance),
}

impl segmented_button::StyleSheet for Theme {
    type Style = SegmentedButton;

    fn horizontal(&self, style: &Self::Style) -> segmented_button::Appearance {
        match style {
            SegmentedButton::ViewSwitcher => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    background: None,
                    border_color: Color::TRANSPARENT,
                    border_radius: BorderRadius::from(0.0),
                    border_width: 0.0,
                    button_active: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.base.into())),
                        border_bottom: Some((4.0, cosmic.accent.base.into())),
                        border_radius_first: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        border_radius_last: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        border_radius_middle: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        text_color: cosmic.accent.base.into(),
                    },
                    button_inactive: segmented_button::ButtonAppearance {
                        background: None,
                        border_bottom: Some((1.0, cosmic.accent.base.into())),
                        border_radius_first: BorderRadius::from(0.0),
                        border_radius_last: BorderRadius::from(0.0),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.primary.on.into(),
                    },
                    button_hover: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        border_bottom: Some((1.0, cosmic.accent.base.into())),
                        border_radius_first: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        border_radius_last: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        border_radius_middle: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                        text_color: cosmic.accent.base.into(),
                    },
                }
            }
            SegmentedButton::Selection => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    background: None,
                    border_color: Color::TRANSPARENT,
                    border_radius: BorderRadius::from(0.0),
                    border_width: 0.0,
                    button_active: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(
                            cosmic.secondary.component.divider.into(),
                        )),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                        border_radius_last: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.accent.base.into(),
                    },
                    button_inactive: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.secondary.component.base.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                        border_radius_last: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.primary.on.into(),
                    },
                    button_hover: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                        border_radius_last: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.accent.base.into(),
                    },
                }
            }
            SegmentedButton::Custom(func) => func(self),
        }
    }

    fn vertical(&self, style: &Self::Style) -> segmented_button::Appearance {
        match style {
            SegmentedButton::ViewSwitcher => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    background: None,
                    border_color: Color::TRANSPARENT,
                    border_radius: BorderRadius::from(0.0),
                    border_width: 0.0,
                    button_active: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.base.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from(24.0),
                        border_radius_last: BorderRadius::from(24.0),
                        border_radius_middle: BorderRadius::from(24.0),
                        text_color: cosmic.accent.base.into(),
                    },
                    button_inactive: segmented_button::ButtonAppearance {
                        background: None,
                        border_bottom: None,
                        border_radius_first: BorderRadius::from(24.0),
                        border_radius_last: BorderRadius::from(24.0),
                        border_radius_middle: BorderRadius::from(24.0),
                        text_color: cosmic.primary.on.into(),
                    },
                    button_hover: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from(24.0),
                        border_radius_last: BorderRadius::from(24.0),
                        border_radius_middle: BorderRadius::from(24.0),
                        text_color: cosmic.accent.base.into(),
                    },
                }
            }
            SegmentedButton::Selection => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    background: None,
                    border_color: Color::TRANSPARENT,
                    border_radius: BorderRadius::from(0.0),
                    border_width: 0.0,
                    button_active: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(
                            cosmic.secondary.component.divider.into(),
                        )),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                        border_radius_last: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.accent.base.into(),
                    },
                    button_inactive: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.secondary.component.base.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                        border_radius_last: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.primary.on.into(),
                    },
                    button_hover: segmented_button::ButtonAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        border_bottom: None,
                        border_radius_first: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                        border_radius_last: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                        border_radius_middle: BorderRadius::from(0.0),
                        text_color: cosmic.accent.base.into(),
                    },
                }
            }
            SegmentedButton::Custom(func) => func(self),
        }
    }
}
