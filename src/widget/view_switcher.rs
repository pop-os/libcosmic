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
#[must_use]
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode>,
) -> HorizontalSegmentedButton<SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
{
    segmented_button::horizontal(model)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(Some(crate::font::FONT_SEMIBOLD))
}

/// A collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a model that is maintained the application.
///
/// For details on the model, see the [`segmented_button`] module for more details.
#[must_use]
pub fn vertical<SelectionMode, Message>(
    model: &Model<SelectionMode>,
) -> VerticalSegmentedButton<SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    SegmentedButton::new(model)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(Some(crate::font::FONT_SEMIBOLD))
}
