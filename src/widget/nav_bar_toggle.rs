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
    nav_bar_active: bool,
    #[setters(strip_option)]
    on_nav_bar_toggled: Option<Message>,
}

#[must_use]
pub fn nav_bar_toggle<Message>() -> NavBarToggle<Message> {
    NavBarToggle {
        nav_bar_active: false,
        on_nav_bar_toggled: None,
    }
}

impl<Message: 'static + Clone> From<NavBarToggle<Message>> for Element<'static, Message> {
    fn from(nav_bar_toggle: NavBarToggle<Message>) -> Self {
        let mut widget = super::icon(
            if nav_bar_toggle.nav_bar_active {
                IconSource::EmbeddedSvg(iced::widget::svg::Handle::from_memory(
                    &include_bytes!("../../res/sidebar-active.svg")[..],
                ))
            } else {
                IconSource::Name("open-menu-symbolic".into())
            },
            16,
        )
        .style(theme::Svg::SymbolicActive)
        .apply(iced::widget::container)
        .apply(iced::widget::button)
        .padding([8, 16, 8, 16])
        .style(theme::Button::Text);

        if let Some(message) = nav_bar_toggle.on_nav_bar_toggled {
            widget = widget.on_press(message);
        }

        widget
            .apply(iced::widget::container)
            .center_y()
            .height(Length::Fill)
            .into()
    }
}
