// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains stylesheet implementations for widgets native to iced.

use crate::theme::{CosmicComponent, Theme, TRANSPARENT_COMPONENT};
use cosmic_theme::composite::over;
use iced::{
    overlay::menu,
    widget::{
        button as iced_button, checkbox as iced_checkbox, container as iced_container, pane_grid,
        pick_list, progress_bar, radio, rule, scrollable,
        slider::{self, Rail},
        svg, toggler,
    },
    Gradient,
};
use iced_core::{Background, Border, Color, Shadow, Vector};
use iced_widget::{pane_grid::Highlight, text_editor, text_input};
use std::rc::Rc;

pub mod application {
    use crate::Theme;
    use iced_runtime::Appearance;

    #[derive(Default)]
    pub enum Application {
        #[default]
        Default,
        Custom(Box<dyn Fn(&Theme) -> Appearance>),
    }

    impl Application {
        pub fn custom<F: Fn(&Theme) -> Appearance + 'static>(f: F) -> Self {
            Self::Custom(Box::new(f))
        }
    }

    pub fn appearance(theme: &Theme) -> Appearance {
        let cosmic = theme.cosmic();

        Appearance {
            icon_color: cosmic.bg_color().into(),
            background_color: cosmic.bg_color().into(),
            text_color: cosmic.on_bg_color().into(),
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
    Custom(Box<dyn Fn(&Theme, iced_button::Status) -> iced_button::Style>),
}

impl iced_button::Catalog for Theme {
    type Class<'a> = Button;

    fn default<'a>() -> Self::Class<'a> {
        Button::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: iced_button::Status) -> iced_button::Style {
        if let Button::Custom(f) = class {
            return f(self, status);
        }
        let cosmic = self.cosmic();
        let corner_radii = &cosmic.corner_radii;
        let component = class.cosmic(self);

        let mut appearance = iced_button::Style {
            border_radius: match class {
                Button::Link => corner_radii.radius_0.into(),
                Button::Card => corner_radii.radius_xs.into(),
                _ => corner_radii.radius_xl.into(),
            },
            border: Border {
                radius: match class {
                    Button::Link => corner_radii.radius_0.into(),
                    Button::Card => corner_radii.radius_xs.into(),
                    _ => corner_radii.radius_xl.into(),
                },
                ..Default::default()
            },
            background: match class {
                Button::Link | Button::Text => None,
                Button::LinkActive => Some(Background::Color(component.divider.into())),
                _ => Some(Background::Color(component.base.into())),
            },
            text_color: match class {
                Button::Link | Button::LinkActive => component.base.into(),
                _ => component.on.into(),
            },
            ..iced_button::Style::default()
        };

        match status {
            iced_button::Status::Active => {}
            iced_button::Status::Hovered => {
                appearance.background = match class {
                    Button::Link => None,
                    Button::LinkActive => Some(Background::Color(component.divider.into())),
                    _ => Some(Background::Color(component.hover.into())),
                };
            }
            iced_button::Status::Pressed => {
                appearance.background = match class {
                    Button::Link => None,
                    Button::LinkActive => Some(Background::Color(component.divider.into())),
                    _ => Some(Background::Color(component.pressed.into())),
                };
            }
            iced_button::Status::Disabled => {
                // Card color is not transparent when it isn't clickable
                if matches!(class, Button::Card) {
                    return appearance;
                }
                appearance.background = appearance.background.map(|background| match background {
                    Background::Color(color) => Background::Color(Color {
                        a: color.a * 0.5,
                        ..color
                    }),
                    Background::Gradient(gradient) => {
                        Background::Gradient(gradient.scale_alpha(0.5))
                    }
                });
                appearance.text_color = Color {
                    a: appearance.text_color.a * 0.5,
                    ..appearance.text_color
                };
            }
        };
        appearance
    }
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

impl iced_checkbox::Catalog for Theme {
    type Class<'a> = Checkbox;

    fn default<'a>() -> Self::Class<'a> {
        Checkbox::default()
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        status: iced_checkbox::Status,
    ) -> iced_checkbox::Style {
        let cosmic = self.cosmic();

        let corners = &cosmic.corner_radii;

        let disabled = matches!(status, iced_checkbox::Status::Disabled { .. });
        match status {
            iced_checkbox::Status::Active { is_checked }
            | iced_checkbox::Status::Disabled { is_checked } => {
                let mut active = match class {
                    Checkbox::Primary => iced_checkbox::Style {
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
                    Checkbox::Secondary => iced_checkbox::Style {
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
                    Checkbox::Success => iced_checkbox::Style {
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
                    Checkbox::Danger => iced_checkbox::Style {
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
                };
                if disabled {
                    match &mut active.background {
                        Background::Color(color) => {
                            color.a /= 2.;
                        }
                        Background::Gradient(gradient) => {
                            *gradient = gradient.scale_alpha(0.5);
                        }
                    }
                    if let Some(c) = active.text_color.as_mut() {
                        c.a /= 2.
                    };
                    active.border.color.a /= 2.;
                }
                active
            }
            iced_checkbox::Status::Hovered { is_checked } => match class {
                Checkbox::Primary => iced_checkbox::Style {
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
                Checkbox::Secondary => iced_checkbox::Style {
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
                Checkbox::Success => iced_checkbox::Style {
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
                Checkbox::Danger => iced_checkbox::Style {
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
            },
        }
    }
}

/*
 * TODO: Container
 */
#[derive(Default)]
pub enum Container<'a> {
    WindowBackground,
    Background,
    Card,
    ContextDrawer,
    Custom(Box<dyn Fn(&Theme) -> iced_container::Style + 'a>),
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

impl<'a> Container<'a> {
    pub fn custom<F: Fn(&Theme) -> iced_container::Style + 'a>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }

    #[must_use]
    pub fn background(theme: &cosmic_theme::Theme) -> iced_container::Style {
        iced_container::Style {
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
    pub fn primary(theme: &cosmic_theme::Theme) -> iced_container::Style {
        iced_container::Style {
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
    pub fn secondary(theme: &cosmic_theme::Theme) -> iced_container::Style {
        iced_container::Style {
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

impl<'a> From<iced_container::StyleFn<'a, Theme>> for Container<'a> {
    fn from(value: iced_container::StyleFn<'a, Theme>) -> Self {
        Self::custom(value)
    }
}

impl iced_container::Catalog for Theme {
    type Class<'a> = Container<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Container::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> iced_container::Style {
        let cosmic = self.cosmic();

        match class {
            Container::Transparent => iced_container::Style::default(),

            Container::Custom(f) => f(self),

            Container::WindowBackground => iced_container::Style {
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
                iced_container::Style {
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

                iced_container::Style {
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

                iced_container::Style {
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

            Container::Tooltip => iced_container::Style {
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
                    cosmic_theme::Layer::Background => iced_container::Style {
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
                    cosmic_theme::Layer::Primary => iced_container::Style {
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
                    cosmic_theme::Layer::Secondary => iced_container::Style {
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

            Container::Dialog => iced_container::Style {
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
        active: Rc<dyn Fn(&Theme) -> slider::Style>,
        hovered: Rc<dyn Fn(&Theme) -> slider::Style>,
        dragging: Rc<dyn Fn(&Theme) -> slider::Style>,
    },
}

/*
 * Slider
 */
impl slider::Catalog for Theme {
    type Class<'a> = Slider;

    fn default<'a>() -> Self::Class<'a> {
        Slider::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: slider::Status) -> slider::Style {
        let cosmic: &cosmic_theme::Theme = self.cosmic();
        let mut appearance = match class {
            Slider::Standard =>
            //TODO: no way to set rail thickness
            {
                slider::Style {
                    rail: Rail {
                        backgrounds: (
                            Background::Color(cosmic.accent.base.into()),
                            Background::Color(cosmic.palette.neutral_6.into()),
                        ),
                        border: Border {
                            radius: cosmic.corner_radii.radius_xs.into(),
                            ..Border::default()
                        },
                        width: 4.0,
                    },

                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle {
                            height: 20,
                            width: 20,
                            border_radius: cosmic.corner_radii.radius_m.into(),
                        },
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                        background: Background::Color(cosmic.accent.base.into()),
                    },

                    breakpoint: slider::Breakpoint {
                        color: cosmic.on_bg_color().into(),
                    },
                }
            }
            Slider::Custom { active, .. } => active(self),
        };
        match status {
            slider::Status::Active => appearance,
            slider::Status::Hovered => match class {
                Slider::Standard => {
                    appearance.handle.shape = slider::HandleShape::Rectangle {
                        height: 26,
                        width: 26,
                        border_radius: cosmic.corner_radii.radius_m.into(),
                    };
                    appearance.handle.border_width = 3.0;
                    let mut border_color = self.cosmic().palette.neutral_10;
                    border_color.alpha = 0.1;
                    appearance.handle.border_color = border_color.into();
                    appearance
                }
                Slider::Custom { hovered, .. } => hovered(self),
            },
            slider::Status::Dragged => match class {
                Slider::Standard => {
                    let mut style = {
                        appearance.handle.shape = slider::HandleShape::Rectangle {
                            height: 26,
                            width: 26,
                            border_radius: cosmic.corner_radii.radius_m.into(),
                        };
                        appearance.handle.border_width = 3.0;
                        let mut border_color = self.cosmic().palette.neutral_10;
                        border_color.alpha = 0.1;
                        appearance.handle.border_color = border_color.into();
                        appearance
                    };
                    let mut border_color = self.cosmic().palette.neutral_10;
                    border_color.alpha = 0.2;
                    style.handle.border_color = border_color.into();

                    style
                }
                Slider::Custom { dragging, .. } => dragging(self),
            },
        }
    }
}

impl menu::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, class: &<Self as menu::Catalog>::Class<'_>) -> menu::Style {
        let cosmic = self.cosmic();

        menu::Style {
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
impl pick_list::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        ()
    }

    fn style(
        &self,
        class: &<Self as pick_list::Catalog>::Class<'_>,
        status: pick_list::Status,
    ) -> pick_list::Style {
        let cosmic = &self.cosmic();

        let appearance = pick_list::Style {
            text_color: cosmic.on_bg_color().into(),
            background: Color::TRANSPARENT.into(),
            placeholder_color: cosmic.on_bg_color().into(),
            border: Border {
                radius: cosmic.corner_radii.radius_m.into(),
                ..Default::default()
            },
            // icon_size: 0.7, // TODO: how to replace
            handle_color: cosmic.on_bg_color().into(),
        };
        match status {
            pick_list::Status::Active => appearance,
            pick_list::Status::Hovered => pick_list::Style {
                background: Background::Color(cosmic.background.base.into()),
                ..appearance
            },
            pick_list::Status::Opened => appearance,
        }
    }
}

/*
 * TODO: Radio
 */
impl radio::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, class: &Self::Class<'_>, status: radio::Status) -> radio::Style {
        let theme = self.cosmic();
        let mut neutral_10 = theme.palette.neutral_10;
        neutral_10.alpha = 0.1;

        match status {
            radio::Status::Active { is_selected } => radio::Style {
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
            },
            radio::Status::Hovered { is_selected } => radio::Style {
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
            },
        }
    }
}

/*
 * Toggler
 */
impl toggler::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
        let cosmic = self.cosmic();
        const HANDLE_MARGIN: f32 = 2.0;
        let mut neutral_10 = cosmic.palette.neutral_10;
        neutral_10.alpha = 0.1;

        let mut active = toggler::Style {
            background: if matches!(status, toggler::Status::Active { is_toggled: true }) {
                cosmic.accent.base.into()
            } else {
                cosmic.palette.neutral_5.into()
            },
            foreground: cosmic.palette.neutral_2.into(),
            border_radius: cosmic.radius_xl().into(),
            handle_radius: cosmic
                .radius_xl()
                .map(|x| (x - HANDLE_MARGIN).max(0.0))
                .into(),
            handle_margin: HANDLE_MARGIN,
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
        };
        match status {
            toggler::Status::Active { is_toggled } => active,
            toggler::Status::Hovered { is_toggled } => {
                let is_active = matches!(status, toggler::Status::Hovered { is_toggled: true });
                toggler::Style {
                    background: if is_active {
                        over(neutral_10, cosmic.accent_color())
                    } else {
                        over(neutral_10, cosmic.palette.neutral_5)
                    }
                    .into(),
                    ..active
                }
            }
            toggler::Status::Disabled => {
                active.background.a /= 2.;
                active.foreground.a /= 2.;
                active
            }
        }
    }
}

/*
 * TODO: Pane Grid
 */
impl pane_grid::Catalog for Theme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as pane_grid::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, class: &<Self as pane_grid::Catalog>::Class<'_>) -> pane_grid::Style {
        let theme = self.cosmic();

        pane_grid::Style {
            hovered_region: Highlight {
                background: Background::Color(theme.bg_color().into()),
                border: Border {
                    radius: theme.corner_radii.radius_0.into(),
                    width: 2.0,
                    color: theme.bg_divider().into(),
                },
            },
            picked_split: pane_grid::Line {
                color: theme.accent.base.into(),
                width: 2.0,
            },
            hovered_split: pane_grid::Line {
                color: theme.accent.hover.into(),
                width: 2.0,
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
    Custom(Box<dyn Fn(&Theme) -> progress_bar::Style>),
}

impl ProgressBar {
    pub fn custom<F: Fn(&Theme) -> progress_bar::Style + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl progress_bar::Catalog for Theme {
    type Class<'a> = ProgressBar;

    fn default<'a>() -> Self::Class<'a> {
        ProgressBar::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> progress_bar::Style {
        let theme = self.cosmic();

        match class {
            ProgressBar::Primary => progress_bar::Style {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.accent.base).into(),
                border: Border {
                    color: Color::default(),
                    width: 0.0,
                    radius: theme.corner_radii.radius_xs.into(),
                },
            },
            ProgressBar::Success => progress_bar::Style {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.success.base).into(),
                border: Border {
                    color: Color::default(),
                    width: 0.0,
                    radius: theme.corner_radii.radius_xs.into(),
                },
            },
            ProgressBar::Danger => progress_bar::Style {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.destructive.base).into(),
                border: Border {
                    color: Color::default(),
                    width: 0.0,
                    radius: theme.corner_radii.radius_xs.into(),
                },
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
    Custom(Box<dyn Fn(&Theme) -> rule::Style>),
}

impl Rule {
    pub fn custom<F: Fn(&Theme) -> rule::Style + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl rule::Catalog for Theme {
    type Class<'a> = Rule;

    fn default<'a>() -> Self::Class<'a> {
        Rule::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        match class {
            Rule::Default => rule::Style {
                color: self.current_container().divider.into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            },
            Rule::LightDivider => rule::Style {
                color: self.current_container().divider.into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Padded(8),
            },
            Rule::HeavyDivider => rule::Style {
                color: self.current_container().divider.into(),
                width: 4,
                radius: 2.0.into(),
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
impl scrollable::Catalog for Theme {
    type Class<'a> = Scrollable;

    fn default<'a>() -> Self::Class<'a> {
        Scrollable::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        match status {
            scrollable::Status::Active => {
                let cosmic = self.cosmic();
                let mut neutral_5 = cosmic.palette.neutral_5;
                neutral_5.alpha = 0.7;
                let mut a = scrollable::Style {
                    container: iced_container::transparent(self),
                    vertical_rail: scrollable::Rail {
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        background: None,
                        scroller: scrollable::Scroller {
                            color: neutral_5.into(),
                            border: Border {
                                radius: cosmic.corner_radii.radius_s.into(),
                                ..Default::default()
                            },
                        },
                    },
                    horizontal_rail: scrollable::Rail {
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        background: None,
                        scroller: scrollable::Scroller {
                            color: neutral_5.into(),
                            border: Border {
                                radius: cosmic.corner_radii.radius_s.into(),
                                ..Default::default()
                            },
                        },
                    },
                    gap: None,
                };

                if matches!(class, Scrollable::Permanent) {
                    let mut neutral_3 = cosmic.palette.neutral_3;
                    neutral_3.alpha = 0.7;
                    a.horizontal_rail.background = Some(Background::Color(neutral_3.into()));
                    a.vertical_rail.background = Some(Background::Color(neutral_3.into()));
                }

                a
            }
            // TODO handle vertical / horizontal
            scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => {
                let cosmic = self.cosmic();
                let mut neutral_5 = cosmic.palette.neutral_5;
                neutral_5.alpha = 0.7;

                // TODO hover

                // if is_mouse_over_scrollbar {
                //     let mut hover_overlay = cosmic.palette.neutral_0;
                //     hover_overlay.alpha = 0.2;
                //     neutral_5 = over(hover_overlay, neutral_5);
                // }
                let mut a: scrollable::Style = scrollable::Style {
                    container: iced_container::Style::default(),
                    vertical_rail: scrollable::Rail {
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        background: None,
                        scroller: scrollable::Scroller {
                            color: neutral_5.into(),
                            border: Border {
                                radius: cosmic.corner_radii.radius_s.into(),
                                ..Default::default()
                            },
                        },
                    },
                    horizontal_rail: scrollable::Rail {
                        border: Border {
                            radius: cosmic.corner_radii.radius_s.into(),
                            ..Default::default()
                        },
                        background: None,
                        scroller: scrollable::Scroller {
                            color: neutral_5.into(),
                            border: Border {
                                radius: cosmic.corner_radii.radius_s.into(),
                                ..Default::default()
                            },
                        },
                    },
                    gap: None,
                };

                if matches!(class, Scrollable::Permanent) {
                    let mut neutral_3 = cosmic.palette.neutral_3;
                    neutral_3.alpha = 0.7;
                    a.horizontal_rail.background = Some(Background::Color(neutral_3.into()));
                    a.vertical_rail.background = Some(Background::Color(neutral_3.into()));
                }

                a
            }
        }
    }
}

#[derive(Clone, Default)]
pub enum Svg {
    /// Apply a custom appearance filter
    Custom(Rc<dyn Fn(&Theme) -> svg::Style>),
    /// No filtering is applied
    #[default]
    Default,
}

impl Svg {
    pub fn custom<F: Fn(&Theme) -> svg::Style + 'static>(f: F) -> Self {
        Self::Custom(Rc::new(f))
    }
}

impl svg::Catalog for Theme {
    type Class<'a> = Svg;

    fn default<'a>() -> Self::Class<'a> {
        Svg::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: svg::Status) -> svg::Style {
        #[allow(clippy::match_same_arms)]
        match class {
            Svg::Default => svg::Style::default(),
            Svg::Custom(appearance) => appearance(self),
        }
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
    Custom(fn(&Theme) -> iced_widget::text::Style),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Self::Color(color)
    }
}

impl iced_widget::text::Catalog for Theme {
    type Class<'a> = Text;

    fn default<'a>() -> Self::Class<'a> {
        Text::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> iced_widget::text::Style {
        match class {
            Text::Accent => iced_widget::text::Style {
                color: Some(self.cosmic().accent.base.into()),
            },
            Text::Default => iced_widget::text::Style { color: None },
            Text::Color(c) => iced_widget::text::Style { color: Some(*c) },
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
impl text_input::Catalog for Theme {
    type Class<'a> = TextInput;

    fn default<'a>() -> Self::Class<'a> {
        TextInput::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;

        let mut neutral_9 = palette.palette.neutral_9;
        let value = neutral_9.into();
        neutral_9.alpha = 0.7;
        let placeholder = neutral_9.into();
        let selection = palette.accent.base.into();

        let mut appearance = match class {
            TextInput::Default => text_input::Style {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: self.current_container().component.divider.into(),
                },
                icon: self.current_container().on.into(),
                placeholder,
                value,
                selection,
            },
            TextInput::Search => text_input::Style {
                background: Color::from(bg).into(),
                border: Border {
                    radius: palette.corner_radii.radius_m.into(),
                    ..Default::default()
                },
                icon: self.current_container().on.into(),
                placeholder,
                value,
                selection,
            },
        };

        match status {
            text_input::Status::Active => appearance,
            text_input::Status::Hovered => {
                let mut bg = palette.palette.neutral_7;
                bg.alpha = 0.25;

                match class {
                    TextInput::Default => text_input::Style {
                        background: Color::from(bg).into(),
                        border: Border {
                            radius: palette.corner_radii.radius_s.into(),
                            width: 1.0,
                            color: self.current_container().on.into(),
                        },
                        icon: self.current_container().on.into(),
                        placeholder,
                        value,
                        selection,
                    },
                    TextInput::Search => text_input::Style {
                        background: Color::from(bg).into(),
                        border: Border {
                            radius: palette.corner_radii.radius_m.into(),
                            ..Default::default()
                        },
                        icon: self.current_container().on.into(),
                        placeholder,
                        value,
                        selection,
                    },
                }
            }
            text_input::Status::Focused => {
                let mut bg = palette.palette.neutral_7;
                bg.alpha = 0.25;

                match class {
                    TextInput::Default => text_input::Style {
                        background: Color::from(bg).into(),
                        border: Border {
                            radius: palette.corner_radii.radius_s.into(),
                            width: 1.0,
                            color: palette.accent.base.into(),
                        },
                        icon: self.current_container().on.into(),
                        placeholder,
                        value,
                        selection,
                    },
                    TextInput::Search => text_input::Style {
                        background: Color::from(bg).into(),
                        border: Border {
                            radius: palette.corner_radii.radius_m.into(),
                            ..Default::default()
                        },
                        icon: self.current_container().on.into(),
                        placeholder,
                        value,
                        selection,
                    },
                }
            }
            text_input::Status::Disabled => {
                appearance.background = match appearance.background {
                    Background::Color(color) => Background::Color(Color {
                        a: color.a * 0.5,
                        ..color
                    }),
                    Background::Gradient(gradient) => {
                        Background::Gradient(gradient.scale_alpha(0.5))
                    }
                };
                appearance.border.color.a /= 2.;
                appearance.icon.a /= 2.;
                appearance.placeholder.a /= 2.;
                appearance.value.a /= 2.;
                appearance
            }
        }
    }
}

// TODO card
// impl crate::widget::card::style::StyleSheet for Theme {
//     fn default(&self) -> crate::widget::card::style::Style {
//         let cosmic = self.cosmic();

//         match self.layer {
//             cosmic_theme::Layer::Background => crate::widget::card::style::Style {
//                 card_1: Background::Color(cosmic.background.component.hover.into()),
//                 card_2: Background::Color(cosmic.background.component.pressed.into()),
//             },
//             cosmic_theme::Layer::Primary => crate::widget::card::style::Style {
//                 card_1: Background::Color(cosmic.primary.component.hover.into()),
//                 card_2: Background::Color(cosmic.primary.component.pressed.into()),
//             },
//             cosmic_theme::Layer::Secondary => crate::widget::card::style::Style {
//                 card_1: Background::Color(cosmic.secondary.component.hover.into()),
//                 card_2: Background::Color(cosmic.secondary.component.pressed.into()),
//             },
//         }
//     }
// }

#[derive(Default)]
pub enum TextEditor<'a> {
    #[default]
    Default,
    Custom(text_editor::StyleFn<'a, Theme>),
}

impl iced_widget::text_editor::Catalog for Theme {
    type Class<'a> = TextEditor<'a>;

    fn default<'a>() -> Self::Class<'a> {
        TextEditor::default()
    }

    fn style(
        &self,
        class: &Self::Class<'_>,
        status: iced_widget::text_editor::Status,
    ) -> iced_widget::text_editor::Style {
        if let TextEditor::Custom(style) = class {
            return style(self, status);
        }

        let cosmic = self.cosmic();

        let selection = cosmic.accent.base.into();
        let value = cosmic.palette.neutral_9.into();
        let mut placeholder = cosmic.palette.neutral_9;
        placeholder.alpha = 0.7;
        let placeholder = placeholder.into();
        let icon = cosmic.background.on.into();

        match status {
            iced_widget::text_editor::Status::Active
            | iced_widget::text_editor::Status::Hovered
            | iced_widget::text_editor::Status::Disabled => iced_widget::text_editor::Style {
                background: iced::Color::from(cosmic.bg_color()).into(),
                border: Border {
                    radius: cosmic.corner_radii.radius_0.into(),
                    width: f32::from(cosmic.space_xxxs()),
                    color: iced::Color::from(cosmic.bg_divider()),
                },
                icon,
                placeholder,
                value,
                selection,
            },
            iced_widget::text_editor::Status::Focused => iced_widget::text_editor::Style {
                background: iced::Color::from(cosmic.bg_color()).into(),
                border: Border {
                    radius: cosmic.corner_radii.radius_0.into(),
                    width: f32::from(cosmic.space_xxxs()),
                    color: iced::Color::from(cosmic.accent.base),
                },
                icon,
                placeholder,
                value,
                selection,
            },
        }
    }
}
