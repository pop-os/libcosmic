// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

use crate::{theme, Element};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, row, text},
    Alignment, Background, Length,
};

pub fn spin_button<T, Message>(value: T, on_increment: Message, on_decrement: Message) -> SpinButton<T, Message>
where T: 'static + Clone + Hash + ToString,
    Message: 'static + Clone + Hash
{
    SpinButton::new(value, on_increment, on_decrement)
}

#[derive(Hash)]
pub struct SpinButton<T, Message> {
    on_increment: Message,
    on_decrement: Message,
    value: T,
}

impl<T, Message: 'static + Clone + Hash> SpinButton<T, Message>
where T: 'static + Clone + Hash + ToString,
    Message: 'static + Clone + Hash
{
    pub fn new(value: T, on_increment: Message, on_decrement: Message) -> Self {
        Self { on_increment, on_decrement, value }
    }

    pub fn into_element(self) -> Element<'static, Message> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        iced_lazy::lazy(hasher.finish(), move || -> Element<'static, Message> {
            container(
                row![
                    button(
                        container(text("-").size(26).vertical_alignment(Vertical::Center))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Button::Text)
                    .on_press(self.on_decrement.clone()),
                    container(text(self.value.clone()).vertical_alignment(Vertical::Center))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    button(
                        container(text("+").size(26).vertical_alignment(Vertical::Center))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Button::Text)
                    .on_press(self.on_increment.clone()),
                ]
                .width(Length::Fill)
                .height(Length::Units(32))
                .align_items(Alignment::Center),
            )
            .padding([4, 4])
            .align_y(Vertical::Center)
            .width(Length::Units(95))
            .height(Length::Units(32))
            .style(theme::Container::Custom(container_style))
            .into()
        }).into()
    }
}

impl<'a, T, Message> From<SpinButton<T, Message>> for Element<'a, Message>
where T: 'static + Clone + Hash + ToString,
    Message: 'static + Clone + Hash
{
    fn from(spin_button: SpinButton<T, Message>) -> Self {
        spin_button.into_element()
    }
}

fn container_style(theme: &crate::Theme) -> iced_style::container::Appearance {
    let secondary = &theme.cosmic().secondary;
    let accent = &theme.cosmic().accent;
    iced_style::container::Appearance {
        text_color: None,
        background: Some(Background::Color(secondary.component.base.into())),
        border_radius: 24.0,
        border_width: 0.0,
        border_color: accent.base.into(),
    }
}