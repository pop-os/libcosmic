// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A control for incremental adjustments of a value.

mod model;
use std::borrow::Cow;

pub use self::model::{Message, Model};

use crate::widget::{button, column, container, icon, row, text};
use crate::{theme, Element};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, Sub};
use apply::Apply;
use iced::{Alignment, Length};
use iced::alignment::Horizontal;
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
                    .center(Length::Fixed(32.0))
                    .apply(button::custom)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .class(theme::Button::Text)
                    .on_press(model::Message::Decrement)
                    .into(),
                text::title4(label)
                    .apply(container)
                    .center_x(Length::Fixed(48.0))
                    .align_y(Alignment::Center)
                    .into(),
                icon::from_name("list-add-symbolic")
                    .size(16)
                    .apply(container)
                    .center(Length::Fixed(32.0))
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
        .width(Length::Shrink)
        .center_y(Length::Fixed(32.0))
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



/// VerticalSpinButton is the state/model of the vertical spin button widget.
/// Restricts T to Add, Sub, Eq, Ord, Display, and Copy so that
/// T can only be numerical values.
pub struct VerticalSpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    label: String,
    step: T,
    value: T,
    min: T,
    max: T,
    on_select: Box<dyn Fn(T) -> M>,
    phantom_data: PhantomData<&'a M>,
}

impl<'a, T, M> VerticalSpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    /// Creates a new vertical spin button widget
    pub fn new(
        label: impl Into<String>,
        step: T,
        value: T,
        min: T,
        max: T,
        on_select: impl Fn(T) -> M + 'static,
    ) -> Self {
        VerticalSpinButton {
            label: label.into(),
            step,
            value,
            min,
            max,
            on_select: Box::new(on_select),
            phantom_data: PhantomData,
        }
    }
}

pub fn vertical_spin_button<T, M>(
    label: impl Into<String>,
    step: T,
    value: T,
    min: T,
    max: T,
    on_select: impl Fn(T) -> M + 'static,
) -> VerticalSpinButton<'static, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    VerticalSpinButton::new(label, step, value, min, max, on_select)
}

fn increment<T, Message>(step: T, val: T, min: T, max: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    std::cmp::min(std::cmp::max(val + step, min), max)
}

fn decrement<T, Message>(step: T, val: T, min: T, max: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    std::cmp::max(std::cmp::min(val - step, max), min)
}

impl<'a, T, Message> From<VerticalSpinButton<'a, T, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    fn from(this: VerticalSpinButton<T, Message>) -> Self {
        let val_text = text(format!("{}", this.value)).size(14);
        let spinner_container = column::with_capacity(3)
            .push(
                button::icon(icon::from_name("list-add-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_select)(increment::<T, Message>(
                        this.step, this.value, this.min, this.max,
                    ))),
            )
            .push(val_text)
            .push(
                button::icon(icon::from_name("list-remove-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_select)(decrement::<T, Message>(
                        this.step, this.value, this.min, this.max,
                    ))),
            )
            .align_x(Horizontal::Center);

        let content_list = column::with_children(vec![
            row::with_capacity(1).push(text(this.label)).into(),
            row::with_children(vec![Element::from(spinner_container)]).into(),
        ])
        .width(75)
        .padding([8, 0])
        .align_x(Alignment::Center);

        Self::new(content_list)
    }
}
