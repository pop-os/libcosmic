// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod expander;
mod segmented_button;

use std::hash::Hash;
use std::hash::Hasher;

pub use self::segmented_button::SegmentedButton;

use cosmic_theme::Component;
use iced_core::BorderRadius;
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
use iced_style::svg;
use iced_style::text;
use iced_style::text_input;
use iced_style::toggler;

use iced_core::{Background, Color};

type CosmicColor = ::palette::rgb::Srgba;
type CosmicComponent = cosmic_theme::Component<CosmicColor>;
type CosmicTheme = cosmic_theme::Theme<CosmicColor>;
type CosmicThemeCss = cosmic_theme::Theme<cosmic_theme::util::CssColor>;

lazy_static::lazy_static! {
    pub static ref COSMIC_DARK: CosmicTheme = CosmicThemeCss::dark_default().into_srgba();
    pub static ref COSMIC_LIGHT: CosmicTheme = CosmicThemeCss::light_default().into_srgba();
    pub static ref TRANSPARENT_COMPONENT: Component<CosmicColor> = Component {
        base: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        hover: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        pressed: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        selected: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        selected_text: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        focus: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        on: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
        on_disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    #[must_use]
    pub fn cosmic(self) -> &'static CosmicTheme {
        match self {
            Self::Dark => &COSMIC_DARK,
            Self::Light => &COSMIC_LIGHT,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Application {
    Default,
    Custom(fn(Theme) -> application::Appearance),
}

impl Default for Application {
    fn default() -> Self {
        Self::Default
    }
}

impl application::StyleSheet for Theme {
    type Style = Application;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        let cosmic = self.cosmic();

        match style {
            Application::Default => application::Appearance {
                background_color: cosmic.bg_color().into(),
                text_color: cosmic.on.into(),
            },
            Application::Custom(f) => f(*self),
        }
    }
}

/*
 * TODO: Button
 */
#[derive(Clone, Copy)]
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
    Custom {
        active: fn(&Theme) -> button::Appearance,
        hover: fn(&Theme) -> button::Appearance,
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
    fn cosmic(&self, theme: &Theme) -> &'static CosmicComponent {
        let cosmic = theme.cosmic();
        match self {
            Button::Primary => &cosmic.accent,
            Button::Secondary => &cosmic.basic,
            Button::Positive => &cosmic.success,
            Button::Destructive => &cosmic.destructive,
            Button::Text => &cosmic.basic,
            Button::Link => &cosmic.accent,
            Button::LinkActive => &cosmic.basic,
            Button::Transparent => &TRANSPARENT_COMPONENT,
            Button::Deactivated => &cosmic.basic,
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
        let cosmic = self.cosmic();
        button::Appearance {
            border_radius: match style {
                Button::Link => BorderRadius::from(0.0),
                _ => BorderRadius::from(24.0),
            },
            background: match style {
                Button::Link | Button::Text => None,
                Button::LinkActive => Some(Background::Color(cosmic.divider.into())),
                _ => Some(Background::Color(component.base.into())),
            },
            text_color: match style {
                Button::Link => component.base.into(),
                Button::LinkActive => component.selected_text.into(),
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
        let cosmic = self.cosmic();

        button::Appearance {
            background: match style {
                Button::Link => None,
                Button::LinkActive => Some(Background::Color(cosmic.divider.into())),
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
        let cosmic = self.cosmic();
        button::Appearance {
            background: match style {
                Button::Link => None,
                Button::LinkActive => Some(Background::Color(cosmic.divider.into())),
                _ => Some(Background::Color(component.hover.into())),
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

        match style {
            Checkbox::Primary => checkbox_appearance(
                palette.accent.on,
                palette.basic.base,
                palette.accent.base,
                is_checked,
            ),
            Checkbox::Secondary => checkbox_appearance(
                palette.basic.on,
                palette.basic.base,
                palette.basic.base,
                is_checked,
            ),
            Checkbox::Success => checkbox_appearance(
                palette.success.on,
                palette.basic.base,
                palette.success.base,
                is_checked,
            ),
            Checkbox::Danger => checkbox_appearance(
                palette.destructive.on,
                palette.basic.base,
                palette.destructive.base,
                is_checked,
            ),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.cosmic();

        match style {
            Checkbox::Primary => checkbox_appearance(
                palette.accent.on,
                palette.basic.hover,
                palette.accent.hover,
                is_checked,
            ),
            Checkbox::Secondary => checkbox_appearance(
                palette.basic.on,
                palette.basic.hover,
                palette.basic.hover,
                is_checked,
            ),
            Checkbox::Success => checkbox_appearance(
                palette.success.on,
                palette.basic.hover,
                palette.success.hover,
                is_checked,
            ),
            Checkbox::Danger => checkbox_appearance(
                palette.destructive.on,
                palette.basic.hover,
                palette.destructive.hover,
                is_checked,
            ),
        }
    }
}

fn checkbox_appearance<T: Into<Color> + Clone>(
    checkmark_color: T,
    base: T,
    accent: T,
    is_checked: bool,
) -> checkbox::Appearance {
    checkbox::Appearance {
        background: Background::Color(if is_checked {
            accent.clone().into()
        } else {
            base.into()
        }),
        checkmark_color: checkmark_color.into(),
        border_radius: 4.0,
        border_width: if is_checked { 0.0 } else { 1.0 },
        border_color: accent.into(),
        text_color: None,
    }
}

#[derive(Clone, Copy)]
pub enum Expander {
    Default,
    Custom(fn(&Theme) -> expander::Appearance),
}

impl Default for Expander {
    fn default() -> Self {
        Self::Default
    }
}

impl From<fn(&Theme) -> expander::Appearance> for Expander {
    fn from(f: fn(&Theme) -> expander::Appearance) -> Self {
        Self::Custom(f)
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
#[derive(Clone, Copy)]
pub enum Container {
    Background,
    Primary,
    Secondary,
    Transparent,
    Custom(fn(&Theme) -> container::Appearance),
}

impl Default for Container {
    fn default() -> Self {
        Self::Transparent
    }
}

impl From<fn(&Theme) -> container::Appearance> for Container {
    fn from(_: fn(&Theme) -> container::Appearance) -> Self {
        Self::default()
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
                    text_color: None,
                    background: Some(iced::Background::Color(palette.background.base.into())),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Primary => {
                let palette = self.cosmic();

                container::Appearance {
                    text_color: None,
                    background: Some(iced::Background::Color(palette.primary.base.into())),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Secondary => {
                let palette = self.cosmic();

                container::Appearance {
                    text_color: None,
                    background: Some(iced::Background::Color(palette.secondary.base.into())),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
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
            rail_colors: (
                cosmic.accent.base.into(),
                //TODO: no way to set color before/after slider
                Color::TRANSPARENT,
            ),
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
        style.handle.border_color = match self {
            Theme::Dark => Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.1),
            Theme::Light => Color::from_rgba8(0, 0, 0, 0.1),
        };
        style
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        let mut style = self.hovered(style);
        style.handle.border_color = match self {
            Theme::Dark => Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.2),
            Theme::Light => Color::from_rgba8(0, 0, 0, 0.2),
        };
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
            text_color: cosmic.on.into(),
            background: Background::Color(cosmic.background.base.into()),
            border_width: 0.0,
            border_radius: 16.0,
            border_color: Color::TRANSPARENT,
            selected_text_color: cosmic.on.into(),
            selected_background: Background::Color(cosmic.basic.hover.into()),
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
            text_color: cosmic.on.into(),
            background: Color::TRANSPARENT.into(),
            placeholder_color: cosmic.on.into(),
            border_radius: 24.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            icon_size: 0.7,
        }
    }

    fn hovered(&self, style: &()) -> pick_list::Appearance {
        let cosmic = &self.cosmic().basic;

        pick_list::Appearance {
            background: Background::Color(cosmic.hover.into()),
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
                Color::from(theme.basic.base).into()
            },
            dot_color: theme.accent.on.into(),
            border_width: 1.0,
            border_color: theme.on.into(),
            text_color: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> radio::Appearance {
        let active = self.active(style, is_selected);
        let theme = self.cosmic();

        radio::Appearance {
            dot_color: theme.accent.on.into(),
            background: if is_selected {
                Color::from(theme.accent.hover).into()
            } else {
                // TODO: this seems to be defined weirdly in FIGMA
                Color::from(theme.basic.hover).into()
            },
            ..active
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
        //TODO: grab colors from palette
        match self {
            Theme::Dark => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x9f, 0xed, 0xed)
                } else {
                    Color::from_rgb8(0xb6, 0xb6, 0xb6)
                },
                ..self.active(style, is_active)
            },
            Theme::Light => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x00, 0x42, 0x62)
                } else {
                    Color::from_rgb8(0x54, 0x54, 0x54)
                },
                ..self.active(style, is_active)
            },
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
}

/*
 * TODO: Progress Bar
 */
#[derive(Clone, Copy)]
pub enum ProgressBar {
    Primary,
    Success,
    Danger,
    Custom(fn(&Theme) -> progress_bar::Appearance),
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::Primary
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ProgressBar;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        let theme = self.cosmic();

        match style {
            ProgressBar::Primary => progress_bar::Appearance {
                background: Color::from(theme.divider).into(),
                bar: Color::from(theme.accent.base).into(),
                border_radius: 2.0,
            },
            ProgressBar::Success => progress_bar::Appearance {
                background: Color::from(theme.divider).into(),
                bar: Color::from(theme.success.base).into(),
                border_radius: 2.0,
            },
            ProgressBar::Danger => progress_bar::Appearance {
                background: Color::from(theme.divider).into(),
                bar: Color::from(theme.destructive.base).into(),
                border_radius: 2.0,
            },
            ProgressBar::Custom(f) => f(self),
        }
    }
}

/*
 * TODO: Rule
 */
#[derive(Clone, Copy)]
pub enum Rule {
    Default,
    LightDivider,
    HeavyDivider,
    Custom(fn(&Theme) -> rule::Appearance),
}

impl Default for Rule {
    fn default() -> Self {
        Self::Default
    }
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        let palette = self.cosmic();

        match style {
            Rule::Default => rule::Appearance {
                color: palette.on.into(),
                width: 1,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
            },
            Rule::LightDivider => {
                let cosmic = &self.cosmic();
                rule::Appearance {
                    color: cosmic.divider.into(),
                    width: 1,
                    radius: 0.0,
                    fill_mode: rule::FillMode::Padded(10),
                }
            }
            Rule::HeavyDivider => {
                let cosmic = &self.cosmic();
                rule::Appearance {
                    color: cosmic.divider.into(),
                    width: 4,
                    radius: 4.0,
                    fill_mode: rule::FillMode::Full,
                }
            }
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
        let theme = self.cosmic();

        scrollable::Scrollbar {
            background: Some(Background::Color(theme.basic.base.into())),
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: theme.divider.into(),
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        let theme = self.cosmic();

        scrollable::Scrollbar {
            background: Some(Background::Color(theme.basic.base.into())),
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: theme.accent.base.into(),
                border_radius: 4.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum Svg {
    /// Apply a custom appearance filter
    Custom(fn(&Theme) -> svg::Appearance),
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

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        #[allow(clippy::match_same_arms)]
        match style {
            Svg::Default => svg::Appearance::default(),
            Svg::Custom(appearance) => appearance(self),
            Svg::Symbolic => svg::Appearance {
                color: Some(self.cosmic().on.into()),
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
    Custom(fn(&Theme) -> text::Appearance),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
    }
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Accent => text::Appearance {
                color: Some(self.cosmic().accent.base.into()),
            },
            Text::Default => text::Appearance::default(),
            Text::Color(c) => text::Appearance { color: Some(c) },
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

        match style {
            TextInput::Default => text_input::Appearance {
                background: Background::Color(palette.basic.base.into()),
                border_radius: 2.0,
                border_width: 1.0,
                border_color: palette.divider.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.cosmic();

        match style {
            TextInput::Default => text_input::Appearance {
                background: Background::Color(palette.basic.base.into()),
                border_radius: 2.0,
                border_width: 1.0,
                border_color: palette.on.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let palette = self.cosmic();

        match style {
            TextInput::Default => text_input::Appearance {
                background: Background::Color(palette.basic.base.into()),
                border_radius: 2.0,
                border_width: 1.0,
                border_color: palette.accent.base.into(),
            },
            TextInput::Search => text_input::Appearance {
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();

        palette.divider.into()
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();

        palette.on.into()
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        let palette = self.cosmic();

        palette.basic.selected_text.into()
    }
}
