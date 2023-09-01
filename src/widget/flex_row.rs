// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::ext::CollectionWidget;
use crate::widget::{column, row};
use crate::Element;
use apply::Apply;
use derive_setters::Setters;
use iced_core::{alignment, Length, Size};
use std::cell::RefCell;

/// Responsively generates rows and columns of widgets based on its dimmensions.
#[derive(Setters)]
pub struct FlexRow<'a, Message> {
    #[allow(clippy::type_complexity)]
    #[setters(skip)]
    generator: Box<dyn Fn(&mut Vec<Element<'a, Message>>, Size) -> u16 + 'a>,
    /// Sets the space between each column of items.
    column_spacing: u16,
    /// Sets the space between each item in a row.
    row_spacing: u16,
    /// Sets the max number of items per row.
    max_items: Option<u16>,
    /// Sets the horizontal alignment of the [`FlexRow`].
    align_x: alignment::Horizontal,
    /// Sets the vertical alignment of the [`FlexRow`].
    align_y: alignment::Vertical,
    /// Sets the width of the [`FlexRow`].
    width: Length,
    /// Sets the height of the [`FlexRow`].
    height: Length,
}

/// Responsively generates rows and columns of widgets based on its dimmensions.
///
/// The `generator` input is a closure which must return the max width of all
/// elements created, while storing elements in the provided `Vec`.
///
/// ## Example
///
/// Suppose that there is a `COLOR_VALUE` variable which contains an array of
/// color values, and a `color_button` function which creates an `Element` from
/// a color.
///
/// We already know beforehand that our color buttons will have a fixed width
/// of `70`, so we store elements in the provided `Vec` and return `70`.
///
/// ```ignore
/// use iced_core::{alignment, Length};
///
/// let flex_row = cosmic::widget::flex_row(|vec, _size| {
///     let elements = DEFAULT_COLORS
///         .iter()
///         .cloned()
///         .map(color_button);
///
///     vec.extend(elements);
///
///     70
/// });
///
/// flex_row
///     .column_spacing(12)
///     .row_spacing(16)
///     .width(Length::Fill)
///     .align_x(alignment::Horizontal::Center)
///     .into()
/// ```
pub fn flex_row<'a, Message: 'static>(
    generator: impl Fn(&mut Vec<Element<'a, Message>>, Size) -> u16 + 'a,
) -> FlexRow<'a, Message> {
    FlexRow {
        generator: Box::new(generator),
        column_spacing: 4,
        row_spacing: 4,
        max_items: None,
        align_x: alignment::Horizontal::Left,
        align_y: alignment::Vertical::Top,
        width: Length::Shrink,
        height: Length::Shrink,
    }
}

impl<'a, Message: 'static> From<FlexRow<'a, Message>> for Element<'a, Message> {
    fn from(container: FlexRow<'a, Message>) -> Self {
        let elements = RefCell::new(Vec::new());

        iced::widget::responsive(move |size| {
            let mut elements = elements.borrow_mut();
            let item_width = (container.generator)(&mut elements, size);

            let mut items_per_row = flex_row_items(
                size.width,
                f32::from(item_width),
                f32::from(container.row_spacing),
            ) as usize;

            if let Some(max_items) = container.max_items {
                items_per_row = items_per_row.max(max_items as usize);
            }

            let mut elements_column = Vec::with_capacity(elements.len() / items_per_row);

            let mut iterator = elements.drain(..);

            while let Some(element) = iterator.next() {
                let elements_row = row::with_capacity(items_per_row)
                    .spacing(container.row_spacing)
                    .push(element)
                    .extend(iterator.by_ref().take(items_per_row - 1));

                elements_column.push(elements_row.into());
            }

            column::with_children(elements_column)
                .spacing(container.column_spacing)
                .apply(iced::widget::container)
                .align_x(container.align_x)
                .align_y(container.align_y)
                .width(container.width)
                .height(container.height)
                .into()
        })
        .into()
    }
}

#[allow(clippy::cast_precision_loss)]
fn flex_row_items(available: f32, item_width: f32, spacing: f32) -> u32 {
    let mut items = 2;

    while available >= (item_width + spacing) * items as f32 - spacing {
        items += 1;
    }

    items - 1
}
