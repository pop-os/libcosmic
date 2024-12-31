#[cfg(feature = "applet-token")]
pub mod token;

use crate::{
    app::{self, iced_settings, Core},
    cctk::sctk,
    iced::{
        self,
        alignment::{Horizontal, Vertical},
        widget::Container,
        window, Color, Length, Limits, Rectangle,
    },
    iced_widget,
    theme::{self, system_dark, system_light, Button, THEME},
    widget::{
        self,
        autosize::{autosize, Autosize},
        layer_container,
    },
    Application, Element, Renderer,
};
use cctk::sctk::shell::xdg::window::WindowConfigure;
pub use cosmic_panel_config;
use cosmic_panel_config::{CosmicPanelBackground, PanelAnchor, PanelSize};
use cosmic_theme::Theme;
use iced::Pixels;
use iced_core::{Padding, Shadow};
use iced_widget::runtime::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};
use std::{borrow::Cow, num::NonZeroU32, rc::Rc, sync::LazyLock};
use tracing::info;

use crate::app::cosmic;
static AUTOSIZE_ID: LazyLock<iced::id::Id> =
    LazyLock::new(|| iced::id::Id::new("cosmic-applet-autosize"));
static AUTOSIZE_MAIN_ID: LazyLock<iced::id::Id> =
    LazyLock::new(|| iced::id::Id::new("cosmic-applet-autosize-main"));

#[derive(Debug, Clone)]
pub struct Context {
    pub size: Size,
    pub anchor: PanelAnchor,
    pub background: CosmicPanelBackground,
    pub output_name: String,
    pub panel_type: PanelType,
    /// Includes the configured size of the window.
    /// This can be used by apples to handle overflow themselves.
    pub suggested_bounds: Option<iced::Size>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Size {
    // (width, height)
    Hardcoded((u16, u16)),
    PanelSize(PanelSize),
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
            suggested_bounds: None,
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
    pub fn suggested_window_size(&self) -> (NonZeroU32, NonZeroU32) {
        let suggested = self.suggested_size(true);
        let applet_padding = self.suggested_padding(true);
        let configured_width = self
            .suggested_bounds
            .as_ref()
            .and_then(|c| NonZeroU32::new(c.width as u32)) // TODO: should this be physical size instead of logical?
            .unwrap_or_else(|| {
                NonZeroU32::new(suggested.0 as u32 + applet_padding as u32 * 2).unwrap()
            });

        let configured_height = self
            .suggested_bounds
            .as_ref()
            .and_then(|c| NonZeroU32::new(c.height as u32))
            .unwrap_or_else(|| {
                NonZeroU32::new(suggested.1 as u32 + applet_padding as u32 * 2).unwrap()
            });
        info!("{configured_height:?}");
        (configured_width, configured_height)
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

    #[allow(clippy::cast_precision_loss)]
    pub fn window_settings(&self) -> crate::app::Settings {
        let (width, height) = self.suggested_size(true);
        let applet_padding = self.suggested_padding(true);
        let width = f32::from(width) + applet_padding as f32 * 2.;
        let height = f32::from(height) + applet_padding as f32 * 2.;
        let mut settings = crate::app::Settings::default()
            .size(iced_core::Size::new(width, height))
            .size_limits(
                Limits::NONE
                    .min_height(height as f32)
                    .min_width(width as f32),
            )
            .resizable(None)
            .default_text_size(14.0)
            .default_font(crate::font::default())
            .transparent(true);
        if let Some(theme) = self.theme() {
            settings = settings.theme(theme);
        }
        settings.exit_on_close = true;
        settings
    }

    #[must_use]
    pub fn is_horizontal(&self) -> bool {
        matches!(self.anchor, PanelAnchor::Top | PanelAnchor::Bottom)
    }

    pub fn icon_button_from_handle<'a, Message: 'static>(
        &self,
        icon: widget::icon::Handle,
    ) -> crate::widget::Button<'a, Message> {
        let suggested = self.suggested_size(icon.symbolic);
        let applet_padding = self.suggested_padding(icon.symbolic);

        let symbolic = icon.symbolic;

        crate::widget::button::custom(
            layer_container(
                widget::icon(icon)
                    .class(if symbolic {
                        theme::Svg::Custom(Rc::new(|theme| crate::iced_widget::svg::Style {
                            color: Some(theme.cosmic().background.on.into()),
                        }))
                    } else {
                        theme::Svg::default()
                    })
                    .width(Length::Fixed(suggested.0 as f32))
                    .height(Length::Fixed(suggested.1 as f32)),
            )
            .center(Length::Fill),
        )
        .width(Length::Fixed((suggested.0 + 2 * applet_padding) as f32))
        .height(Length::Fixed((suggested.1 + 2 * applet_padding) as f32))
        .class(Button::AppletIcon)
    }

    pub fn icon_button<'a, Message: 'static>(
        &self,
        icon_name: &'a str,
    ) -> crate::widget::Button<'a, Message> {
        let suggested_size = self.suggested_size(true);
        self.icon_button_from_handle(
            widget::icon::from_name(icon_name)
                .symbolic(true)
                .size(suggested_size.0)
                .into(),
        )
    }

    // TODO popup container which tracks the size of itself and requests the popup to resize to match
    pub fn popup_container<'a, Message: 'static>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> Autosize<'a, Message, crate::Theme, Renderer> {
        let (vertical_align, horizontal_align) = match self.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };

        autosize(
            Container::<Message, _, Renderer>::new(
                Container::<Message, _, Renderer>::new(content).style(|theme| {
                    let cosmic = theme.cosmic();
                    let corners = cosmic.corner_radii.clone();
                    iced_widget::container::Style {
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
                }),
            )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_x(horizontal_align)
            .align_y(vertical_align),
            AUTOSIZE_ID.clone(),
        )
        .limits(
            Limits::NONE
                .min_width(1.)
                .min_height(1.)
                .max_width(500.)
                .max_height(1000.),
        )
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
        let pixel_offset = 4;
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

    pub fn autosize_window<'a, Message: 'static>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> Autosize<'a, Message, crate::Theme, crate::Renderer> {
        let force_configured = matches!(&self.panel_type, &PanelType::Other(ref n) if n.is_empty());
        let w = autosize(content, AUTOSIZE_MAIN_ID.clone());
        let mut limits = Limits::NONE;
        let suggested_window_size = self.suggested_window_size();

        if let Some(width) = self
            .suggested_bounds
            .as_ref()
            .filter(|c| c.width as i32 > 0)
            .map(|c| c.width)
        {
            limits = limits.width(width as f32);
        }
        if let Some(height) = self
            .suggested_bounds
            .as_ref()
            .filter(|c| c.height as i32 > 0)
            .map(|c| c.height)
        {
            limits = limits.height(height as f32);
        }

        w.limits(limits)
    }

    #[must_use]
    pub fn theme(&self) -> Option<theme::Theme> {
        match self.background {
            CosmicPanelBackground::Dark => {
                let mut theme = system_dark();
                theme.theme_type.prefer_dark(Some(true));
                Some(theme)
            }
            CosmicPanelBackground::Light => {
                let mut theme = system_light();
                theme.theme_type.prefer_dark(Some(false));
                Some(theme)
            }
            _ => Some(theme::system_preference()),
        }
    }

    pub fn text<'a>(&self, msg: impl Into<Cow<'a, str>>) -> crate::widget::Text<'a, crate::Theme> {
        let msg = msg.into();
        let t = match self.size {
            Size::PanelSize(PanelSize::XL) => crate::widget::text::title2,
            Size::PanelSize(PanelSize::L) => crate::widget::text::title3,
            Size::PanelSize(PanelSize::M) => crate::widget::text::title4,
            Size::PanelSize(PanelSize::S) => crate::widget::text::body,
            Size::PanelSize(PanelSize::XS) => crate::widget::text::body,
            Size::Hardcoded(_) => crate::widget::text,
        };
        t(msg).font(crate::font::default())
    }
}

/// Launch the application with the given settings.
///
/// # Errors
///
/// Returns error on application failure.
pub fn run<App: Application>(flags: App::Flags) -> iced::Result {
    let helper = Context::default();

    let mut settings = helper.window_settings();
    settings.resizable = None;

    #[cfg(target_env = "gnu")]
    if let Some(threshold) = settings.default_mmap_threshold {
        crate::malloc::limit_mmap_threshold(threshold);
    }

    if let Some(icon_theme) = settings.default_icon_theme.clone() {
        crate::icon_theme::set_default(icon_theme);
    }

    THEME
        .lock()
        .unwrap()
        .set_theme(settings.theme.theme_type.clone());

    let (iced_settings, (mut core, flags), mut window_settings) =
        iced_settings::<App>(settings, flags);
    core.window.show_headerbar = false;
    core.window.sharp_corners = true;
    core.window.show_maximize = false;
    core.window.show_minimize = false;
    core.window.use_template = false;

    window_settings.decorations = false;
    window_settings.exit_on_close_request = true;
    window_settings.resizable = false;
    window_settings.resize_border = 0;

    // TODO make multi-window not mandatory

    let mut app = super::app::multi_window::multi_window::<_, _, _, _, App::Executor>(
        cosmic::Cosmic::title,
        cosmic::Cosmic::update,
        cosmic::Cosmic::view,
    );
    if core.main_window.is_none() {
        app = app.window(window_settings.clone());
        core.main_window = Some(iced_core::window::Id::RESERVED);
    }
    app.subscription(cosmic::Cosmic::subscription)
        .style(cosmic::Cosmic::style)
        .theme(cosmic::Cosmic::theme)
        .settings(iced_settings)
        .run_with(move || cosmic::Cosmic::<App>::init((core, flags)))
}

#[must_use]
pub fn style() -> iced_runtime::Appearance {
    let theme = crate::theme::THEME.lock().unwrap();
    iced_runtime::Appearance {
        background_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
        text_color: theme.cosmic().on_bg_color().into(),
        icon_color: theme.cosmic().on_bg_color().into(),
    }
}

pub fn menu_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> crate::widget::Button<'a, Message> {
    crate::widget::button::custom(content)
        .class(Button::AppletMenu)
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
    let guard = THEME.lock().unwrap();
    let cosmic = guard.cosmic();
    [cosmic.space_xxs(), cosmic.space_m()].into()
}
