// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A control for incremental adjustments of a value.

use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::marker::PhantomData;
use apply::Apply;
use fraction::Decimal;
use iced::alignment::Horizontal;
use iced::{Alignment, Border, Length, Shadow};
use crate::{
    Element,
    widget::{
        button, 
        column, 
        container, 
        icon, 
        row, 
        text,
    },
};

pub enum Direction {
    Horizontal,
    Vertical,
}

pub struct SpinButton<'a, T, M> 
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy 
{
    label: String,
    /// The amount to increment or decrement the value.
    pub step: T,
    /// The current value of the spin button.
    pub value: T,
    /// The minimum value permitted.
    pub min: T,
    /// The maximum value permitted.
    pub max: T,
    /// The direction that the spin button is laid out; Horizontal (default) or Vertical
    pub direction: Direction,
    on_press: Box<dyn Fn(T) -> M>,
    phantom_data: PhantomData<&'a M>,
}

impl<'a, T, M> SpinButton<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy
{
    /// Create a new default spin button
    pub fn new(
        label: impl Into<String>,
        step: T,
        value: T,
        min: T,
        max: T,
        direction: Direction,
        on_press: impl Fn(T) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            step,
            value,
            min,
            max,
            direction,
            on_press: Box::from(on_press),
            phantom_data: PhantomData,
        }
    }
}

/// Shorthand function to create a default spin button
pub fn spin_button<'a, T, M>(
    label: impl Into<String>,
    step: T,
    value: T,
    min: T,
    max: T,
    direction: Direction,
    on_press: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where 
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy
{
    SpinButton::new(label, step, value, min, max, direction, on_press)
}

fn increment<T>(step: T, value: T, min: T, max: T) -> T
where 
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy
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
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy
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
    T: Add<Output = T> + Sub<Output = T> + PartialEq + PartialOrd + Display + Copy
{
    fn from(this: SpinButton<'a, T, Message>) -> Self {
        //! Matching on the direction enum given by the developer when the
        //! widget is initially created in the application's view function.
        match this.direction {
            Direction::Horizontal => {
                // Create a spinner container variable that contains the row with all of
                // the combined widgets that make up the widget.
                let spinner_container = column::with_capacity(2)
                    .push(
                        row::with_children(
                            vec![
                                // Using the title4 variant of text, just like the original spin button did.
                                text::title4(this.label.clone())
                                .apply(container)
                                .center_x(Length::Fill)
                                .align_y(Alignment::Center)
                                .into(),
                            ]
                        )
                    )
                    .push(
                        row::with_children(
                            vec![                    
                                // Using an button instead of an icon for the decrement functionality.
                                button::icon(icon::from_name("list-remove-symbolic"))
                                    .padding([0, 12])
                                    .on_press((this.on_press)(decrement::<T>(
                                        this.step, this.value, this.min, this.max,
                                        ))
                                    )
                                    .into(),
                                // Using the title4 variant of text for consistency.
                                text::title4(format!("{}", this.value))
                                    .apply(container)
                                    .center_x(Length::Fixed(48.0))
                                    .align_y(Alignment::Center)
                                    .into(),
                                // Using another button for the increment functionality.
                                button::icon(icon::from_name("list-add-symbolic"))
                                    .padding([0, 12])
                                    .on_press((this.on_press)(increment::<T>(
                                        this.step, this.value, this.min, this.max,
                                    ))
                                )
                                .into(),
                            ]
                        )
                        .align_y(Alignment::Center)
                    )
                    .align_x(Alignment::Center);
                
                // Return the horizontal spin button from the match statement.
                Self::new(spinner_container)
            },
            Direction::Vertical => {
                // Create a text widget that holds the value
                let val_text = text(format!("{}", this.value)).size(14);
                // Create a spinner container variable that contains the column with all of
                // the combined widgets that make up the widget.
                let spinner_container = column::with_capacity(3)
                .push(
                    // Use a button for the increment functionality
                    button::icon(icon::from_name("list-add-symbolic"))
                        .padding([0, 12])
                        .on_press((this.on_press)(increment::<T>(
                            this.step, this.value, this.min, this.max,
                        ))),
                )
                // Add the text widget that holds the current value
                .push(val_text)
                .push(
                    // Use a button for the decrement functionality
                    button::icon(icon::from_name("list-remove-symbolic"))
                        .padding([0, 12])
                        .on_press((this.on_press)(decrement::<T>(
                            this.step, this.value, this.min, this.max,
                        ))),
                )
                .align_x(Horizontal::Center);

                // Create a column that contains two rows:
                // First Row -> The label/title for the spin button.
                // Second Row -> The spin button container from above.
                let content_list = column::with_children(vec![
                    row::with_capacity(1).push(text(this.label)).into(),
                    row::with_children(vec![Element::from(spinner_container)]).into(),
                ])
                .width(75)
                .padding([8, 0])
                .align_x(Alignment::Center);
                
                // Return the vertical spin button from the match statement.
                Self::new(content_list)
            }
        }
    }
}

