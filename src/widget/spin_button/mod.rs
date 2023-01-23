// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod model;
use std::borrow::Cow;

pub use self::model::{Message, Model};

use crate::widget::{icon, text};
use crate::{theme, Element};
use apply::Apply;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, row},
    Alignment, Background, Length,
};

pub struct SpinButton<'a, Message> {
    label: Cow<'a, str>,
    on_change: Box<dyn Fn(model::Message) -> Message + 'static>,
}

pub fn spin_button<'a, Message: 'static>(
    label: impl Into<Cow<'a, str>>,
    on_change: impl Fn(model::Message) -> Message + 'static,
) -> SpinButton<'a, Message> {
    SpinButton::new(label, on_change)
}

impl<'a, Message: 'static> SpinButton<'a, Message> {
    pub fn new(
        label: impl Into<Cow<'a, str>>,
        on_change: impl Fn(model::Message) -> Message + 'static,
    ) -> Self {
        Self {
            on_change: Box::from(on_change),
            label: label.into(),
        }
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        let Self { on_change, label } = self;
        container(
            row![
                icon("list-remove-symbolic", 24)
                    .style(theme::Svg::Symbolic)
                    .apply(container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .apply(button)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Button::Text)
                    .on_press(model::Message::Decrement),
                text(label)
                    .vertical_alignment(Vertical::Center)
                    .apply(container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
                icon("list-add-symbolic", 24)
                    .style(theme::Svg::Symbolic)
                    .apply(container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .apply(button)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Button::Text)
                    .on_press(model::Message::Increment),
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
        .apply(Element::from)
        .map(on_change)
    }
}

impl<'a, Message: 'static> From<SpinButton<'a, Message>> for Element<'a, Message> {
    fn from(spin_button: SpinButton<'a, Message>) -> Self {
        spin_button.into_element()
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
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
