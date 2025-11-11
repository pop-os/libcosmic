// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A button for toggling the navigation side panel.

use crate::{Element, widget};
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
pub const fn nav_bar_toggle<Message>() -> NavBarToggle<Message> {
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
            "navbar-open-symbolic"
        } else {
            "navbar-closed-symbolic"
        };

        widget::button::icon(widget::icon::from_name(icon))
            .padding([8, 16])
            .on_press_maybe(nav_bar_toggle.on_toggle)
            .selected(nav_bar_toggle.selected)
            .class(nav_bar_toggle.class)
            .into()
    }
}
