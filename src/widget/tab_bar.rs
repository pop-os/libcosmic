// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A collection of tabs for developing a tabbed interface.
//!
//! See the [`segmented_button`] module for more details.

use super::segmented_button::{
    self, HorizontalSegmentedButton, Model, SegmentedButton, Selectable, VerticalSegmentedButton,
};

/// A collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a model supplied by the application.
///
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode>,
) -> HorizontalSegmentedButton<'_, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
{
    let space_s = crate::theme::spacing().space_s;
    let space_xs = crate::theme::spacing().space_xs;

    segmented_button::horizontal(model)
        .minimum_button_width(76)
        .maximum_button_width(250)
        .button_height(44)
        .button_padding([space_s, space_xs, space_s, space_xs])
        .style(crate::theme::SegmentedButton::TabBar)
}

/// A collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a model that is maintained the application.
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn vertical<SelectionMode, Message>(
    model: &Model<SelectionMode>,
) -> VerticalSegmentedButton<'_, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    let space_s = crate::theme::spacing().space_s;
    let space_xs = crate::theme::spacing().space_xs;

    SegmentedButton::new(model)
        .minimum_button_width(76)
        .maximum_button_width(250)
        .button_height(44)
        .button_padding([space_s, space_xs, space_s, space_xs])
        .style(crate::theme::SegmentedButton::TabBar)
}
