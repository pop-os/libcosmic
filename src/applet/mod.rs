use cosmic_panel_config::{PanelAnchor, PanelSize};
use iced::{
    alignment::{Horizontal, Vertical},
    wayland::InitialSurface,
    widget::{self, Container},
    Color, Element, Length, Rectangle, Settings,
};
use iced_core::BorderRadius;
use iced_native::command::platform_specific::wayland::{
    popup::{SctkPopupSettings, SctkPositioner},
    window::SctkWindowSettings,
};
use iced_style::{button::StyleSheet, container::Appearance};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};

use crate::{theme::Button, Renderer};

pub use cosmic_panel_config;

const APPLET_PADDING: u32 = 8;

pub const APPLET_BUTTON_THEME: Button = Button::Custom {
    active: |t| iced_style::button::Appearance {
        border_radius: BorderRadius::from(0.0),
        ..t.active(&Button::Text)
    },
    hover: |t| iced_style::button::Appearance {
        border_radius: BorderRadius::from(0.0),
        ..t.hovered(&Button::Text)
    },
};

#[derive(Debug, Clone)]
pub struct CosmicAppletHelper {
    pub size: Size,
    pub anchor: PanelAnchor,
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
                    .and_then(|size| size.parse::<PanelSize>().ok())
                    .unwrap_or(PanelSize::S),
            ),
            anchor: std::env::var("COSMIC_PANEL_ANCHOR")
                .ok()
                .and_then(|size| size.parse::<PanelAnchor>().ok())
                .unwrap_or(PanelAnchor::Top),
        }
    }
}

impl CosmicAppletHelper {
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
    pub fn window_settings_with_flags<F>(&self, flags: F) -> Settings<F> {
        let (width, height) = self.suggested_size();
        let width = u32::from(width);
        let height = u32::from(height);
        Settings {
            initial_surface: InitialSurface::XdgWindow(SctkWindowSettings {
                iced_settings: iced_native::window::Settings {
                    size: (width + APPLET_PADDING * 2, height + APPLET_PADDING * 2),
                    min_size: Some((width + APPLET_PADDING * 2, height + APPLET_PADDING * 2)),
                    max_size: Some((width + APPLET_PADDING * 2, height + APPLET_PADDING * 2)),
                    ..Default::default()
                },
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
        let (valign, halign) = match self.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };

        Container::<Message, Renderer>::new(Container::<Message, Renderer>::new(content).style(
            crate::theme::Container::Custom(|theme| Appearance {
                text_color: Some(theme.cosmic().on.into()),
                background: Some(Color::from(theme.cosmic().background.base).into()),
                border_radius: 12.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }),
        ))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .align_x(halign)
        .align_y(valign)
    }

    #[must_use]
    pub fn get_popup_settings(
        &self,
        parent: iced_native::window::Id,
        id: iced_native::window::Id,
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
