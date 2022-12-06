// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use apply::Apply;
use derive_setters::Setters;
use iced::{alignment::Vertical, Length};
use crate::{Element, theme};

#[derive(Setters)]
pub struct NavButton<'a, Message> {
    title: &'a str,
    sidebar_active: bool,
    #[setters(strip_option)]
    on_sidebar_toggled: Option<Message>,
}

#[must_use]
pub fn nav_button<Message>(title: &str) -> NavButton<Message> {
    NavButton {
        title,
        sidebar_active: false,
        on_sidebar_toggled: None,
    }
}

impl<'a, Message: 'static + Clone> From<NavButton<'a, Message>> for Element<'a, Message> {
    fn from(nav_button: NavButton<'a, Message>) -> Self {
        let text = iced::widget::text(&nav_button.title)
            .style(theme::Text::Accent)
            .vertical_alignment(Vertical::Center)
            .width(Length::Shrink)
            .height(Length::Fill);

        let icon = super::icon(
            if nav_button.sidebar_active {
                "go-previous-symbolic"
            } else {
                "go-next-symbolic"
            },
            24,
        )
        .style(theme::Svg::SymbolicActive)
        .width(Length::Units(24))
        .height(Length::Fill);

        let mut widget = iced::widget::row!(text, crate::widget::vertical_rule(4), icon)
            .padding(4)
            .spacing(4)
            .apply(iced::widget::button)
            .style(theme::Button::Secondary);

        if let Some(message) = nav_button.on_sidebar_toggled.clone() {
            widget = widget.on_press(message);
        }

        widget.apply(iced::widget::container)
            .center_y()
            .height(Length::Fill)
            .into()
    }
}