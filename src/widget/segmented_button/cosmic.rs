// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{
    horizontal_segmented_button, HorizontalSegmentedButton, Model, SegmentedButton, Selectable,
    VerticalSegmentedButton,
};

/// Appears as a collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a model supplied by the application.
#[must_use]
pub fn horizontal_view_switcher<SelectionMode, Component, Message>(
    model: &Model<SelectionMode, Component>,
) -> HorizontalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    SelectionMode: Selectable,
{
    horizontal_segmented_button(model)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a selection of choices for choosing between.
///
/// The data for the widget comes from a model that is maintained the application.
#[must_use]
pub fn horizontal_segmented_selection<SelectionMode, Component, Message>(
    model: &Model<SelectionMode, Component>,
) -> HorizontalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    SelectionMode: Selectable,
{
    SegmentedButton::new(model)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a selection of choices for choosing between.
///
/// The data for the widget comes from a model that is maintained the application.
#[must_use]
pub fn vertical_segmented_selection<SelectionMode, Component, Message>(
    model: &Model<SelectionMode, Component>,
) -> VerticalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    SelectionMode: Selectable,
{
    SegmentedButton::new(model)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a model that is maintained the application.
#[must_use]
pub fn vertical_view_switcher<SelectionMode, Component, Message>(
    model: &Model<SelectionMode, Component>,
) -> VerticalSegmentedButton<SelectionMode, Message, crate::Renderer>
where
    SelectionMode: Selectable,
{
    SegmentedButton::new(model)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(crate::font::FONT_SEMIBOLD)
}
