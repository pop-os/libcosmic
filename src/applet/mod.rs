#[cfg(feature = "applet-token")]
pub mod token;

use crate::{
    app::Core,
    cctk::sctk,
    iced::{
        self,
        alignment::{Horizontal, Vertical},
        widget::Container,
        window, Color, Length, Limits, Rectangle,
    },
    iced_style, iced_widget,
    theme::{self, Button, THEME},
    widget, Application, Element, Renderer,
};
pub use cosmic_panel_config;
use cosmic_panel_config::{CosmicPanelBackground, PanelAnchor, PanelSize};
use iced_core::{Padding, Shadow};
use iced_style::container::Appearance;
use iced_widget::runtime::command::platform_specific::wayland::popup::{
    SctkPopupSettings, SctkPositioner,
};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};
use std::rc::Rc;

use crate::app::cosmic;

#[derive(Debug, Clone)]
pub struct Context {
    pub size: Size,
    pub anchor: PanelAnchor,
    pub background: CosmicPanelBackground,
    pub output_name: String,
    pub panel_type: PanelType,
}

#[derive(Clone, Debug)]
pub enum Size {
    PanelSize(PanelSize),
    // (width, height)
    Hardcoded((u16, u16)),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PanelType {
    Panel,
    Dock,
    Other(String),
}

impl ToString for PanelType {
    fn to_string(&self) -> String {
        match self {
            Self::Panel => "Panel".to_string(),
            Self::Dock => "Dock".to_string(),
            Self::Other(other) => other.clone(),
        }
    }
}

impl From<String> for PanelType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Panel" => PanelType::Panel,
            "Dock" => PanelType::Dock,
            other => PanelType::Other(other.to_string()),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            size: Size::PanelSize(
                std::env::var("COSMIC_PANEL_SIZE")
                    .ok()
                    .and_then(|size| ron::from_str(size.as_str()).ok())
                    .unwrap_or(PanelSize::S),
            ),
            anchor: std::env::var("COSMIC_PANEL_ANCHOR")
                .ok()
                .and_then(|size| ron::from_str(size.as_str()).ok())
                .unwrap_or(PanelAnchor::Top),
            background: std::env::var("COSMIC_PANEL_BACKGROUND")
                .ok()
                .and_then(|size| ron::from_str(size.as_str()).ok())
                .unwrap_or(CosmicPanelBackground::ThemeDefault),
            output_name: std::env::var("COSMIC_PANEL_OUTPUT").unwrap_or_default(),
            panel_type: PanelType::from(std::env::var("COSMIC_PANEL_NAME").unwrap_or_default()),
        }
    }
}

impl Context {
    #[must_use]
    pub fn suggested_size(&self, is_symbolic: bool) -> (u16, u16) {
        match &self.size {
            Size::PanelSize(ref size) => {
                let s = size.get_applet_icon_size(is_symbolic) as u16;
                (s, s)
            }
            Size::Hardcoded((width, height)) => (*width, *height),
        }
    }

    #[must_use]
    pub fn suggested_padding(&self, is_symbolic: bool) -> u16 {
        match &self.size {
            Size::PanelSize(ref size) => size.get_applet_padding(is_symbolic),
            Size::Hardcoded(_) => 8,
        }
    }

    // Set the default window size. Helper for application init with hardcoded size.
    pub fn window_size(&mut self, width: u16, height: u16) {
        self.size = Size::Hardcoded((width, height));
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn window_settings(&self) -> crate::app::Settings {
        let (width, height) = self.suggested_size(true);
        let width = f32::from(width);
        let height = f32::from(height);
        let applet_padding = self.suggested_padding(true);
        let mut settings = crate::app::Settings::default()
            .size(iced_core::Size::new(
                width + applet_padding as f32 * 2.,
                height + applet_padding as f32 * 2.,
            ))
            .size_limits(
                Limits::NONE
                    .min_height(height as f32 + applet_padding as f32 * 2.0)
                    .max_height(height as f32 + applet_padding as f32 * 2.0)
                    .min_width(width as f32 + applet_padding as f32 * 2.0)
                    .max_width(width as f32 + applet_padding as f32 * 2.0),
            )
            .resizable(None)
            .default_text_size(18.0)
            .default_font(crate::font::FONT)
            .transparent(true);
        if let Some(theme) = self.theme() {
            settings = settings.theme(theme);
        }
        settings
    }

    #[must_use]
    pub fn icon_button_from_handle<'a, Message: 'static>(
        &self,
        icon: widget::icon::Handle,
    ) -> crate::widget::Button<'a, Message, crate::Theme, Renderer> {
        let symbolic = icon.symbolic;
        let suggested = self.suggested_size(symbolic);
        let applet_padding = self.suggested_padding(symbolic);
        crate::widget::button(
            widget::icon(icon)
                .style(if symbolic {
                    theme::Svg::Custom(Rc::new(|theme| crate::iced_style::svg::Appearance {
                        color: Some(theme.cosmic().background.on.into()),
                    }))
                } else {
                    theme::Svg::default()
                })
                .width(Length::Fixed(suggested.0 as f32))
                .height(Length::Fixed(suggested.1 as f32)),
        )
        .padding(applet_padding)
        .style(Button::AppletIcon)
    }

    #[must_use]
    pub fn icon_button<'a, Message: 'static>(
        &self,
        icon_name: &'a str,
    ) -> crate::widget::Button<'a, Message, crate::Theme, Renderer> {
        self.icon_button_from_handle(
            widget::icon::from_name(icon_name)
                .symbolic(true)
                .size(self.suggested_size(true).0)
                .into(),
        )
    }

    // TODO popup container which tracks the size of itself and requests the popup to resize to match
    pub fn popup_container<'a, Message: 'static>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> Container<'a, Message, crate::Theme, Renderer> {
        let (vertical_align, horizontal_align) = match self.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };

        Container::<Message, _, Renderer>::new(
            Container::<Message, _, Renderer>::new(content).style(theme::Container::custom(
                |theme| {
                    let cosmic = theme.cosmic();
                    let corners = cosmic.corner_radii.clone();
                    Appearance {
                        text_color: Some(cosmic.background.on.into()),
                        background: Some(Color::from(cosmic.background.base).into()),
                        border: iced::Border {
                            radius: corners.radius_m.into(),
                            width: 1.0,
                            color: cosmic.background.divider.into(),
                        },
                        shadow: Shadow::default(),
                        icon_color: Some(cosmic.background.on.into()),
                    }
                },
            )),
        )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .align_x(horizontal_align)
        .align_y(vertical_align)
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn get_popup_settings(
        &self,
        parent: window::Id,
        id: window::Id,
        size: Option<(u32, u32)>,
        width_padding: Option<i32>,
        height_padding: Option<i32>,
    ) -> SctkPopupSettings {
        let (width, height) = self.suggested_size(true);
        let applet_padding = self.suggested_padding(true);
        let pixel_offset = 8;
        let (offset, anchor, gravity) = match self.anchor {
            PanelAnchor::Left => ((pixel_offset, 0), Anchor::Right, Gravity::Right),
            PanelAnchor::Right => ((-pixel_offset, 0), Anchor::Left, Gravity::Left),
            PanelAnchor::Top => ((0, pixel_offset), Anchor::Bottom, Gravity::Bottom),
            PanelAnchor::Bottom => ((0, -pixel_offset), Anchor::Top, Gravity::Top),
        };
        SctkPopupSettings {
            parent,
            id,
            positioner: SctkPositioner {
                anchor,
                gravity,
                offset,
                size,
                anchor_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: width_padding.unwrap_or(applet_padding as i32) * 2 + i32::from(width),
                    height: height_padding.unwrap_or(applet_padding as i32) * 2 + i32::from(height),
                },
                reactive: true,
                constraint_adjustment: 15, // slide_y, slide_x, flip_x, flip_y
                ..Default::default()
            },
            parent_size: None,
            grab: true,
        }
    }

    #[must_use]
    pub fn theme(&self) -> Option<theme::Theme> {
        match self.background {
            CosmicPanelBackground::Dark => Some(theme::Theme::dark()),
            CosmicPanelBackground::Light => Some(theme::Theme::light()),
            _ => Some(theme::system_preference()),
        }
    }
}

/// Launch the application with the given settings.
///
/// # Errors
///
/// Returns error on application failure.
pub fn run<App: Application>(autosize: bool, flags: App::Flags) -> iced::Result {
    let helper = Context::default();
    let mut settings = helper.window_settings();
    settings.autosize = autosize;
    if autosize {
        settings.size_limits = Limits::NONE;
    }

    if let Some(icon_theme) = settings.default_icon_theme {
        crate::icon_theme::set_default(icon_theme);
    }

    let (width, height) = (settings.size.width as u32, settings.size.height as u32);

    let mut core = Core::default();
    core.window.show_window_menu = false;
    core.window.show_headerbar = false;
    core.window.sharp_corners = true;
    core.window.show_maximize = false;
    core.window.show_minimize = false;
    core.window.use_template = false;

    core.debug = settings.debug;
    core.set_scale_factor(settings.scale_factor);
    core.set_window_width(width);
    core.set_window_height(height);

    THEME.with(move |t| {
        let mut cosmic_theme = t.borrow_mut();
        cosmic_theme.set_theme(settings.theme.theme_type);
    });

    let mut iced = iced::Settings::with_flags((core, flags));

    iced.antialiasing = settings.antialiasing;
    iced.default_font = settings.default_font;
    iced.default_text_size = settings.default_text_size.into();
    iced.id = Some(App::APP_ID.to_owned());

    {
        use iced::wayland::actions::window::SctkWindowSettings;
        use iced_sctk::settings::InitialSurface;
        iced.initial_surface = InitialSurface::XdgWindow(SctkWindowSettings {
            app_id: Some(App::APP_ID.to_owned()),
            autosize: settings.autosize,
            client_decorations: settings.client_decorations,
            resizable: settings.resizable,
            size: (width, height),
            size_limits: settings.size_limits,
            title: None,
            transparent: settings.transparent,
            ..SctkWindowSettings::default()
        });
    }

    <cosmic::Cosmic<App> as iced::Application>::run(iced)
}

#[must_use]
pub fn style() -> <crate::Theme as iced_style::application::StyleSheet>::Style {
    <crate::Theme as iced_style::application::StyleSheet>::Style::Custom(Box::new(|theme| {
        iced_style::application::Appearance {
            background_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            text_color: theme.cosmic().on_bg_color().into(),
            icon_color: theme.cosmic().on_bg_color().into(),
        }
    }))
}

pub fn menu_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> crate::widget::Button<'a, Message, crate::Theme, crate::Renderer> {
    crate::widget::Button::new(content)
        .style(Button::AppletMenu)
        .padding(menu_control_padding())
        .width(Length::Fill)
}

pub fn padded_control<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> crate::widget::container::Container<'a, Message, crate::Theme, crate::Renderer> {
    crate::widget::container(content)
        .padding(menu_control_padding())
        .width(Length::Fill)
}

pub fn menu_control_padding() -> Padding {
    THEME
        .with(|t| {
            let t = t.borrow();
            let cosmic = t.cosmic();
            [cosmic.space_xxs(), cosmic.space_m()]
        })
        .into()
}
