// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementations for widgets native to iced.

use crate::theme::{CosmicComponent, Theme, TRANSPARENT_COMPONENT};
use cosmic_theme::composite::over;
use iced_core::{Background, Border, Color, Shadow, Vector};
use iced_style::application;
use iced_style::button as iced_button;
use iced_style::checkbox;
use iced_style::container;
use iced_style::menu;
use iced_style::pane_grid;
use iced_style::pick_list;
use iced_style::progress_bar;
use iced_style::radio;
use iced_style::rule;
use iced_style::scrollable;
use iced_style::slider;
use iced_style::slider::Rail;
use iced_style::svg;
use iced_style::text_input;
use iced_style::toggler;

use std::rc::Rc;

#[derive(Default)]
pub enum Application {
    #[default]
    Default,
    Custom(Box<dyn Fn(&Theme) -> application::Appearance>),
}

impl Application {
    pub fn custom<F: Fn(&Theme) -> application::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl application::StyleSheet for Theme {
    type Style = Application;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        let cosmic = self.cosmic();

        match style {
            Application::Default => application::Appearance {
                icon_color: cosmic.bg_color().into(),
                background_color: cosmic.bg_color().into(),
                text_color: cosmic.on_bg_color().into(),
            },
            Application::Custom(f) => f(self),
        }
    }
}

/// Styles for the button widget from iced-rs.
#[derive(Default)]
pub enum Button {
    Deactivated,
    Destructive,
    Positive,
    #[default]
    Primary,
    Secondary,
    Text,
    Link,
    LinkActive,
    Transparent,
    Card,
    Custom {
        active: Box<dyn Fn(&Theme) -> iced_button::Appearance>,
        hover: Box<dyn Fn(&Theme) -> iced_button::Appearance>,
    },
}

impl Button {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::match_same_arms)]
    fn cosmic<'a>(&'a self, theme: &'a Theme) -> &CosmicComponent {
        let cosmic = theme.cosmic();
        match self {
            Self::Primary => &cosmic.accent_button,
            Self::Secondary => &theme.current_container().component,
            Self::Positive => &cosmic.success_button,
            Self::Destructive => &cosmic.destructive_button,
            Self::Text => &cosmic.text_button,
            Self::Link => &cosmic.accent_button,
            Self::LinkActive => &cosmic.accent_button,
            Self::Transparent => &TRANSPARENT_COMPONENT,
            Self::Deactivated => &theme.current_container().component,
            Self::Card => &theme.current_container().component,
            Self::Custom { .. } => &TRANSPARENT_COMPONENT,
        }
    }
}

impl iced_button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> iced_button::Appearance {
        if let Button::Custom { active, .. } = style {
            return active(self);
        }

        let corner_radii = &self.cosmic().corner_radii;
        let component = style.cosmic(self);
        iced_button::Appearance {
            border_radius: match style {
                Button::Link => corner_radii.radius_0.into(),
                Button::Card => corner_radii.radius_xs.into(),
                _ => corner_radii.radius_xl.into(),
            },
            border: Border {
                radius: match style {
                    Button::Link => corner_radii.radius_0.into(),
                    Button::Card => corner_radii.radius_xs.into(),
                    _ => corner_radii.radius_xl.into(),
                },
                ..Default::default()
            },
            background: match style {
                Button::Link | Button::Text => None,
                Button::LinkActive => Some(Background::Color(component.divider.into())),
                _ => Some(Background::Color(component.base.into())),
            },
            text_color: match style {
                Button::Link | Button::LinkActive => component.base.into(),
                _ => component.on.into(),
            },
            ..iced_button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced_button::Appearance {
        if let Button::Custom { hover, .. } = style {
            return hover(self);
        }

        let active = self.active(style);
        let component = style.cosmic(self);

        iced_button::Appearance {
            background: match style {
                Button::Link => None,
                Button::LinkActive => Some(Background::Color(component.divider.into())),
                _ => Some(Background::Color(component.hover.into())),
            },
            ..active
        }
    }

    fn disabled(&self, style: &Self::Style) -> iced_button::Appearance {
        let active = self.active(style);

        if matches!(style, Button::Card) {
            return active;
        }

        iced_button::Appearance {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                Background::Gradient(gradient) => Background::Gradient(gradient.mul_alpha(0.5)),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

/*
 * TODO: Checkbox
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checkbox {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::Primary
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let cosmic = self.cosmic();

        let corners = &cosmic.corner_radii;
        match style {
            Checkbox::Primary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.accent.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.accent.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.accent.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },

                text_color: None,
            },
            Checkbox::Secondary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.background.component.base.into()
                } else {
                    cosmic.background.base.into()
                }),
                icon_color: cosmic.background.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: cosmic.button.border.into(),
                },
                text_color: None,
            },
            Checkbox::Success => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.success.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.success.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.success.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
            Checkbox::Danger => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.destructive.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.destructive.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.destructive.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let cosmic = self.cosmic();
        let corners = &cosmic.corner_radii;

        match style {
            Checkbox::Primary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.accent.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.accent.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.accent.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
            Checkbox::Secondary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    self.current_container().base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: self.current_container().on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        self.current_container().base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
            Checkbox::Success => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.success.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.success.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.success.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
            Checkbox::Danger => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    cosmic.destructive.base.into()
                } else {
                    cosmic.button.base.into()
                }),
                icon_color: cosmic.destructive.on.into(),
                border: Border {
                    radius: corners.radius_xs.into(),
                    width: if is_checked { 0.0 } else { 1.0 },
                    color: if is_checked {
                        cosmic.destructive.base
                    } else {
                        cosmic.button.border
                    }
                    .into(),
                },
                text_color: None,
            },
        }
    }
}

/*
 * TODO: Container
 */
#[derive(Default)]
pub enum Container {
    WindowBackground,
    Background,
    Card,
    ContextDrawer,
    Custom(Box<dyn Fn(&Theme) -> container::Appearance>),
    Dialog,
    Dropdown,
    HeaderBar {
        focused: bool,
    },
    List,
    Primary,
    Secondary,
    Tooltip,
    #[default]
    Transparent,
}

impl Container {
    pub fn custom<F: Fn(&Theme) -> container::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }

    #[must_use]
    pub fn background(theme: &cosmic_theme::Theme) -> container::Appearance {
        container::Appearance {
            icon_color: Some(Color::from(theme.background.on)),
            text_color: Some(Color::from(theme.background.on)),
            background: Some(iced::Background::Color(theme.background.base.into())),
            border: Border {
                radius: theme.corner_radii.radius_xs.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
        }
    }

    #[must_use]
    pub fn primary(theme: &cosmic_theme::Theme) -> container::Appearance {
        container::Appearance {
            icon_color: Some(Color::from(theme.primary.on)),
            text_color: Some(Color::from(theme.primary.on)),
            background: Some(iced::Background::Color(theme.primary.base.into())),
            border: Border {
                radius: theme.corner_radii.radius_xs.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
        }
    }

    #[must_use]
    pub fn secondary(theme: &cosmic_theme::Theme) -> container::Appearance {
        container::Appearance {
            icon_color: Some(Color::from(theme.secondary.on)),
            text_color: Some(Color::from(theme.secondary.on)),
            background: Some(iced::Background::Color(theme.secondary.base.into())),
            border: Border {
                radius: theme.corner_radii.radius_xs.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = Container;

    #[allow(clippy::too_many_lines)]
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let cosmic = self.cosmic();

        match style {
            Container::Transparent => container::Appearance::default(),

            Container::Custom(f) => f(self),

            Container::WindowBackground => container::Appearance {
                icon_color: Some(Color::from(cosmic.background.on)),
                text_color: Some(Color::from(cosmic.background.on)),
                background: Some(iced::Background::Color(cosmic.background.base.into())),
                border: Border {
                    radius: [
                        cosmic.corner_radii.radius_0[0],
                        cosmic.corner_radii.radius_0[1],
                        cosmic.corner_radii.radius_s[2],
                        cosmic.corner_radii.radius_s[3],
                    ]
                    .into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
            },

            Container::List => {
                let component = &self.current_container().component;
                container::Appearance {
                    icon_color: Some(component.on.into()),
                    text_color: Some(component.on.into()),
                    background: Some(Background::Color(component.base.into())),
                    border: iced::Border {
                        radius: cosmic.corner_radii.radius_s.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                }
            }

            Container::HeaderBar { focused } => {
                let (icon_color, text_color) = if *focused {
                    (
                        Color::from(cosmic.accent.base),
                        Color::from(cosmic.background.on),
                    )
                } else {
                    use crate::ext::ColorExt;
                    let unfocused_color = Color::from(cosmic.background.component.on)
                        .blend_alpha(cosmic.background.base.into(), 0.5);
                    (unfocused_color, unfocused_color)
                };

                container::Appearance {
                    icon_color: Some(icon_color),
                    text_color: Some(text_color),
                    background: Some(iced::Background::Color(cosmic.background.base.into())),
                    border: Border {
                        radius: [
                            cosmic.corner_radii.radius_s[0],
                            cosmic.corner_radii.radius_s[1],
                            cosmic.corner_radii.radius_0[2],
                            cosmic.corner_radii.radius_0[3],
                        ]
                        .into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                }
            }

            Container::ContextDrawer => {
                let mut appearance = crate::style::Container::primary(cosmic);

                appearance.border = Border {
                    color: cosmic.primary.divider.into(),
                    width: 0.0,
                    radius: cosmic.corner_radii.radius_s.into(),
                };

                appearance.shadow = Shadow {
                    color: cosmic.shade.into(),
                    offset: Vector::new(0.0, 0.0),
                    blur_radius: 16.0,
                };

                appearance
            }

            Container::Background => Container::background(cosmic),

            Container::Primary => Container::primary(cosmic),

            Container::Secondary => Container::secondary(cosmic),

            Container::Dropdown => {
                let theme = self.cosmic();

                container::Appearance {
                    icon_color: None,
                    text_color: None,
                    background: Some(iced::Background::Color(theme.primary.base.into())),
                    border: Border {
                        radius: cosmic.corner_radii.radius_xs.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                }
            }

            Container::Tooltip => container::Appearance {
                icon_color: None,
                text_color: None,
                background: Some(iced::Background::Color(cosmic.palette.neutral_2.into())),
                border: Border {
                    radius: cosmic.corner_radii.radius_l.into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
            },

            Container::Card => {
                let cosmic = self.cosmic();

                match self.layer {
                    cosmic_theme::Layer::Background => container::Appearance {
                        icon_color: Some(Color::from(cosmic.background.component.on)),
                        text_color: Some(Color::from(cosmic.background.component.on)),
                        background: Some(iced::Background::Color(
                            cosmic.background.component.base.into(),
                        )),
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    cosmic_theme::Layer::Primary => container::Appearance {
                        icon_color: Some(Color::from(cosmic.primary.component.on)),
                        text_color: Some(Color::from(cosmic.primary.component.on)),
                        background: Some(iced::Background::Color(
                            cosmic.primary.component.base.into(),
                        )),
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    cosmic_theme::Layer::Secondary => container::Appearance {
                        icon_color: Some(Color::from(cosmic.secondary.component.on)),
                        text_color: Some(Color::from(cosmic.secondary.component.on)),
                        background: Some(iced::Background::Color(
                            cosmic.secondary.component.base.into(),
                        )),
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                }
            }

            Container::Dialog => container::Appearance {
                icon_color: Some(Color::from(cosmic.primary.on)),
                text_color: Some(Color::from(cosmic.primary.on)),
                background: Some(iced::Background::Color(cosmic.primary.base.into())),
                border: Border {
                    color: cosmic.primary.divider.into(),
                    width: 1.0,
                    radius: cosmic.corner_radii.radius_m.into(),
                },
                shadow: Shadow {
                    color: cosmic.shade.into(),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
            },
        }
    }
}

#[derive(Default)]
pub enum Slider {
    #[default]
    Standard,
    Custom {
        active: Rc<dyn Fn(&Theme) -> slider::Appearance>,
        hovered: Rc<dyn Fn(&Theme) -> slider::Appearance>,
        dragging: Rc<dyn Fn(&Theme) -> slider::Appearance>,
    },
}

/*
 * Slider
 */
impl slider::StyleSheet for Theme {
    type Style = Slider;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Standard =>
            //TODO: no way to set rail thickness
            {
                let cosmic: &cosmic_theme::Theme = self.cosmic();

                slider::Appearance {
                    rail: Rail {
                        colors: slider::RailBackground::Pair(
                            cosmic.accent.base.into(),
                            cosmic.palette.neutral_6.into(),
                        ),
                        width: 4.0,
                        border_radius: cosmic.corner_radii.radius_xs.into(),
                    },

                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle {
                            height: 20,
                            width: 20,
                            border_radius: cosmic.corner_radii.radius_m.into(),
                        },
                        color: cosmic.accent.base.into(),
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                    },

                    breakpoint: slider::Breakpoint {
                        color: cosmic.on_bg_color().into(),
                    },
                }
            }
            Slider::Custom { active, .. } => active(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Standard => {
                let cosmic: &cosmic_theme::Theme = self.cosmic();

                let mut style = self.active(style);
                style.handle.shape = slider::HandleShape::Rectangle {
                    height: 26,
                    width: 26,
                    border_radius: cosmic.corner_radii.radius_m.into(),
                };
                style.handle.border_width = 3.0;
                let mut border_color = self.cosmic().palette.neutral_10;
                border_color.alpha = 0.1;
                style.handle.border_color = border_color.into();
                style
            }
            Slider::Custom { hovered, .. } => hovered(self),
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Standard => {
                let mut style = self.hovered(style);
                let mut border_color = self.cosmic().palette.neutral_10;
                border_color.alpha = 0.2;
                style.handle.border_color = border_color.into();

                style
            }
            Slider::Custom { dragging, .. } => dragging(self),
        }
    }
}

/*
 * TODO: Menu
 */
impl menu::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
        let cosmic = self.cosmic();

        menu::Appearance {
            text_color: cosmic.on_bg_color().into(),
            background: Background::Color(cosmic.background.base.into()),
            border: Border {
                radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            selected_text_color: cosmic.accent.base.into(),
            selected_background: Background::Color(cosmic.background.component.hover.into()),
        }
    }
}

/*
 * TODO: Pick List
 */
impl pick_list::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &()) -> pick_list::Appearance {
        let cosmic = &self.cosmic();

        pick_list::Appearance {
            text_color: cosmic.on_bg_color().into(),
            background: Color::TRANSPARENT.into(),
            placeholder_color: cosmic.on_bg_color().into(),
            border: Border {
                radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            // icon_size: 0.7, // TODO: how to replace
            handle_color: cosmic.on_bg_color().into(),
        }
    }

    fn hovered(&self, style: &()) -> pick_list::Appearance {
        let cosmic = &self.cosmic();

        pick_list::Appearance {
            background: Background::Color(cosmic.background.base.into()),
            ..self.active(style)
        }
    }
}

/*
 * TODO: Radio
 */
impl radio::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style, is_selected: bool) -> radio::Appearance {
        let theme = self.cosmic();

        radio::Appearance {
            background: if is_selected {
                Color::from(theme.accent.base).into()
            } else {
                // TODO: this seems to be defined weirdly in FIGMA
                Color::from(theme.background.base).into()
            },
            dot_color: theme.accent.on.into(),
            border_width: 1.0,
            border_color: if is_selected {
                Color::from(theme.accent.base)
            } else {
                // TODO: this seems to be defined weirdly in FIGMA
                Color::from(theme.palette.neutral_7)
            },
            text_color: None,
        }
    }

    fn hovered(&self, _style: &Self::Style, is_selected: bool) -> radio::Appearance {
        let theme = self.cosmic();
        let mut neutral_10 = theme.palette.neutral_10;
        neutral_10.alpha = 0.1;

        radio::Appearance {
            background: if is_selected {
                Color::from(theme.accent.base).into()
            } else {
                // TODO: this seems to be defined weirdly in FIGMA
                Color::from(neutral_10).into()
            },
            dot_color: theme.accent.on.into(),
            border_width: 1.0,
            border_color: if is_selected {
                Color::from(theme.accent.base)
            } else {
                // TODO: this seems to be defined weirdly in FIGMA
                Color::from(theme.palette.neutral_7)
            },
            text_color: None,
        }
    }
}

/*
 * Toggler
 */
impl toggler::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let theme = self.cosmic();
        const HANDLE_MARGIN: f32 = 2.0;
        toggler::Appearance {
            background: if is_active {
                theme.accent.base.into()
            } else {
                theme.palette.neutral_5.into()
            },
            background_border: None,
            foreground: theme.palette.neutral_2.into(),
            foreground_border: None,
            border_radius: theme.radius_xl().into(),
            handle_radius: theme
                .radius_xl()
                .map(|x| (x - HANDLE_MARGIN).max(0.0))
                .into(),
            handle_margin: HANDLE_MARGIN,
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let cosmic = self.cosmic();
        //TODO: grab colors from palette
        let mut neutral_10 = cosmic.palette.neutral_10;
        neutral_10.alpha = 0.1;

        toggler::Appearance {
            background: if is_active {
                over(neutral_10, cosmic.accent_color())
            } else {
                over(neutral_10, cosmic.palette.neutral_5)
            }
            .into(),
            ..self.active(style, is_active)
        }
    }
}

/*
 * TODO: Pane Grid
 */
impl pane_grid::StyleSheet for Theme {
    type Style = ();

    fn picked_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        let theme = self.cosmic();

        Some(pane_grid::Line {
            color: theme.accent.base.into(),
            width: 2.0,
        })
    }

    fn hovered_split(&self, _style: &Self::Style) -> Option<pane_grid::Line> {
        let theme = self.cosmic();

        Some(pane_grid::Line {
            color: theme.accent.hover.into(),
            width: 2.0,
        })
    }

    fn hovered_region(&self, _style: &Self::Style) -> pane_grid::Appearance {
        let theme = self.cosmic();
        pane_grid::Appearance {
            background: Background::Color(theme.bg_color().into()),
            border: Border {
                radius: theme.corner_radii.radius_0.into(),
                width: 2.0,
                color: theme.bg_divider().into(),
            },
        }
    }
}

/*
 * TODO: Progress Bar
 */
#[derive(Default)]
pub enum ProgressBar {
    #[default]
    Primary,
    Success,
    Danger,
    Custom(Box<dyn Fn(&Theme) -> progress_bar::Appearance>),
}

impl ProgressBar {
    pub fn custom<F: Fn(&Theme) -> progress_bar::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ProgressBar;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        let theme = self.cosmic();

        match style {
            ProgressBar::Primary => progress_bar::Appearance {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.accent.base).into(),
                border_radius: theme.corner_radii.radius_xs.into(),
            },
            ProgressBar::Success => progress_bar::Appearance {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.success.base).into(),
                border_radius: theme.corner_radii.radius_xs.into(),
            },
            ProgressBar::Danger => progress_bar::Appearance {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.destructive.base).into(),
                border_radius: theme.corner_radii.radius_xs.into(),
            },
            ProgressBar::Custom(f) => f(self),
        }
    }
}

/*
 * TODO: Rule
 */
#[derive(Default)]
pub enum Rule {
    #[default]
    Default,
    LightDivider,
    HeavyDivider,
    Custom(Box<dyn Fn(&Theme) -> rule::Appearance>),
}

impl Rule {
    pub fn custom<F: Fn(&Theme) -> rule::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        match style {
            Rule::Default => rule::Appearance {
                color: self.current_container().divider.into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            },
            Rule::LightDivider => rule::Appearance {
                color: self.current_container().divider.into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Padded(10),
            },
            Rule::HeavyDivider => rule::Appearance {
                color: self.current_container().divider.into(),
                width: 4,
                radius: 4.0.into(),
                fill_mode: rule::FillMode::Full,
            },
            Rule::Custom(f) => f(self),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum Scrollable {
    #[default]
    Permanent,
    Minimal,
}

/*
 * TODO: Scrollable
 */
impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Scrollbar {
        let cosmic = self.cosmic();
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.7;
        let mut a = scrollable::Scrollbar {
            background: None,
            border: Border {
                radius: cosmic.corner_radii.radius_s.into(),
                ..Default::default()
            },
            scroller: scrollable::Scroller {
                color: neutral_5.into(),
                border: Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    ..Default::default()
                },
            },
        };

        if matches!(style, Scrollable::Permanent) {
            let mut neutral_3 = cosmic.palette.neutral_3;
            neutral_3.alpha = 0.7;
            a.background = Some(Background::Color(neutral_3.into()));
        }

        a
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        let cosmic = self.cosmic();
        let mut neutral_5 = cosmic.palette.neutral_5;
        neutral_5.alpha = 0.7;

        if is_mouse_over_scrollbar {
            let mut hover_overlay = cosmic.palette.neutral_0;
            hover_overlay.alpha = 0.2;
            neutral_5 = over(hover_overlay, neutral_5);
        }
        let mut a = scrollable::Scrollbar {
            background: None,
            border: Border {
                radius: cosmic.corner_radii.radius_s.into(),
                ..Default::default()
            },
            scroller: scrollable::Scroller {
                color: neutral_5.into(),
                border: Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    ..Default::default()
                },
            },
        };
        if matches!(style, Scrollable::Permanent) {
            let mut neutral_3 = cosmic.palette.neutral_3;
            neutral_3.alpha = 0.7;
            a.background = Some(Background::Color(neutral_3.into()));
        }

        a
    }
}

#[derive(Clone, Default)]
pub enum Svg {
    /// Apply a custom appearance filter
    Custom(Rc<dyn Fn(&Theme) -> svg::Appearance>),
    /// No filtering is applied
    #[default]
    Default,
}

impl Svg {
    pub fn custom<F: Fn(&Theme) -> svg::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Rc::new(f))
    }
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        #[allow(clippy::match_same_arms)]
        match style {
            Svg::Default => svg::Appearance::default(),
            Svg::Custom(appearance) => appearance(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> svg::Appearance {
        self.appearance(style)
    }
}

/*
 * TODO: Text
 */
#[derive(Clone, Copy, Default)]
pub enum Text {
    Accent,
    #[default]
    Default,
    Color(Color),
    // TODO: Can't use dyn Fn since this must be copy
    Custom(fn(&Theme) -> iced_widget::text::Appearance),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl iced_widget::text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> iced_widget::text::Appearance {
        match style {
            Text::Accent => iced_widget::text::Appearance {
                color: Some(self.cosmic().accent.base.into()),
            },
            Text::Default => iced_widget::text::Appearance { color: None },
            Text::Color(c) => iced_widget::text::Appearance { color: Some(c) },
            Text::Custom(f) => f(self),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub enum TextInput {
    #[default]
    Default,
    Search,
}

/*
 * TODO: Text Input
 */
impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;
        match style {
            TextInput::Default => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: self.current_container().component.divider.into(),
                },
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_m.into(),
                    ..Default::default()
                },
                icon_color: self.current_container().on.into(),
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;

        match style {
            TextInput::Default => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: self.current_container().on.into(),
                },
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_m.into(),
                    ..Default::default()
                },
                icon_color: self.current_container().on.into(),
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;

        match style {
            TextInput::Default => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: palette.accent.base.into(),
                },
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_m.into(),
                    ..Default::default()
                },
                icon_color: self.current_container().on.into(),
            },
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();
        let mut neutral_9 = palette.palette.neutral_9;
        neutral_9.alpha = 0.7;
        neutral_9.into()
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();

        palette.palette.neutral_9.into()
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();

        palette.accent.base.into()
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();
        let mut neutral_9 = palette.palette.neutral_9;
        neutral_9.alpha = 0.5;
        neutral_9.into()
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }
}

impl crate::widget::card::style::StyleSheet for Theme {
    fn default(&self) -> crate::widget::card::style::Appearance {
        let cosmic = self.cosmic();

        match self.layer {
            cosmic_theme::Layer::Background => crate::widget::card::style::Appearance {
                card_1: Background::Color(cosmic.background.component.hover.into()),
                card_2: Background::Color(cosmic.background.component.pressed.into()),
            },
            cosmic_theme::Layer::Primary => crate::widget::card::style::Appearance {
                card_1: Background::Color(cosmic.primary.component.hover.into()),
                card_2: Background::Color(cosmic.primary.component.pressed.into()),
            },
            cosmic_theme::Layer::Secondary => crate::widget::card::style::Appearance {
                card_1: Background::Color(cosmic.secondary.component.hover.into()),
                card_2: Background::Color(cosmic.secondary.component.pressed.into()),
            },
        }
    }
}

#[derive(Default)]
pub enum TextEditor {
    #[default]
    Default,
    Custom(Box<dyn iced_style::text_editor::StyleSheet<Style = Theme>>),
}

impl iced_style::text_editor::StyleSheet for Theme {
    type Style = TextEditor;

    fn active(&self, style: &Self::Style) -> iced_style::text_editor::Appearance {
        if let TextEditor::Custom(style) = style {
            return style.active(self);
        }

        let cosmic = self.cosmic();
        iced_style::text_editor::Appearance {
            background: iced::Color::from(cosmic.bg_color()).into(),
            border: Border {
                radius: cosmic.corner_radii.radius_0.into(),
                width: f32::from(cosmic.space_xxxs()),
                color: iced::Color::from(cosmic.bg_divider()),
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> iced_style::text_editor::Appearance {
        if let TextEditor::Custom(style) = style {
            return style.focused(self);
        }

        let cosmic = self.cosmic();
        iced_style::text_editor::Appearance {
            background: iced::Color::from(cosmic.bg_color()).into(),
            border: Border {
                radius: cosmic.corner_radii.radius_0.into(),
                width: f32::from(cosmic.space_xxxs()),
                color: iced::Color::from(cosmic.accent.base),
            },
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(style) = style {
            return style.placeholder_color(self);
        }
        let palette = self.cosmic();
        let mut neutral_9 = palette.palette.neutral_9;
        neutral_9.alpha = 0.7;
        neutral_9.into()
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(style) = style {
            return style.value_color(self);
        }
        let palette = self.cosmic();

        palette.palette.neutral_9.into()
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(style) = style {
            return style.disabled_color(self);
        }
        let palette = self.cosmic();
        let mut neutral_9 = palette.palette.neutral_9;
        neutral_9.alpha = 0.5;
        neutral_9.into()
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(style) = style {
            return style.selection_color(self);
        }
        let cosmic = self.cosmic();
        cosmic.accent.base.into()
    }

    fn disabled(&self, style: &Self::Style) -> iced_style::text_editor::Appearance {
        if let TextEditor::Custom(style) = style {
            return style.disabled(self);
        }
        self.active(style)
    }
}
