use cosmic_panel_config::{PanelAnchor, PanelSize};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{self, container, Container},
    Color, Element, Length, Rectangle,
};
use iced_native::{command::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner}, widget::Button};
use iced_style::container::{Appearance};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};

use crate::{Renderer, Theme};

#[derive(Debug, Clone)]
pub struct CosmicAppletHelper {
    pub size: PanelSize,
    pub anchor: PanelAnchor,
}

impl Default for CosmicAppletHelper {
    fn default() -> Self {
        Self {
            size: std::env::var("COSMIC_PANEL_SIZE")
                .ok()
                .and_then(|size| size.parse::<PanelSize>().ok())
                .unwrap_or(PanelSize::S),
            anchor: std::env::var("COSMIC_PANEL_SIZE")
                .ok()
                .and_then(|size| size.parse::<PanelAnchor>().ok())
                .unwrap_or(PanelAnchor::Top),
        }
    }
}

impl CosmicAppletHelper {
    pub fn suggested_icon_size(&self) -> u16 {
        match self.size {
            PanelSize::XL => 64,
            PanelSize::L => 36,
            PanelSize::M => 24,
            PanelSize::S => 16,
            PanelSize::XS => 12,
        }
    }


    pub fn icon_button<'a, Message: 'static>(&self, icon_name: &'a str) -> widget::Button<'a, Message, Renderer> {
        crate::widget::button(crate::theme::Button::Text).icon(crate::theme::Svg::Symbolic, icon_name, self.suggested_icon_size())
    }
    
    // TODO popup container which tracks the size of itself and requests the popup to resize to match
    pub fn popup_container<Message: 'static>(
        &self,
        content: impl Into<Element<'static, Message, Renderer>>,
    ) -> Container<Message, Renderer>
    {
        let (valign, halign) = match self.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };
    
        Container::<Message, Renderer>::new(
            Container::<Message, Renderer>::new(content).style(crate::theme::Container::Custom(|theme| Appearance {
                text_color: Some(theme.cosmic().on_bg_color().into()),
                background: Some(theme.extended_palette().background.base.color.into()),
                border_radius: 12.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            })),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(halign)
        .align_y(valign)
    }

    pub fn get_popup_settings(
        &self,
        parent: iced_native::window::Id,
        id: iced_native::window::Id,
        size: (u32, u32),
        width_padding: Option<i32>,
        height_padding: Option<i32>,
    ) -> SctkPopupSettings {
        let pixels = self.suggested_icon_size();
        let (offset, anchor, gravity) = match self.anchor {
            PanelAnchor::Left => ((8, 0), Anchor::Right, Gravity::Right),
            PanelAnchor::Right => ((-8, 0), Anchor::Left, Gravity::Left),
            PanelAnchor::Top => ((0, 8), Anchor::Bottom, Gravity::Bottom),
            PanelAnchor::Bottom => ((0, -8), Anchor::Top, Gravity::Top),
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
                    width: width_padding.unwrap_or(16) * 2 + pixels as i32,
                    height: height_padding.unwrap_or(8) * 2 + pixels as i32,
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

