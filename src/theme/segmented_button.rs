// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::theme::Theme;
use crate::widget::segmented_button;
use iced_core::{Background, BorderRadius};

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
                    border_radius: BorderRadius::from(0.0),
                    active: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.base.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((4.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((4.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((4.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    inactive: segmented_button::ButtonStatusAppearance {
                        background: None,
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        text_color: cosmic.primary.on.into(),
                    },
                    hover: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    ..Default::default()
                }
            }
            SegmentedButton::Selection => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    border_radius: BorderRadius::from(0.0),
                    active: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(
                            cosmic.secondary.component.divider.into(),
                        )),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    inactive: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.secondary.component.base.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.primary.on.into(),
                    },
                    hover: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    ..Default::default()
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
                    border_radius: BorderRadius::from(0.0),
                    active: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.base.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    inactive: segmented_button::ButtonStatusAppearance {
                        background: None,
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        text_color: cosmic.primary.on.into(),
                    },
                    hover: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(24.0),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    ..Default::default()
                }
            }
            SegmentedButton::Selection => {
                let cosmic = self.cosmic();
                segmented_button::Appearance {
                    border_radius: BorderRadius::from(0.0),
                    active: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(
                            cosmic.secondary.component.divider.into(),
                        )),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    inactive: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.secondary.component.base.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.primary.on.into(),
                    },
                    hover: segmented_button::ButtonStatusAppearance {
                        background: Some(Background::Color(cosmic.primary.component.hover.into())),
                        first: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                            ..Default::default()
                        },
                        middle: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: segmented_button::ButtonAppearance {
                            border_radius: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                            ..Default::default()
                        },
                        text_color: cosmic.accent.base.into(),
                    },
                    ..Default::default()
                }
            }
            SegmentedButton::Custom(func) => func(self),
        }
    }
}
