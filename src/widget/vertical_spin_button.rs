// Spinner for integer selection
// Author: Bryan Hyland <bryan.hyland32@gmail.com>

use crate::iced::alignment::Horizontal;
use crate::iced::Alignment;
use crate::widget::{button, column, icon, row, text};
use crate::Element;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, Sub};

/// Spinner is the state/model of the vertical spinner widget.
/// Restricts T to Add, Sub, Eq, Ord, Display, and Copy so that
/// T can only be numerical values.
pub struct VerticalSpinner<'a, T, M>
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

impl<'a, T, M> VerticalSpinner<'a, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    /// Creates a new vertical spinner widget
    pub fn new(
        label: impl Into<String>,
        step: T,
        value: T,
        min: T,
        max: T,
        on_select: impl Fn(T) -> M + 'static,
    ) -> Self {
        VerticalSpinner {
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

pub fn vertical_spinner<T, M>(
    label: impl Into<String>,
    step: T,
    value: T,
    min: T,
    max: T,
    on_select: impl Fn(T) -> M + 'static,
) -> VerticalSpinner<'static, T, M>
where
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    VerticalSpinner::new(label, step, value, min, max, on_select)
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

impl<'a, T, Message> From<VerticalSpinner<'a, T, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
    T: Add<Output = T> + Sub<Output = T> + Eq + Ord + Display + Copy,
{
    fn from(this: VerticalSpinner<T, Message>) -> Self {
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
