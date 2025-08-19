// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::segmented_button`].

use crate::widget::segmented_button::{Appearance, ItemAppearance, StyleSheet};
use crate::{theme::Theme, widget::segmented_button::ItemStatusAppearance};
use iced::Border;
use iced_core::{Background, border::Radius};
use palette::WithAlpha;

#[derive(Default)]
pub enum SegmentedButton {
    /// A tabbed widget for switching between views in an interface.
    #[default]
    TabBar,
    /// A widget for multiple choice selection.
    Control,
    /// Navigation bar style
    NavBar,
    /// File browser
    FileNav,
    /// Or implement any custom theme of your liking.
    Custom(Box<dyn Fn(&Theme) -> Appearance>),
}

impl StyleSheet for Theme {
    type Style = SegmentedButton;

    #[allow(clippy::too_many_lines)]
    fn horizontal(&self, style: &Self::Style) -> Appearance {
        let cosmic = self.cosmic();
        let container = self.current_container();
        match style {
            SegmentedButton::Control => {
                let rad_xl = cosmic.corner_radii.radius_xl;
                let rad_0 = cosmic.corner_radii.radius_0;
                let active = horizontal::selection_active(cosmic, &container.component);
                Appearance {
                    background: Some(Background::Color(container.component.base.into())),
                    border: Border {
                        radius: rad_xl.into(),
                        ..Default::default()
                    },
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border: Border {
                                radius: Radius::from([rad_xl[0], rad_0[1], rad_0[2], rad_xl[3]]),
                                ..Default::default()
                            },
                        },
                        middle: ItemAppearance {
                            border: Border {
                                radius: cosmic.corner_radii.radius_0.into(),
                                ..Default::default()
                            },
                        },
                        last: ItemAppearance {
                            border: Border {
                                radius: Radius::from([rad_0[0], rad_xl[1], rad_xl[2], rad_0[3]]),
                                ..Default::default()
                            },
                        },
                        text_color: container.component.on.into(),
                    },
                    hover: hover(cosmic, &active, 0.2),
                    pressed: hover(cosmic, &active, 0.15),
                    active,
                    ..Default::default()
                }
            }

            SegmentedButton::NavBar | SegmentedButton::FileNav => Appearance {
                active_width: 0.0,
                ..horizontal::tab_bar(cosmic, container)
            },

            SegmentedButton::TabBar => horizontal::tab_bar(cosmic, container),

            SegmentedButton::Custom(func) => func(self),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn vertical(&self, style: &Self::Style) -> Appearance {
        let cosmic = self.cosmic();
        let container = self.current_container();
        match style {
            SegmentedButton::Control => {
                let rad_xl = cosmic.corner_radii.radius_xl;
                let rad_0 = cosmic.corner_radii.radius_0;
                let active = vertical::selection_active(cosmic, &container.component);
                Appearance {
                    background: Some(Background::Color(container.component.base.into())),
                    border: Border {
                        radius: rad_xl.into(),
                        ..Default::default()
                    },
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border: Border {
                                radius: Radius::from([rad_xl[0], rad_xl[1], rad_0[0], rad_0[0]]),
                                ..Default::default()
                            },
                        },
                        middle: ItemAppearance {
                            border: Border {
                                radius: cosmic.corner_radii.radius_0.into(),
                                ..Default::default()
                            },
                        },
                        last: ItemAppearance {
                            border: Border {
                                radius: Radius::from([rad_0[0], rad_0[1], rad_xl[2], rad_xl[3]]),
                                ..Default::default()
                            },
                        },
                        text_color: container.component.on.into(),
                    },
                    hover: hover(cosmic, &active, 0.2),
                    pressed: hover(cosmic, &active, 0.15),
                    active,
                    ..Default::default()
                }
            }

            SegmentedButton::NavBar | SegmentedButton::FileNav => Appearance {
                active_width: 0.0,
                ..vertical::tab_bar(cosmic, container)
            },

            SegmentedButton::TabBar => vertical::tab_bar(cosmic, container),

            SegmentedButton::Custom(func) => func(self),
        }
    }
}

mod horizontal {
    use super::Appearance;
    use crate::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};
    use cosmic_theme::{Component, Container};
    use iced::Border;
    use iced_core::{Background, border::Radius};
    use palette::WithAlpha;

    pub fn tab_bar(cosmic: &cosmic_theme::Theme, container: &Container) -> Appearance {
        let active = tab_bar_active(cosmic);
        let hc = cosmic.is_high_contrast;
        let border = if hc {
            Border {
                color: container.component.border.into(),
                radius: cosmic.corner_radii.radius_0.into(),
                width: 1.0,
            }
        } else {
            Border::default()
        };

        Appearance {
            active_width: 4.0,
            border: Border {
                radius: cosmic.corner_radii.radius_0.into(),
                ..Default::default()
            },
            inactive: ItemStatusAppearance {
                background: None,
                first: ItemAppearance { border },
                middle: ItemAppearance { border },
                last: ItemAppearance { border },
                text_color: container.component.on.into(),
            },
            hover: super::hover(cosmic, &active, 0.3),
            pressed: super::hover(cosmic, &active, 0.25),
            active,
            ..Default::default()
        }
    }

    pub fn selection_active(
        cosmic: &cosmic_theme::Theme,
        component: &Component,
    ) -> ItemStatusAppearance {
        let rad_xl = cosmic.corner_radii.radius_xl;
        let rad_0 = cosmic.corner_radii.radius_0;

        ItemStatusAppearance {
            background: Some(Background::Color(
                cosmic.palette.neutral_5.with_alpha(0.1).into(),
            )),
            first: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_xl[0], rad_0[1], rad_0[2], rad_xl[3]]),
                    ..Default::default()
                },
            },
            middle: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_0.into(),
                    ..Default::default()
                },
            },
            last: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_0[0], rad_xl[1], rad_xl[2], rad_0[3]]),
                    ..Default::default()
                },
            },
            text_color: cosmic.accent_text_color().into(),
        }
    }

    pub fn tab_bar_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        let rad_s = cosmic.corner_radii.radius_s;
        let rad_0 = cosmic.corner_radii.radius_0;
        ItemStatusAppearance {
            background: Some(Background::Color(
                cosmic.palette.neutral_5.with_alpha(0.2).into(),
            )),
            first: ItemAppearance {
                border: Border {
                    color: cosmic.accent.base.into(),
                    radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                    width: 0.0,
                },
            },
            middle: ItemAppearance {
                border: Border {
                    color: cosmic.accent.base.into(),
                    radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                    width: 0.0,
                },
            },
            last: ItemAppearance {
                border: Border {
                    color: cosmic.accent.base.into(),
                    radius: Radius::from([rad_s[0], rad_s[1], rad_0[2], rad_0[3]]),
                    width: 0.0,
                },
            },
            text_color: cosmic.accent_text_color().into(),
        }
    }
}

mod vertical {
    use super::Appearance;
    use crate::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};
    use cosmic_theme::{Component, Container};
    use iced::Border;
    use iced_core::{Background, border::Radius};
    use palette::WithAlpha;

    pub fn tab_bar(cosmic: &cosmic_theme::Theme, container: &Container) -> Appearance {
        let active = tab_bar_active(cosmic);
        Appearance {
            active_width: 4.0,
            border: Border {
                radius: cosmic.corner_radii.radius_0.into(),
                ..Default::default()
            },
            inactive: ItemStatusAppearance {
                background: None,
                text_color: container.component.on.into(),
                ..active
            },
            hover: super::hover(cosmic, &active, 0.3),
            pressed: super::hover(cosmic, &active, 0.25),
            active,
            ..Default::default()
        }
    }

    pub fn selection_active(
        cosmic: &cosmic_theme::Theme,
        component: &Component,
    ) -> ItemStatusAppearance {
        let rad_0 = cosmic.corner_radii.radius_0;
        let rad_xl = cosmic.corner_radii.radius_xl;

        ItemStatusAppearance {
            background: Some(Background::Color(
                cosmic.palette.neutral_5.with_alpha(0.1).into(),
            )),
            first: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_xl[0], rad_xl[1], rad_0[2], rad_0[3]]),
                    ..Default::default()
                },
            },
            middle: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_0.into(),
                    ..Default::default()
                },
            },
            last: ItemAppearance {
                border: Border {
                    radius: Radius::from([rad_0[0], rad_0[1], rad_xl[2], rad_xl[3]]),
                    ..Default::default()
                },
            },
            text_color: cosmic.accent_text_color().into(),
        }
    }

    pub fn tab_bar_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        ItemStatusAppearance {
            background: Some(Background::Color(
                cosmic.palette.neutral_5.with_alpha(0.2).into(),
            )),
            first: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 0.0,
                    ..Default::default()
                },
            },
            middle: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 0.0,
                    ..Default::default()
                },
            },
            last: ItemAppearance {
                border: Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 0.0,
                    ..Default::default()
                },
            },
            text_color: cosmic.accent_text_color().into(),
        }
    }
}

pub fn hover(
    cosmic: &cosmic_theme::Theme,
    default: &ItemStatusAppearance,
    alpha: f32,
) -> ItemStatusAppearance {
    ItemStatusAppearance {
        background: Some(Background::Color(
            cosmic.palette.neutral_5.with_alpha(alpha).into(),
        )),
        text_color: cosmic.accent_text_color().into(),
        ..*default
    }
}
