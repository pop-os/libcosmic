// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A selection of multiple choices appearing as a conjoined button.
//!
//! See the [`segmented_button`] module for more details.

use super::segmented_button::{
    self, HorizontalSegmentedButton, Model, Selectable, VerticalSegmentedButton,
};

/// A selection of multiple choices appearing as a conjoined button.
///
/// The data for the widget comes from a model that is maintained the application.
///
/// For details on the model, see the [`segmented_button`] module for more details.
#[must_use]
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode>,
) -> HorizontalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    Model<SelectionMode>: Selectable,
{
    segmented_button::horizontal(model)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// A selection of multiple choices appearing as a conjoined button.
///
/// The data for the widget comes from a model that is maintained the application.
///
/// For details on the model, see the [`segmented_button`] module for more details.
#[must_use]
pub fn vertical<SelectionMode, Message>(
    model: &Model<SelectionMode>,
) -> VerticalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    segmented_button::vertical(model)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}
