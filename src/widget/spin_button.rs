// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A control for incremental adjustments of a value.

use crate::{
    theme,
    widget::{button, column, container, icon, row, text},
    Element,
};
use apply::Apply;
use iced::{Alignment, Length};
use iced::{Border, Shadow};
use std::borrow::Cow;
use std::ops::{Add, Sub};

/// Horizontal spin button widget.
pub fn spin_button<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    SpinButton::new(
        label,
        value,
        step,
        min,
        max,
        Orientation::Horizontal,
        on_press,
    )
}

/// Vertical spin button widget.
pub fn vertical<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    SpinButton::new(
        label,
        value,
        step,
        min,
        max,
        Orientation::Vertical,
        on_press,
    )
}

#[derive(Clone, Copy)]
enum Orientation {
    Horizontal,
    Vertical,
}

pub struct SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    /// The formatted value of the spin button.
    label: Cow<'a, str>,
    /// The current value of the spin button.
    value: T,
    /// The amount to increment or decrement the value.
    step: T,
    /// The minimum value permitted.
    min: T,
    /// The maximum value permitted.
    max: T,
    orientation: Orientation,
    on_press: Box<dyn Fn(T) -> M>,
}

impl<'a, T, M> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    /// Create a new new button
    fn new(
        label: impl Into<Cow<'a, str>>,
        value: T,
        step: T,
        min: T,
        max: T,
        orientation: Orientation,
        on_press: impl Fn(T) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            step,
            value: if value < min {
                min
            } else if value > max {
                max
            } else {
                value
            },
            min,
            max,
            orientation,
            on_press: Box::from(on_press),
        }
    }
}

fn increment<T>(value: T, step: T, min: T, max: T) -> T
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    if value > max - step {
        max
    } else {
        value + step
    }
}

fn decrement<T>(value: T, step: T, min: T, max: T) -> T
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    if value < min + step {
        min
    } else {
        value - step
    }
}

impl<'a, T, Message> From<SpinButton<'a, T, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    fn from(this: SpinButton<'a, T, Message>) -> Self {
        match this.orientation {
            Orientation::Horizontal => horizontal_variant(this),
            Orientation::Vertical => vertical_variant(this),
        }
    }
}

fn horizontal_variant<T, Message>(spin_button: SpinButton<'_, T, Message>) -> Element<'_, Message>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let decrement_button = icon::from_name("list-remove-symbolic")
        .apply(button::icon)
        .on_press((spin_button.on_press)(decrement::<T>(
            spin_button.value,
            spin_button.step,
            spin_button.min,
            spin_button.max,
        )));

    let increment_button = icon::from_name("list-add-symbolic")
        .apply(button::icon)
        .on_press((spin_button.on_press)(increment::<T>(
            spin_button.value,
            spin_button.step,
            spin_button.min,
            spin_button.max,
        )));

    let label = text::title4(spin_button.label)
        .apply(container)
        .center_x(Length::Fixed(48.0))
        .align_y(Alignment::Center);

    row::with_capacity(3)
        .push(decrement_button)
        .push(label)
        .push(increment_button)
        .align_y(Alignment::Center)
        .apply(container)
        .class(theme::Container::custom(container_style))
        .into()
}

fn vertical_variant<T, Message>(spin_button: SpinButton<'_, T, Message>) -> Element<'_, Message>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let decrement_button = icon::from_name("list-remove-symbolic")
        .apply(button::icon)
        .on_press((spin_button.on_press)(decrement::<T>(
            spin_button.value,
            spin_button.step,
            spin_button.min,
            spin_button.max,
        )));

    let increment_button = icon::from_name("list-add-symbolic")
        .apply(button::icon)
        .on_press((spin_button.on_press)(increment::<T>(
            spin_button.value,
            spin_button.step,
            spin_button.min,
            spin_button.max,
        )));

    let label = text::title4(spin_button.label)
        .apply(container)
        .center_x(Length::Fixed(48.0))
        .align_y(Alignment::Center);

    column::with_capacity(3)
        .push(increment_button)
        .push(label)
        .push(decrement_button)
        .align_x(Alignment::Center)
        .apply(container)
        .class(theme::Container::custom(container_style))
        .into()
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn container_style(theme: &crate::Theme) -> iced_widget::container::Style {
    let cosmic_theme = &theme.cosmic();
    let mut neutral_10 = cosmic_theme.palette.neutral_10;
    neutral_10.alpha = 0.1;
    let accent = &cosmic_theme.accent;
    let corners = &cosmic_theme.corner_radii;
    iced_widget::container::Style {
        icon_color: Some(accent.base.into()),
        text_color: Some(cosmic_theme.palette.neutral_10.into()),
        background: None,
        border: Border {
            radius: corners.radius_s.into(),
            width: 0.0,
            color: accent.base.into(),
        },
        shadow: Shadow::default(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decrement() {
        assert_eq!(super::decrement(0i32, 10, 15, 35), 15);
    }
}
