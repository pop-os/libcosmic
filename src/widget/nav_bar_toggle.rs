// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A button for toggling the navigation side panel.

use crate::{widget, Element};
use derive_setters::Setters;

#[derive(Setters)]
pub struct NavBarToggle<Message> {
    active: bool,
    #[setters(strip_option)]
    on_toggle: Option<Message>,
    class: crate::theme::Button,
    selected: bool,
}

#[must_use]
pub fn nav_bar_toggle<Message>() -> NavBarToggle<Message> {
    NavBarToggle {
        active: false,
        on_toggle: None,
        class: crate::theme::Button::NavToggle,
        selected: false,
    }
}

impl<Message: 'static + Clone> From<NavBarToggle<Message>> for Element<'_, Message> {
    fn from(nav_bar_toggle: NavBarToggle<Message>) -> Self {
        let icon = if nav_bar_toggle.active {
            widget::icon::from_svg_bytes(
                &include_bytes!("../../res/icons/navbar-open-symbolic.svg")[..],
            )
            .symbolic(true)
        } else {
            widget::icon::from_svg_bytes(
                &include_bytes!("../../res/icons/navbar-closed-symbolic.svg")[..],
            )
            .symbolic(true)
        };

        widget::button::icon(icon)
            .padding([8, 16])
            .on_press_maybe(nav_bar_toggle.on_toggle)
            .selected(nav_bar_toggle.selected)
            .class(nav_bar_toggle.class)
            .into()
    }
}
