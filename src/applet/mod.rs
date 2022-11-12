use cosmic_panel_config::{PanelSize, PanelAnchor};
use iced::{widget::Button, Rectangle};
use iced_native::{command::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner}};
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};

use crate::{button, widget::icon};

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

pub fn get_popup_settings(parent: iced_native::window::Id, id: iced_native::window::Id, width_padding: Option<i32>, height_padding: Option<i32>) -> SctkPopupSettings {
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
    SctkPopupSettings { parent, id, positioner: SctkPositioner {
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
    }, parent_size: None, grab: true }
}