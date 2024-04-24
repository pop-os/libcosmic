// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::segmented_button`].

use crate::widget::segmented_button::{Appearance, ItemAppearance, StyleSheet};
use crate::{theme::Theme, widget::segmented_button::ItemStatusAppearance};
use cosmic_theme::{Component, Container};
use iced_core::{border::Radius, Background};

#[derive(Default)]
pub enum SegmentedButton {
    /// A tabbed widget for switching between views in an interface.
    #[default]
    TabBar,
    /// A widget for multiple choice selection.
    Control,
    /// Or implement any custom theme of your liking.
    Custom(Box<dyn Fn(&Theme) -> Appearance>),
}

impl StyleSheet for Theme {
    type Style = SegmentedButton;

    #[allow(clippy::too_many_lines)]
    fn horizontal(&self, style: &Self::Style) -> Appearance {
        let container = &self.current_container();

        match style {
            SegmentedButton::TabBar => {
                let cosmic = self.cosmic();
                let active = horizontal::tab_bar_active(cosmic);
                Appearance {
                    border_radius: cosmic.corner_radii.radius_0.into(),
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border_radius: cosmic.corner_radii.radius_0.into(),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: cosmic.corner_radii.radius_0.into(),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: cosmic.corner_radii.radius_0.into(),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        text_color: container.component.on.into(),
                    },
                    hover: hover(cosmic, &container.component, &active),
                    focus: focus(cosmic, container, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Control => {
                let cosmic = self.cosmic();
                let active = horizontal::selection_active(cosmic, &container.component);
                let rad_m = cosmic.corner_radii.radius_m;
                let rad_0 = cosmic.corner_radii.radius_0;
                Appearance {
                    background: Some(Background::Color(container.small_widget.into())),
                    border_radius: rad_m.into(),
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border_radius: Radius::from([rad_m[0], rad_0[1], rad_0[2], rad_m[3]]),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: cosmic.corner_radii.radius_0.into(),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: Radius::from([rad_0[0], rad_m[1], rad_m[2], rad_0[3]]),
                            ..Default::default()
                        },
                        text_color: container.component.on.into(),
                    },
                    hover: hover(cosmic, &container.component, &active),
                    focus: focus(cosmic, container, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Custom(func) => func(self),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn vertical(&self, style: &Self::Style) -> Appearance {
        let cosmic = self.cosmic();
        let rad_m = cosmic.corner_radii.radius_m;
        let rad_0 = cosmic.corner_radii.radius_0;
        match style {
            SegmentedButton::TabBar => {
                let container = &self.cosmic().primary;
                let active = vertical::tab_bar_active(cosmic);
                Appearance {
                    border_radius: cosmic.corner_radii.radius_0.into(),
                    inactive: ItemStatusAppearance {
                        background: None,
                        text_color: container.component.on.into(),
                        ..active
                    },
                    hover: hover(cosmic, &container.component, &active),
                    focus: focus(cosmic, container, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Control => {
                let container = self.current_container();
                let active = vertical::selection_active(cosmic, &container.component);
                Appearance {
                    background: Some(Background::Color(container.small_widget.into())),
                    border_radius: rad_m.into(),
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border_radius: Radius::from([rad_m[0], rad_m[1], rad_0[0], rad_0[0]]),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: cosmic.corner_radii.radius_0.into(),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: Radius::from([rad_0[0], rad_0[1], rad_m[2], rad_m[3]]),
                            ..Default::default()
                        },
                        text_color: container.component.on.into(),
                    },
                    hover: hover(cosmic, &container.component, &active),
                    focus: focus(cosmic, container, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Custom(func) => func(self),
        }
    }
}

mod horizontal {
    use crate::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};
    use cosmic_theme::Component;
    use iced_core::{border::Radius, Background};

    pub fn selection_active(
        cosmic: &cosmic_theme::Theme,
        component: &Component,
    ) -> ItemStatusAppearance {
        let mut color = cosmic.palette.neutral_5;
        color.alpha = 0.2;

        let rad_m = cosmic.corner_radii.radius_m;
        let rad_0 = cosmic.corner_radii.radius_0;

        ItemStatusAppearance {
            background: Some(Background::Color(color.into())),
            first: ItemAppearance {
                border_radius: Radius::from([rad_m[0], rad_0[1], rad_0[2], rad_m[3]]),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: cosmic.corner_radii.radius_0.into(),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: Radius::from([rad_0[0], rad_m[1], rad_m[2], rad_0[3]]),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }

    pub fn tab_bar_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        let rad_s = cosmic.corner_radii.radius_s;
        let rad_0 = cosmic.corner_radii.radius_0;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }
}

pub fn focus(
    cosmic: &cosmic_theme::Theme,
    container: &Container,
    default: &ItemStatusAppearance,
) -> ItemStatusAppearance {
    let color = container.small_widget;
    ItemStatusAppearance {
        background: Some(Background::Color(color.into())),
        text_color: cosmic.accent.base.into(),
        ..*default
    }
}

pub fn hover(
    cosmic: &cosmic_theme::Theme,
    component: &Component,
    default: &ItemStatusAppearance,
) -> ItemStatusAppearance {
    let mut color = cosmic.palette.neutral_8;
    color.alpha = 0.2;
    ItemStatusAppearance {
        background: Some(Background::Color(color.into())),
        text_color: cosmic.accent.base.into(),
        ..*default
    }
}

mod vertical {
    use crate::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};
    use cosmic_theme::Component;
    use iced_core::{border::Radius, Background};

    pub fn selection_active(
        cosmic: &cosmic_theme::Theme,
        component: &Component,
    ) -> ItemStatusAppearance {
        let mut color = component.selected_state_color();
        color.alpha = 0.3;

        let rad_0 = cosmic.corner_radii.radius_0;
        let rad_m = cosmic.corner_radii.radius_m;

        ItemStatusAppearance {
            background: Some(Background::Color(color.into())),
            first: ItemAppearance {
                border_radius: Radius::from([rad_m[0], rad_m[1], rad_0[2], rad_0[3]]),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: cosmic.corner_radii.radius_0.into(),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: Radius::from([rad_0[0], rad_0[1], rad_m[2], rad_m[3]]),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }

    pub fn tab_bar_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }
}
