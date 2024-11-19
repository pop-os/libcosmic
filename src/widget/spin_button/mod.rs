// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

// Updated by Bryan Hyland <bryan.hyland32@gmail.com>
// Updated on: 18Nov24

//! A control for incremental adjustments of a value.

use crate::{
    widget::{button, column, container, icon, row, text},
    Element,
};
use apply::Apply;
use iced::alignment::Horizontal;
use iced::{Alignment, Length};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, Sub};

#[derive(Clone, Copy)]
enum Orientation {
    Horizontal,
    Vertical,
}

pub struct SpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    /// The label that the spin button widget will have.
    /// It is placed on the top of and centered on the spin button widget itself.
    label: String,
    /// The amount to increment or decrement the value.
    step: T,
    /// The current value of the spin button.
    /// It is displayed in the center of the spin button widget, no matter the orientation.
    value: T,
    /// The minimum value permitted.
    /// If the value is decremented below this value the current value will rollover to the max value.
    min: T,
    /// The maximum value permitted.
    /// If the value is incremented above this value the current value will rollover to the min value.
    max: T,
    /// The direction that the spin button is laid out; Orientation::Horizontal or Orientation::Vertical
    orientation: Orientation,
    /// The message that the spin button emits to the application's update function.
    on_press: Box<dyn Fn(T) -> M>,
    phantom_data: PhantomData<&'a M>,
}

impl<'a, T, M> SpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    /// Create a new new button
    fn new(
        label: impl Into<String>,
        step: T,
        value: T,
        min: T,
        max: T,
        orientation: Orientation,
        on_press: impl Fn(T) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            step,
            value,
            min,
            max,
            orientation,
            on_press: Box::from(on_press),
            phantom_data: PhantomData,
        }
    }
}

/// Shorthand function to create a new spin button
pub fn spin_button<'a, T, M>(
    label: impl Into<String>,
    step: T,
    value: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    SpinButton::new(
        label,
        step,
        value,
        min,
        max,
        Orientation::Horizontal,
        on_press,
    )
}

/// Shorthand to create a standard (horizontal) spin button widget
pub fn vertical<'a, T, M>(
    label: impl Into<String>,
    step: T,
    value: T,
    min: T,
    max: T,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    SpinButton::new(
        label,
        step,
        value,
        min,
        max,
        Orientation::Vertical,
        on_press,
    )
}

fn increment<T>(step: T, value: T, min: T, max: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    //! Make it roll over back to min if the increase is too high
    if value + step > max {
        min
    } else {
        value + step
    }
}

fn decrement<T>(step: T, value: T, min: T, max: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    //! Make it roll over back to max if the decrese is too low
    if value - step < min {
        max
    } else {
        value - step
    }
}

impl<'a, T, Message> From<SpinButton<'a, T, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    fn from(this: SpinButton<'a, T, Message>) -> Self {
        // Matching on the direction enum given by the developer when the
        // widget is initially created in the application's view function.
        match this.orientation {
            Orientation::Horizontal => create_horizontal_spin_button(&this),
            Orientation::Vertical => create_vertical_spin_button(&this),
        }
    }
}

// Helper Functions
// Create a horizontal spin button
// Implemented to make the creation easier to read in the from function for Element implementation.
fn create_horizontal_spin_button<'a, T, Message>(
    spin_btn: &SpinButton<T, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    // Create a spinner container variable that contains the row with all of
    // the combined widgets that make up the widget.
    let spinner_container = column::with_capacity(2)
        .push(row::with_children(vec![
            // Using the title4 variant of text, just like the original spin button did.
            text::title4(spin_btn.label.clone())
                .apply(container)
                .center_x(Length::Fill)
                .align_y(Alignment::Center)
                .into(),
        ]))
        .push(
            row::with_children(vec![
                // Using an button instead of an icon for the decrement functionality.
                button::icon(icon::from_name("list-remove-symbolic"))
                    .padding([0, 12])
                    .on_press((spin_btn.on_press)(decrement::<T>(
                        spin_btn.step,
                        spin_btn.value,
                        spin_btn.min,
                        spin_btn.max,
                    )))
                    .into(),
                // Using the title4 variant of text for consistency.
                text::title4(format!("{}", spin_btn.value))
                    .apply(container)
                    .center_x(Length::Fixed(48.0))
                    .align_y(Alignment::Center)
                    .into(),
                // Using another button for the increment functionality.
                button::icon(icon::from_name("list-add-symbolic"))
                    .padding([0, 12])
                    .on_press((spin_btn.on_press)(increment::<T>(
                        spin_btn.step,
                        spin_btn.value,
                        spin_btn.min,
                        spin_btn.max,
                    )))
                    .into(),
            ])
            .align_y(Alignment::Center),
        )
        .align_x(Alignment::Center);

    // Return the horizontal spin button from the match statement.
    Element::new(spinner_container)
}

// Used to create a vertical spin button widget.
// Implemented to make the creation easier to read in the from function for Element implementation.
fn create_vertical_spin_button<'a, T, Message>(
    spin_btn: &SpinButton<T, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy,
{
    // Create a text widget that holds the value
    let val_text = text::title4(format!("{}", spin_btn.value));
    // Create a spinner container variable that contains the column with all of
    // the combined widgets that make up the widget.
    let spinner_container = column::with_capacity(3)
        .push(
            // Use a button for the increment functionality
            button::icon(icon::from_name("list-add-symbolic"))
                .padding([5, 0])
                .on_press((spin_btn.on_press)(increment::<T>(
                    spin_btn.step,
                    spin_btn.value,
                    spin_btn.min,
                    spin_btn.max,
                ))),
        )
        // Add the text widget that holds the current value
        .push(val_text)
        .push(
            // Use a button for the decrement functionality
            button::icon(icon::from_name("list-remove-symbolic"))
                .padding([5, 0])
                .on_press((spin_btn.on_press)(decrement::<T>(
                    spin_btn.step,
                    spin_btn.value,
                    spin_btn.min,
                    spin_btn.max,
                ))),
        )
        .align_x(Horizontal::Center);

    // Create a column that contains two rows:
    // First Row -> The label/title for the spin button.
    // Second Row -> The spin button container from above.
    let content_list = column::with_children(vec![
        row::with_capacity(1)
            .push(text::title4(spin_btn.label.clone()))
            .into(),
        row::with_children(vec![Element::from(spinner_container)]).into(),
    ])
    .width(75)
    .padding([8, 0])
    .align_x(Alignment::Center);

    // Return the vertical spin button from the match statement.
    Element::new(content_list)
}
