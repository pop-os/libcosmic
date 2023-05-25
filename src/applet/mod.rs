use cosmic_panel_config::{CosmicPanelBackground, PanelAnchor, PanelSize};
use iced::{
    alignment::{Horizontal, Vertical},
    wayland::InitialSurface,
    widget::{self, Container},
    Color, Element, Length, Rectangle, Settings,
};
use iced_core::layout::Limits;
use iced_style::{button::StyleSheet, container::Appearance};
use iced_widget::runtime::command::platform_specific::wayland::{
    popup::{SctkPopupSettings, SctkPositioner},
    window::SctkWindowSettings,
};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};

use crate::{theme::Button, Renderer};

pub use cosmic_panel_config;

const APPLET_PADDING: u32 = 8;

#[must_use]
pub fn applet_button_theme() -> Button {
    Button::Custom {
        active: Box::new(|t| iced_style::button::Appearance {
            border_radius: 0.0,
            ..t.active(&Button::Text)
        }),
        hover: Box::new(|t| iced_style::button::Appearance {
            border_radius: 0.0,
            ..t.hovered(&Button::Text)
        }),
    }
}

#[derive(Debug, Clone)]
pub struct CosmicAppletHelper {
    pub size: Size,
    pub anchor: PanelAnchor,
    pub background: CosmicPanelBackground,
    pub output_name: String,
}

#[derive(Clone, Debug)]
pub enum Size {
    PanelSize(PanelSize),
    // (width, height)
    Hardcoded((u16, u16)),
}

impl Default for CosmicAppletHelper {
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
        }
    }
}

impl CosmicAppletHelper {
    #[must_use]
    pub fn suggested_size(&self) -> (u16, u16) {
        match &self.size {
            Size::PanelSize(size) => match size {
                PanelSize::XL => (64, 64),
                PanelSize::L => (36, 36),
                PanelSize::M => (24, 24),
                PanelSize::S => (16, 16),
                PanelSize::XS => (12, 12),
            },
            Size::Hardcoded((width, height)) => (*width, *height),
        }
    }

    // Set the default window size. Helper for application init with hardcoded size.
    pub fn window_size(&mut self, width: u16, height: u16) {
        self.size = Size::Hardcoded((width, height));
    }

    #[must_use]
    pub fn window_settings<F: Default>(&self) -> Settings<F> {
        self.window_settings_with_flags(F::default())
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn window_settings_with_flags<F>(&self, flags: F) -> Settings<F> {
        let (width, height) = self.suggested_size();
        let width = u32::from(width);
        let height = u32::from(height);
        Settings {
            initial_surface: InitialSurface::XdgWindow(SctkWindowSettings {
                size: (width + APPLET_PADDING * 2, height + APPLET_PADDING * 2),
                size_limits: Limits::NONE
                    .min_height(height as f32 + APPLET_PADDING as f32 * 2.0)
                    .max_height(height as f32 + APPLET_PADDING as f32 * 2.0)
                    .min_width(width as f32 + APPLET_PADDING as f32 * 2.0)
                    .max_width(width as f32 + APPLET_PADDING as f32 * 2.0),
                resizable: None,
                ..Default::default()
            }),
            ..crate::settings_with_flags(flags)
        }
    }

    #[must_use]
    pub fn icon_button<'a, Message: 'static>(
        &self,
        icon_name: &'a str,
    ) -> widget::Button<'a, Message, Renderer> {
        crate::widget::button(crate::theme::Button::Text)
            .icon(
                crate::theme::Svg::Symbolic,
                icon_name,
                self.suggested_size().0,
            )
            .padding(8)
    }

    // TODO popup container which tracks the size of itself and requests the popup to resize to match
    pub fn popup_container<'a, Message: 'static>(
        &self,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Container<'a, Message, Renderer> {
        let (vertical_align, horizontal_align) = match self.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };

        Container::<Message, Renderer>::new(Container::<Message, Renderer>::new(content).style(
            crate::theme::Container::custom(|theme| Appearance {
                text_color: Some(theme.cosmic().background.on.into()),
                background: Some(Color::from(theme.cosmic().background.base).into()),
                border_radius: 12.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }),
        ))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .align_x(horizontal_align)
        .align_y(vertical_align)
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn get_popup_settings(
        &self,
        parent: iced_core::window::Id,
        id: iced_core::window::Id,
        size: Option<(u32, u32)>,
        width_padding: Option<i32>,
        height_padding: Option<i32>,
    ) -> SctkPopupSettings {
        let (width, height) = self.suggested_size();
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
                    width: width_padding.unwrap_or(APPLET_PADDING as i32) * 2 + i32::from(width),
                    height: height_padding.unwrap_or(APPLET_PADDING as i32) * 2 + i32::from(height),
                },
                reactive: true,
                constraint_adjustment: 15, // slide_y, slide_x, flip_x, flip_y
                ..Default::default()
            },
            parent_size: None,
            grab: true,
        }
    }
}
