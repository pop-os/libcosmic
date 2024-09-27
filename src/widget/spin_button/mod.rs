// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A control for incremental adjustments of a value.

mod model;
use std::borrow::Cow;

pub use self::model::{Message, Model};

use crate::widget::{button, container, icon, row, text};
use crate::{theme, Element};
use apply::Apply;
use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Length,
};
use iced_core::{Border, Shadow};

pub struct SpinButton<'a, Message> {
    label: Cow<'a, str>,
    on_change: Box<dyn Fn(model::Message) -> Message + 'static>,
}

/// A control for incremental adjustments of a value.
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
            row::with_children(vec![
                icon::from_name("list-remove-symbolic")
                    .size(16)
                    .apply(container)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .apply(button::custom)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .class(theme::Button::Text)
                    .on_press(model::Message::Decrement)
                    .into(),
                text::title4(label)
                    .apply(container)
                    .width(Length::Fixed(48.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into(),
                icon::from_name("list-add-symbolic")
                    .size(16)
                    .apply(container)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .apply(button::custom)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .class(theme::Button::Text)
                    .on_press(model::Message::Increment)
                    .into(),
            ])
            .width(Length::Shrink)
            .height(Length::Fixed(32.0))
            .align_y(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .width(Length::Shrink)
        .height(Length::Fixed(32.0))
        .class(theme::Container::custom(container_style))
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
fn container_style(theme: &crate::Theme) -> iced_widget::container::Style {
    let basic = &theme.cosmic();
    let mut neutral_10 = basic.palette.neutral_10;
    neutral_10.alpha = 0.1;
    let accent = &basic.accent;
    let corners = &basic.corner_radii;
    iced_widget::container::Style {
        icon_color: Some(basic.palette.neutral_10.into()),
        text_color: Some(basic.palette.neutral_10.into()),
        background: None,
        border: Border {
            radius: corners.radius_s.into(),
            width: 0.0,
            color: accent.base.into(),
        },
        shadow: Shadow::default(),
    }
}
