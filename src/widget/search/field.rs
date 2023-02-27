// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::iced::{
    self,
    widget::{container, Button},
    Background, Length,
};
use crate::Renderer;
use apply::Apply;

/// A search field for COSMIC applications.
pub fn field<Message: 'static + Clone>(
    id: iced::widget::text_input::Id,
    phrase: &str,
    on_change: fn(String) -> Message,
    on_clear: Message,
    on_submit: Option<Message>,
) -> Field<Message> {
    Field {
        id,
        phrase,
        on_change,
        on_clear,
        on_submit,
    }
}

/// A search field for COSMIC applications.
#[must_use]
pub struct Field<'a, Message: 'static + Clone> {
    id: iced::widget::text_input::Id,
    phrase: &'a str,
    on_change: fn(String) -> Message,
    on_clear: Message,
    on_submit: Option<Message>,
}

impl<'a, Message: 'static + Clone> Field<'a, Message> {
    pub fn into_element(mut self) -> crate::Element<'a, Message> {
        let mut input = iced::widget::text_input("", self.phrase, self.on_change)
            .style(crate::theme::TextInput::Search)
            .width(Length::Fill)
            .id(self.id);

        if let Some(message) = self.on_submit.take() {
            input = input.on_submit(message);
        }

        iced::widget::row!(
            super::icon::search(16),
            input,
            clear_button().on_press(self.on_clear)
        )
        .width(Length::Units(300))
        .height(Length::Units(38))
        .padding([0, 16])
        .spacing(8)
        .align_items(iced::Alignment::Center)
        .apply(container)
        .style(crate::theme::Container::Custom(active_style))
        .into()
    }
}

impl<'a, Message: 'static + Clone> From<Field<'a, Message>> for crate::Element<'a, Message> {
    fn from(field: Field<'a, Message>) -> Self {
        field.into_element()
    }
}

fn clear_button<Message: 'static>() -> Button<'static, Message, Renderer> {
    super::icon::edit_clear(16)
        .style(crate::theme::Svg::Symbolic)
        .apply(iced::widget::button)
        .style(crate::theme::Button::Text)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn active_style(theme: &crate::Theme) -> container::Appearance {
    let cosmic = &theme.cosmic();
    let mut neutral_7 = cosmic.palette.neutral_7;
    neutral_7.alpha = 0.25;
    iced::widget::container::Appearance {
        text_color: Some(cosmic.palette.neutral_9.into()),
        background: Some(Background::Color(neutral_7.into())),
        border_radius: 24.0,
        border_width: 2.0,
        border_color: cosmic.accent.focus.into(),
    }
}
