// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::{button, column, container, horizontal_space, icon, list, row, text};
use crate::{theme, Apply, Element};
use iced::Length;

#[must_use]
pub fn page_list_item<'a, Message: 'static + Clone>(
    title: &'a str,
    description: &'a str,
    icon: &'a str,
    message: Message,
) -> Element<'a, Message> {
    let control = row::with_children(vec![
        horizontal_space(Length::Fill).into(),
        icon::from_name("go-next-symbolic").size(16).into(),
    ]);

    super::settings::item::builder(title)
        .description(description)
        .icon(icon::from_name(icon).size(16))
        .control(control)
        .spacing(16)
        .apply(container)
        .padding([20, 24])
        .style(theme::Container::custom(list::style))
        .apply(button)
        .style(theme::Button::Transparent)
        .on_press(message)
        .into()
}

#[must_use]
pub fn sub_page_header<'a, Message: 'static + Clone>(
    sub_page: &'a str,
    parent_page: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    let previous_button = button::icon(icon::from_name("go-previous-symbolic"))
        .extra_small()
        .label(parent_page)
        .spacing(16)
        .style(button::Style::Link)
        .on_press(on_press);

    let sub_page_header = row::with_capacity(2)
        .push(text::title3(sub_page))
        .push(horizontal_space(Length::Fill));

    column::with_capacity(2)
        .push(previous_button)
        .push(sub_page_header)
        .spacing(6)
        .into()
}
