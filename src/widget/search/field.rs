// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::iced::{Background, Length};
use crate::widget::{container, icon, row, text_input};
use apply::Apply;

/// A search field for COSMIC applications.
pub fn field<Message: 'static + Clone>(
    id: iced_core::id::Id,
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
    id: iced_core::id::Id,
    phrase: &'a str,
    on_change: fn(String) -> Message,
    on_clear: Message,
    on_submit: Option<Message>,
}

impl<'a, Message: 'static + Clone> Field<'a, Message> {
    pub fn into_element(mut self) -> crate::Element<'a, Message> {
        let input = text_input("", self.phrase)
            .on_input(self.on_change)
            .width(Length::Fill)
            .id(self.id)
            .on_submit_maybe(self.on_submit.take());

        row::with_capacity(3)
            .push(
                icon::from_svg_bytes(&include_bytes!("search.svg")[..])
                    .symbolic(true)
                    .icon()
                    .size(16),
            )
            .push(input)
            .push(clear_button().on_press(self.on_clear))
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(38.0))
            .padding([0, 16])
            .spacing(8)
            .align_items(iced::Alignment::Center)
            .apply(container)
            .style(crate::theme::Container::custom(active_style))
            .into()
    }
}

impl<'a, Message: 'static + Clone> From<Field<'a, Message>> for crate::Element<'a, Message> {
    fn from(field: Field<'a, Message>) -> Self {
        field.into_element()
    }
}

fn clear_button<Message: 'static>() -> crate::widget::IconButton<'static, Message> {
    icon::from_name("edit-clear-symbolic")
        .size(16)
        .apply(crate::widget::button::icon)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn active_style(theme: &crate::Theme) -> container::Appearance {
    let cosmic = &theme.cosmic();
    let mut neutral_7 = cosmic.palette.neutral_7;
    neutral_7.alpha = 0.25;
    iced::widget::container::Appearance {
        icon_color: Some(cosmic.palette.neutral_9.into()),
        text_color: Some(cosmic.palette.neutral_9.into()),
        background: Some(Background::Color(neutral_7.into())),
        border_radius: cosmic.corner_radii.radius_m.into(),
        border_width: 2.0,
        border_color: cosmic.accent.focus.into(),
    }
}
