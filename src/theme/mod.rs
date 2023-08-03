// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod expander;
mod segmented_button;

use std::f32::consts::PI;
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;
use std::sync::Arc;

pub use self::segmented_button::SegmentedButton;

use cosmic_config::config_subscription;
use cosmic_config::CosmicConfigEntry;
use cosmic_theme::util::CssColor;
use cosmic_theme::Component;
use cosmic_theme::LayeredTheme;
use iced_core::gradient::Linear;
use iced_core::BorderRadius;
use iced_core::Radians;
use iced_futures::Subscription;
use iced_style::application;
use iced_style::button;
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

use iced_core::{Background, Color};
use palette::Srgba;

pub type CosmicColor = ::palette::rgb::Srgba;
pub type CosmicComponent = cosmic_theme::Component<CosmicColor>;
pub type CosmicTheme = cosmic_theme::Theme<CosmicColor>;

lazy_static::lazy_static! {
    pub static ref COSMIC_DARK: CosmicTheme = CosmicTheme::dark_default();
    pub static ref COSMIC_HC_DARK: CosmicTheme = CosmicTheme::high_contrast_dark_default();
    pub static ref COSMIC_LIGHT: CosmicTheme = CosmicTheme::light_default();
    pub static ref COSMIC_HC_LIGHT: CosmicTheme = CosmicTheme::high_contrast_light_default();
    pub static ref TRANSPARENT_COMPONENT: Component<CosmicColor> = Component {
        base: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        hover: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        pressed: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        selected: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        selected_text: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        focus: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        on: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        on_disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        divider: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    };
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ThemeType {
    #[default]
    Dark,
    Light,
    HighContrastDark,
    HighContrastLight,
    Custom(Arc<CosmicTheme>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Theme {
    pub theme_type: ThemeType,
    pub layer: cosmic_theme::Layer,
}

impl Theme {
    #[must_use]
    pub fn cosmic(&self) -> &cosmic_theme::Theme<Srgba> {
        match self.theme_type {
            ThemeType::Dark => &COSMIC_DARK,
            ThemeType::Light => &COSMIC_LIGHT,
            ThemeType::HighContrastDark => &COSMIC_HC_DARK,
            ThemeType::HighContrastLight => &COSMIC_HC_LIGHT,
            ThemeType::Custom(ref t) => t.as_ref(),
        }
    }

    #[must_use]
    pub fn dark() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light() -> Self {
        Self {
            theme_type: ThemeType::Light,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn dark_hc() -> Self {
        Self {
            theme_type: ThemeType::HighContrastDark,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn light_hc() -> Self {
        Self {
            theme_type: ThemeType::HighContrastLight,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn custom(theme: Arc<CosmicTheme>) -> Self {
        Self {
            theme_type: ThemeType::Custom(theme),
            ..Default::default()
        }
    }

    /// get current container
    /// can be used in a component that is intended to be a child of a `CosmicContainer`
    #[must_use]
    pub fn current_container(&self) -> &cosmic_theme::Container<Srgba> {
        match self.layer {
            cosmic_theme::Layer::Background => &self.cosmic().background,
            cosmic_theme::Layer::Primary => &self.cosmic().primary,
            cosmic_theme::Layer::Secondary => &self.cosmic().secondary,
        }
    }
}

impl LayeredTheme for Theme {
    fn set_layer(&mut self, layer: cosmic_theme::Layer) {
        self.layer = layer;
    }
}

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
                background_color: cosmic.bg_color().into(),
                text_color: cosmic.on_bg_color().into(),
            },
            Application::Custom(f) => f(self),
        }
    }
}

/*
 * TODO: Button
 */
pub enum Button {
    Deactivated,
    Destructive,
    Positive,
    Primary,
    Secondary,
    Text,
    Link,
    LinkActive,
    Transparent,
    Card,
    Custom {
        active: Box<dyn Fn(&Theme) -> button::Appearance>,
        hover: Box<dyn Fn(&Theme) -> button::Appearance>,
    },
}

impl Default for Button {
    fn default() -> Self {
        Self::Primary
    }
}

impl Button {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::match_same_arms)]
    fn cosmic<'a>(&'a self, theme: &'a Theme) -> &CosmicComponent {
        let cosmic = theme.cosmic();
        match self {
            Button::Primary => &cosmic.accent,
            Button::Secondary => &theme.current_container().component,
            Button::Positive => &cosmic.success,
            Button::Destructive => &cosmic.destructive,
            Button::Text => &theme.current_container().component,
            Button::Link => &cosmic.accent,
            Button::LinkActive => &cosmic.accent,
            Button::Transparent => &TRANSPARENT_COMPONENT,
            Button::Deactivated => &theme.current_container().component,
            Button::Card => &theme.current_container().component,
            Button::Custom { .. } => &TRANSPARENT_COMPONENT,
        }
    }
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        if let Button::Custom { active, .. } = style {
            return active(self);
        }

        let component = style.cosmic(self);
        button::Appearance {
            border_radius: match style {
                Button::Link => 0.0.into(),
                Button::Card => 8.0.into(),
                _ => 24.0.into(),
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
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        if let Button::Custom { hover, .. } = style {
            return hover(self);
        }

        let active = self.active(style);
        let component = style.cosmic(self);

        button::Appearance {
            background: match style {
                Button::Link => None,
                Button::LinkActive => Some(Background::Color(component.divider.into())),
                _ => Some(Background::Color(component.hover.into())),
            },
            ..active
        }
    }

    fn focused(&self, style: &Self::Style) -> button::Appearance {
        if let Button::Custom { hover, .. } = style {
            return hover(self);
        }

        let active = self.active(style);
        let component = style.cosmic(self);
        button::Appearance {
            background: match style {
                Button::Link => None,
                Button::LinkActive => Some(Background::Color(component.divider.into())),
                _ => Some(Background::Color(component.hover.into())),
            },
            ..active
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        if matches!(style, Button::Card) {
            return active;
        }

        button::Appearance {
            shadow_offset: iced_core::Vector::default(),
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
        let palette = self.cosmic();
        let neutral_7 = palette.palette.neutral_10;

        match style {
            Checkbox::Primary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.accent.base.into()
                } else {
                    palette.background.base.into()
                }),
                icon_color: palette.accent.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.accent.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
            Checkbox::Secondary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.background.component.base.into()
                } else {
                    palette.background.base.into()
                }),
                icon_color: palette.background.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: neutral_7.into(),
                text_color: None,
            },
            Checkbox::Success => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.success.base.into()
                } else {
                    palette.background.base.into()
                }),
                icon_color: palette.success.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.success.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
            Checkbox::Danger => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.destructive.base.into()
                } else {
                    palette.background.base.into()
                }),
                icon_color: palette.destructive.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.destructive.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.cosmic();
        let mut neutral_10 = palette.palette.neutral_10;
        let neutral_7 = palette.palette.neutral_10;

        neutral_10.alpha = 0.1;
        match style {
            Checkbox::Primary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.accent.base.into()
                } else {
                    neutral_10.into()
                }),
                icon_color: palette.accent.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.accent.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
            Checkbox::Secondary => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    self.current_container().base.into()
                } else {
                    neutral_10.into()
                }),
                icon_color: self.current_container().on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    self.current_container().base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
            Checkbox::Success => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.success.base.into()
                } else {
                    neutral_10.into()
                }),
                icon_color: palette.success.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.success.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
            Checkbox::Danger => checkbox::Appearance {
                background: Background::Color(if is_checked {
                    palette.destructive.base.into()
                } else {
                    neutral_10.into()
                }),
                icon_color: palette.destructive.on.into(),
                border_radius: 4.0.into(),
                border_width: if is_checked { 0.0 } else { 1.0 },
                border_color: if is_checked {
                    palette.destructive.base
                } else {
                    neutral_7
                }
                .into(),
                text_color: None,
            },
        }
    }
}

#[derive(Default)]
pub enum Expander {
    #[default]
    Default,
    Custom(Box<dyn Fn(&Theme) -> expander::Appearance>),
}

impl Expander {
    pub fn custom<F: Fn(&Theme) -> expander::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl expander::StyleSheet for Theme {
    type Style = Expander;

    fn appearance(&self, style: Self::Style) -> expander::Appearance {
        match style {
            Expander::Default => expander::Appearance::default(),
            Expander::Custom(f) => f(self),
        }
    }
}

/*
 * TODO: Container
 */
#[derive(Default)]
pub enum Container {
    Background,
    Primary,
    Secondary,
    #[default]
    Transparent,
    HeaderBar,
    Custom(Box<dyn Fn(&Theme) -> container::Appearance>),
    Card,
}

impl Container {
    pub fn custom<F: Fn(&Theme) -> container::Appearance + 'static>(f: F) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => container::Appearance::default(),
            Container::Custom(f) => f(self),
            Container::Background => {
                let palette = self.cosmic();

                container::Appearance {
                    text_color: Some(Color::from(palette.background.on)),
                    background: Some(iced::Background::Color(palette.background.base.into())),
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::HeaderBar => {
                let palette = self.cosmic();
                let mut header_top = palette.background.base;
                let header_bottom = palette.background.base;
                header_top.alpha = 0.8;

                container::Appearance {
                    text_color: Some(Color::from(palette.background.on)),
                    background: Some(iced::Background::Gradient(iced_core::Gradient::Linear(
                        Linear::new(Radians(3.0 * PI / 2.0))
                            .add_stop(0.0, header_top.into())
                            .add_stop(1.0, header_bottom.into()),
                    ))),
                    border_radius: BorderRadius::from([16.0, 16.0, 0.0, 0.0]),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Primary => {
                let palette = self.cosmic();

                container::Appearance {
                    text_color: Some(Color::from(palette.primary.on)),
                    background: Some(iced::Background::Color(palette.primary.base.into())),
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Secondary => {
                let palette = self.cosmic();

                container::Appearance {
                    text_color: Some(Color::from(palette.secondary.on)),
                    background: Some(iced::Background::Color(palette.secondary.base.into())),
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Card => {
                let palette = self.cosmic();

                match self.layer {
                    cosmic_theme::Layer::Background => container::Appearance {
                        text_color: Some(Color::from(palette.background.component.on)),
                        background: Some(iced::Background::Color(
                            palette.background.component.base.into(),
                        )),
                        border_radius: 8.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    cosmic_theme::Layer::Primary => container::Appearance {
                        text_color: Some(Color::from(palette.primary.component.on)),
                        background: Some(iced::Background::Color(
                            palette.primary.component.base.into(),
                        )),
                        border_radius: 8.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    cosmic_theme::Layer::Secondary => container::Appearance {
                        text_color: Some(Color::from(palette.secondary.component.on)),
                        background: Some(iced::Background::Color(
                            palette.secondary.component.base.into(),
                        )),
                        border_radius: 8.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }
        }
    }
}

/*
 * Slider
 */
impl slider::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        let cosmic = self.cosmic();

        //TODO: no way to set rail thickness
        slider::Appearance {
            rail: Rail {
                colors: (
                    cosmic.accent.base.into(),
                    //TODO: no way to set color before/after slider
                    Color::TRANSPARENT,
                ),
                width: 4.0,
                border_radius: 2.0.into(),
            },

            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 10.0 },
                color: cosmic.accent.base.into(),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        let mut style = self.active(style);
        style.handle.shape = slider::HandleShape::Circle { radius: 16.0 };
        style.handle.border_width = 6.0;
        let mut border_color = self.cosmic().palette.neutral_10;
        border_color.alpha = 0.1;
        style.handle.border_color = border_color.into();
        style
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        let mut style = self.hovered(style);
        let mut border_color = self.cosmic().palette.neutral_10;
        border_color.alpha = 0.2;
        style.handle.border_color = border_color.into();

        style
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
            border_width: 0.0,
            border_radius: 16.0.into(),
            border_color: Color::TRANSPARENT,
            selected_text_color: cosmic.on_bg_color().into(),
            // TODO doesn't seem to be specified
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
            border_radius: 24.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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

        toggler::Appearance {
            background: if is_active {
                theme.accent.base.into()
            } else {
                //TODO: Grab neutral from palette
                theme.palette.neutral_5.into()
            },
            background_border: None,
            //TODO: Grab neutral from palette
            foreground: theme.palette.neutral_2.into(),
            foreground_border: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let cosmic = self.cosmic();
        //TODO: grab colors from palette
        let mut neutral_10 = cosmic.palette.neutral_10;
        neutral_10.alpha = 0.1;
        toggler::Appearance {
            background: if is_active {
                cosmic.accent.hover
            } else {
                neutral_10
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
            border_width: 2.0,
            border_color: theme.bg_divider().into(),
            border_radius: 0.0.into(),
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
                border_radius: 2.0.into(),
            },
            ProgressBar::Success => progress_bar::Appearance {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.success.base).into(),
                border_radius: 2.0.into(),
            },
            ProgressBar::Danger => progress_bar::Appearance {
                background: Color::from(theme.background.divider).into(),
                bar: Color::from(theme.destructive.base).into(),
                border_radius: 2.0.into(),
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

/*
 * TODO: Scrollable
 */
impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: Some(Background::Color(
                self.current_container().component.base.into(),
            )),
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: self.current_container().component.divider.into(),
                border_radius: 4.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        _is_mouse_over_scrollbar: bool,
    ) -> scrollable::Scrollbar {
        let theme = self.cosmic();

        scrollable::Scrollbar {
            background: Some(Background::Color(
                self.current_container().component.hover.into(),
            )),
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: theme.accent.base.into(),
                border_radius: 4.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

#[derive(Clone, Default)]
pub enum Svg {
    /// Apply a custom appearance filter
    Custom(Rc<dyn Fn(&Theme) -> svg::Appearance>),
    /// No filtering is applied
    #[default]
    Default,
    /// Icon fill color will match text color
    Symbolic,
    /// Icon fill color will match accent color
    SymbolicActive,
    /// Icon fill color will match on primary color
    SymbolicPrimary,
    /// Icon fill color will use accent color
    SymbolicLink,
}

impl Hash for Svg {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let id = match self {
            Svg::Custom(_) => 0,
            Svg::Default => 1,
            Svg::Symbolic => 2,
            Svg::SymbolicActive => 3,
            Svg::SymbolicPrimary => 4,
            Svg::SymbolicLink => 5,
        };

        id.hash(state);
    }
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
            Svg::Symbolic => svg::Appearance {
                color: Some(self.current_container().on.into()),
            },
            Svg::SymbolicActive => svg::Appearance {
                color: Some(self.cosmic().accent.base.into()),
            },
            Svg::SymbolicPrimary => svg::Appearance {
                color: Some(self.cosmic().accent.on.into()),
            },
            Svg::SymbolicLink => svg::Appearance {
                color: Some(self.cosmic().accent.base.into()),
            },
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
    Custom(fn(&Theme) -> iced_widget::text::Appearance),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
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
                border_radius: 8.0.into(),
                border_width: 1.0,
                border_color: self.current_container().component.divider.into(),
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border_radius: 24.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                border_radius: 8.0.into(),
                border_width: 1.0,
                border_color: palette.accent.base.into(),
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border_radius: 24.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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
                border_radius: 8.0.into(),
                border_width: 1.0,
                border_color: palette.accent.base.into(),
                icon_color: self.current_container().on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Color::from(bg).into(),
                border_radius: 24.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
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

#[must_use]
pub fn theme() -> Theme {
    let Ok(helper) = crate::cosmic_config::Config::new(
        crate::cosmic_theme::NAME,
        crate::cosmic_theme::Theme::<CssColor>::version(),
    ) else {
        return crate::theme::Theme::dark();
    };
    let t = crate::cosmic_theme::Theme::get_entry(&helper).unwrap_or_else(|(errors, theme)| {
        for err in errors {
            tracing::error!("{:?}", err);
        }
        theme
    });
    crate::theme::Theme::custom(Arc::new(t))
}

pub fn subscription(id: u64) -> Subscription<crate::theme::Theme> {
    config_subscription::<u64, crate::cosmic_theme::Theme<Srgba>>(
        id,
        crate::cosmic_theme::NAME.into(),
        crate::cosmic_theme::Theme::<Srgba>::version(),
    )
    .map(|(_, res)| {
        let theme = res.unwrap_or_else(|(errors, theme)| {
            for err in errors {
                tracing::error!("{:?}", err);
            }
            theme
        });
        crate::theme::Theme::custom(Arc::new(theme))
    })
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
