// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A button for toggling the navigation side panel.

use crate::{theme, Element};
use apply::Apply;
use derive_setters::Setters;
use iced::Length;

use super::IconSource;

#[derive(Setters)]
pub struct NavBarToggle<Message> {
    active: bool,
    #[setters(strip_option)]
    on_toggle: Option<Message>,
}

#[must_use]
pub fn nav_bar_toggle<Message>() -> NavBarToggle<Message> {
    NavBarToggle {
        active: false,
        on_toggle: None,
    }
}

impl<'a, Message: 'static + Clone> From<NavBarToggle<Message>> for Element<'a, Message> {
    fn from(nav_bar_toggle: NavBarToggle<Message>) -> Self {
        let mut widget = super::icon(
            if nav_bar_toggle.active {
                IconSource::svg_from_memory(&include_bytes!("../../res/sidebar-active.svg")[..])
            } else {
                IconSource::from("open-menu-symbolic")
            },
            16,
        )
        .style(theme::Svg::SymbolicActive)
        .apply(iced::widget::container)
        .apply(iced::widget::button)
        .padding([8, 16, 8, 16])
        .style(theme::Button::Text);

        if let Some(message) = nav_bar_toggle.on_toggle {
            widget = widget.on_press(message);
        }

        widget
            .apply(iced::widget::container)
            .center_y()
            .height(Length::Fill)
            .into()
    }
}
