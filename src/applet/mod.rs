use cosmic_panel_config::{PanelAnchor, PanelSize};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{self, Button},
    Color, Element, Length, Rectangle,
};
use iced_native::command::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner};
use iced_style::container::Appearance;
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};

use crate::{
    button,
    theme::{self, Container},
    widget::icon,
};

pub fn icon_button<'a, M: 'a, Renderer>() -> Button<'a, M, Renderer>
where
    Renderer::Theme: iced_native::svg::StyleSheet + iced_style::button::StyleSheet,
    Renderer: iced_native::Renderer + iced_native::svg::Renderer + 'a,
{
    let pixels = std::env::var("COSMIC_PANEL_SIZE")
        .ok()
        .and_then(|size| match size.parse::<PanelSize>() {
            Ok(PanelSize::XL) => Some(64),
            Ok(PanelSize::L) => Some(36),
            Ok(PanelSize::M) => Some(24),
            Ok(PanelSize::S) => Some(16),
            Ok(PanelSize::XS) => Some(12),
            Err(_) => Some(12),
        })
        .unwrap_or(16);
    button!(icon("input-gaming-symbolic", pixels))
}

pub fn get_popup_settings(
    parent: iced_native::window::Id,
    id: iced_native::window::Id,
    width_padding: Option<i32>,
    height_padding: Option<i32>,
) -> SctkPopupSettings {
    let anchor = std::env::var("COSMIC_PANEL_ANCHOR")
        .ok()
        .map(|size| match size.parse::<PanelAnchor>() {
            Ok(p) => p,
            Err(_) => PanelAnchor::Top,
        })
        .unwrap_or(PanelAnchor::Top);
    let pixels = std::env::var("COSMIC_PANEL_SIZE")
        .ok()
        .and_then(|size| match size.parse::<PanelSize>() {
            Ok(PanelSize::XL) => Some(64),
            Ok(PanelSize::L) => Some(36),
            Ok(PanelSize::M) => Some(24),
            Ok(PanelSize::S) => Some(16),
            Ok(PanelSize::XS) => Some(12),
            Err(_) => Some(12),
        })
        .unwrap_or(16);
    let (anchor, gravity) = match anchor {
        PanelAnchor::Left => (Anchor::Right, Gravity::Right),
        PanelAnchor::Right => (Anchor::Left, Gravity::Left),
        PanelAnchor::Top => (Anchor::Bottom, Gravity::Bottom),
        PanelAnchor::Bottom => (Anchor::Top, Gravity::Top),
    };
    SctkPopupSettings {
        parent,
        id,
        positioner: SctkPositioner {
            anchor,
            gravity,
            size: (200, 200),
            anchor_rect: Rectangle {
                x: 0,
                y: 0,
                width: width_padding.unwrap_or(16) * 2 + pixels as i32,
                height: height_padding.unwrap_or(8) * 2 + pixels as i32,
            },
            reactive: true,
            ..Default::default()
        },
        parent_size: None,
        grab: true,
    }
}

// TODO popup container which tracks the size of itself and requests the popup to resize to match
pub fn popup_container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> crate::widget::widget::Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    Message: 'a,
    <<Renderer as iced_native::Renderer>::Theme as iced_style::container::StyleSheet>::Style:
        From<theme::Container>,
    Renderer::Theme: widget::container::StyleSheet,
{
    let anchor = std::env::var("COSMIC_PANEL_ANCHOR")
        .ok()
        .map(|size| match size.parse::<PanelAnchor>() {
            Ok(p) => p,
            Err(_) => PanelAnchor::Top,
        })
        .unwrap_or(PanelAnchor::Top);
    let (valign, halign) = match anchor {
        PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
        PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
        PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
        PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
    };
    crate::widget::widget::container(crate::widget::widget::container(content).style(
        Container::Custom(|theme| Appearance {
            text_color: Some(theme.cosmic().on_bg_color().into()),
            background: Some(theme.extended_palette().background.base.color.into()),
            border_radius: 12.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(halign)
    .align_y(valign)
}

