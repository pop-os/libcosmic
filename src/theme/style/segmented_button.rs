// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementation for [`crate::widget::segmented_button`].

use crate::widget::segmented_button::{Appearance, ItemAppearance, StyleSheet};
use crate::{theme::Theme, widget::segmented_button::ItemStatusAppearance};
use iced_core::{Background, BorderRadius};
use palette::{rgb::Rgb, Alpha};

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
                    border_radius: BorderRadius::from(0.0),
                    inactive: ItemStatusAppearance {
                        background: None,
                        first: ItemAppearance {
                            border_radius: BorderRadius::from(0.0),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: BorderRadius::from(0.0),
                            border_bottom: Some((1.0, cosmic.accent.base.into())),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: BorderRadius::from(0.0),
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
                Appearance {
                    border_radius: BorderRadius::from(0.0),
                    inactive: ItemStatusAppearance {
                        background: Some(Background::Color(neutral_5.into())),
                        first: ItemAppearance {
                            border_radius: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
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
        match style {
            SegmentedButton::ViewSwitcher => {
                let cosmic = self.cosmic();
                let active = vertical::view_switcher_active(cosmic);
                Appearance {
                    border_radius: BorderRadius::from(0.0),
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
                let cosmic = self.cosmic();
                let active = vertical::selection_active(cosmic);
                let mut neutral_5 = cosmic.palette.neutral_5;
                neutral_5.alpha = 0.2;
                Appearance {
                    border_radius: BorderRadius::from(0.0),
                    inactive: ItemStatusAppearance {
                        background: Some(Background::Color(neutral_5.into())),
                        first: ItemAppearance {
                            border_radius: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                            ..Default::default()
                        },
                        middle: ItemAppearance {
                            border_radius: BorderRadius::from(0.0),
                            ..Default::default()
                        },
                        last: ItemAppearance {
                            border_radius: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
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
    use iced_core::{Background, BorderRadius};
    use palette::{rgb::Rgb, Alpha};

    pub fn selection_active(cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: BorderRadius::from([24.0, 0.0, 0.0, 24.0]),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: BorderRadius::from(0.0),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: BorderRadius::from([0.0, 24.0, 24.0, 0.0]),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }

    pub fn view_switcher_active(
        cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>,
    ) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: BorderRadius::from([8.0, 8.0, 0.0, 0.0]),
                border_bottom: Some((4.0, cosmic.accent.base.into())),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }
}

pub fn focus(
    cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>,
    default: &ItemStatusAppearance,
) -> ItemStatusAppearance {
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

pub fn hover(
    cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>,
    default: &ItemStatusAppearance,
) -> ItemStatusAppearance {
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
    use iced_core::{Background, BorderRadius};
    use palette::{rgb::Rgb, Alpha};

    pub fn selection_active(cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: BorderRadius::from([24.0, 24.0, 0.0, 0.0]),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: BorderRadius::from(0.0),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: BorderRadius::from([0.0, 0.0, 24.0, 24.0]),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }

    pub fn view_switcher_active(
        cosmic: &cosmic_theme::Theme<Alpha<Rgb, f32>>,
    ) -> ItemStatusAppearance {
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.2;
        ItemStatusAppearance {
            background: Some(Background::Color(neutral_5.into())),
            first: ItemAppearance {
                border_radius: BorderRadius::from(24.0),
                ..Default::default()
            },
            middle: ItemAppearance {
                border_radius: BorderRadius::from(24.0),
                ..Default::default()
            },
            last: ItemAppearance {
                border_radius: BorderRadius::from(24.0),
                ..Default::default()
            },
            text_color: cosmic.accent.base.into(),
        }
    }
}
