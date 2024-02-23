// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::segmented_button`].

use crate::widget::segmented_button::{Appearance, ItemAppearance, StyleSheet};
use crate::{theme::Theme, widget::segmented_button::ItemStatusAppearance};
use iced_core::{border::Radius, Background};

#[derive(Default)]
pub enum SegmentedButton {
    /// A tabbed widget for switching between views in an interface.
    #[default]
    ViewSwitcher,
    /// A widget for multiple choice selection.
    Selection,
    /// Or implement any custom theme of your liking.
    Custom(Box<dyn Fn(&Theme) -> Appearance>),
}

impl StyleSheet for Theme {
    type Style = SegmentedButton;

    #[allow(clippy::too_many_lines)]
    fn horizontal(&self, style: &Self::Style) -> Appearance {
        match style {
            SegmentedButton::ViewSwitcher => {
                let cosmic = self.cosmic();
                let active = horizontal::view_switcher_active(cosmic);
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
                        text_color: cosmic.on_bg_color().into(),
                    },
                    hover: hover(cosmic, &active),
                    focus: focus(cosmic, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Selection => {
                let cosmic = self.cosmic();
                let active = horizontal::selection_active(cosmic);
                let mut neutral_5 = cosmic.palette.neutral_5;
                neutral_5.alpha = 0.2;
                let rad_m = cosmic.corner_radii.radius_m;
                let rad_0 = cosmic.corner_radii.radius_0;
                Appearance {
                    border_radius: cosmic.corner_radii.radius_0.into(),
                    inactive: ItemStatusAppearance {
                        background: Some(Background::Color(cosmic.small_container_widget().into())),
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
                        text_color: cosmic.on_bg_color().into(),
                    },
                    hover: hover(cosmic, &active),
                    focus: focus(cosmic, &active),
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
            SegmentedButton::ViewSwitcher => {
                let active = vertical::view_switcher_active(cosmic);
                Appearance {
                    border_radius: cosmic.corner_radii.radius_0.into(),
                    inactive: ItemStatusAppearance {
                        background: None,
                        text_color: cosmic.on_bg_color().into(),
                        ..active
                    },
                    hover: hover(cosmic, &active),
                    focus: focus(cosmic, &active),
                    active,
                    ..Default::default()
                }
            }
            SegmentedButton::Selection => {
                let active = vertical::selection_active(cosmic);
                let mut neutral_5 = cosmic.palette.neutral_5;
                neutral_5.alpha = 0.2;
                Appearance {
                    border_radius: cosmic.corner_radii.radius_0.into(),
                    inactive: ItemStatusAppearance {
                        background: Some(Background::Color(neutral_5.into())),
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
                        text_color: cosmic.on_bg_color().into(),
                    },
                    hover: hover(cosmic, &active),
                    focus: focus(cosmic, &active),
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
    use iced_core::{border::Radius, Background};

    pub fn selection_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        let rad_m = cosmic.corner_radii.radius_m;
        let rad_0 = cosmic.corner_radii.radius_0;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
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

    pub fn view_switcher_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
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

pub fn focus(cosmic: &cosmic_theme::Theme, default: &ItemStatusAppearance) -> ItemStatusAppearance {
    // TODO: This is a hack to make the hover color lighter than the selected color
    // I'm not sure why the alpha is being applied differently here than in figma
    let mut neutral_5 = cosmic.palette.neutral_5;
    neutral_5.alpha = 0.2;
    ItemStatusAppearance {
        background: Some(Background::Color(neutral_5.into())),
        text_color: cosmic.accent.base.into(),
        ..*default
    }
}

pub fn hover(cosmic: &cosmic_theme::Theme, default: &ItemStatusAppearance) -> ItemStatusAppearance {
    let mut neutral_10 = cosmic.palette.neutral_10;
    neutral_10.alpha = 0.1;
    ItemStatusAppearance {
        background: Some(Background::Color(neutral_10.into())),
        text_color: cosmic.accent.base.into(),
        ..*default
    }
}

mod vertical {
    use crate::widget::segmented_button::{ItemAppearance, ItemStatusAppearance};
    use iced_core::{border::Radius, Background};

    pub fn selection_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        let rad_0 = cosmic.corner_radii.radius_0;
        let rad_m = cosmic.corner_radii.radius_m;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
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

    pub fn view_switcher_active(cosmic: &cosmic_theme::Theme) -> ItemStatusAppearance {
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
