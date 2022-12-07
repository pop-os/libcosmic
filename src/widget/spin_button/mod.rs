// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod model;
pub use self::model::SpinButtonModel;

use crate::widget::icon;
use crate::{theme, Element};
use apply::Apply;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, row, text},
    Alignment, Background, Length,
};
use std::hash::Hash;

pub struct SpinButton<T> {
    value: T,
}

/// A message emitted by the [`SpinButton`] widget.
#[derive(Clone, Copy, Debug, Hash)]
pub enum SpinMessage {
    Increment,
    Decrement,
}

pub fn spin_button<T: 'static + Clone + Hash + ToString>(value: T) -> SpinButton<T> {
    SpinButton::new(value)
}

impl<T: 'static + Clone + Hash + ToString> SpinButton<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn into_element(self) -> Element<'static, SpinMessage> {
        let Self { value } = self;
        Element::from(iced_lazy::lazy(
            value.clone(),
            move || -> Element<'static, SpinMessage> {
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
                            .on_press(SpinMessage::Decrement),
                        text(value.clone())
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
                            .on_press(SpinMessage::Increment),
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
            },
        ))
    }
}

impl<'a, T: 'static + Clone + Hash + ToString> From<SpinButton<T>> for Element<'a, SpinMessage> {
    fn from(spin_button: SpinButton<T>) -> Self {
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
