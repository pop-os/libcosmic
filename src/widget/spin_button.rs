// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A control for incremental adjustments of a value.

use crate::{
    Element, theme,
    widget::{button, column, container, icon, row, text},
};
use apply::Apply;
use iced::{Alignment, Length};
use iced::{Border, Shadow};
use std::borrow::Cow;
use std::ops::{Add, Sub};

/// Horizontal spin button widget.
pub fn spin_button<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    #[cfg(feature = "a11y")] name: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let mut button = SpinButton::new(
        label,
        value,
        step,
        min,
        max,
        Orientation::Horizontal,
        on_press,
    );

    #[cfg(feature = "a11y")]
    {
        button = button.name(name.into());
    }

    button
}

/// Vertical spin button widget.
pub fn vertical<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    #[cfg(feature = "a11y")] name: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let mut button = SpinButton::new(
        label,
        value,
        step,
        min,
        max,
        Orientation::Horizontal,
        on_press,
    );

    #[cfg(feature = "a11y")]
    {
        button = button.name(name.into());
    }

    button
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
    /// A name for screen reader support.
    #[cfg(feature = "a11y")]
    name: Cow<'a, str>,
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
            #[cfg(feature = "a11y")]
            name: Cow::Borrowed(""),
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

    #[cfg(feature = "a11y")]
    pub(self) fn name(mut self, name: Cow<'a, str>) -> Self {
        self.name = name;
        self
    }
}

fn increment<T>(value: T, step: T, _min: T, max: T) -> T
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    if value > max - step {
        max
    } else {
        value + step
    }
}

fn decrement<T>(value: T, step: T, min: T, _max: T) -> T
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

fn make_button<'a, T, Message>(
    spin_button: &SpinButton<'a, T, Message>,
    icon: &'static str,
    #[cfg(feature = "a11y")] name: String,
    operation: fn(T, T, T, T) -> T,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let mut button = icon::from_name(icon)
        .apply(button::icon)
        .on_press((spin_button.on_press)(operation(
            spin_button.value,
            spin_button.step,
            spin_button.min,
            spin_button.max,
        )));

    #[cfg(feature = "a11y")]
    {
        button = button.name(name.clone());
    }

    button.into()
}

fn horizontal_variant<T, Message>(spin_button: SpinButton<'_, T, Message>) -> Element<'_, Message>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let decrement_button = make_button(
        &spin_button,
        "list-remove-symbolic",
        #[cfg(feature = "a11y")]
        [&spin_button.name, " decrease"].concat(),
        decrement,
    );
    let increment_button = make_button(
        &spin_button,
        "list-add-symbolic",
        #[cfg(feature = "a11y")]
        [&spin_button.name, " increase"].concat(),
        increment,
    );
    let label = text::body(spin_button.label)
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
    let decrement_button = make_button(
        &spin_button,
        "list-remove-symbolic",
        [&spin_button.label, " decrease"].concat(),
        decrement,
    );
    let increment_button = make_button(
        &spin_button,
        "list-add-symbolic",
        [&spin_button.label, " increase"].concat(),
        increment,
    );

    let label = text::body(spin_button.label)
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
    let accent = &cosmic_theme.accent;
    let corners = &cosmic_theme.corner_radii;
    let current_container = theme.current_container();
    let border = if theme.theme_type.is_high_contrast() {
        Border {
            radius: corners.radius_s.into(),
            width: 1.,
            color: current_container.component.border.into(),
        }
    } else {
        Border {
            radius: corners.radius_s.into(),
            width: 0.0,
            color: accent.base.into(),
        }
    };

    iced_widget::container::Style {
        icon_color: Some(current_container.on.into()),
        text_color: Some(current_container.on.into()),
        background: None,
        border,
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
