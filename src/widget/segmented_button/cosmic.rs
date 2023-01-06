// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{
    state::Selectable, HorizontalSegmentedButton, SegmentedButton, State, VerticalSegmentedButton,
};

/// Appears as a collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[must_use]
pub fn horizontal_view_switcher<Selection, Message, Data>(
    state: &State<Selection, Data>,
) -> HorizontalSegmentedButton<Selection, Message, crate::Renderer>
where
    Selection: Selectable,
{
    SegmentedButton::new(&state.inner)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a selection of choices for choosing between.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[must_use]
pub fn horizontal_segmented_selection<Selection, Message, Data>(
    state: &State<Selection, Data>,
) -> HorizontalSegmentedButton<Selection, Message, crate::Renderer>
where
    Selection: Selectable,
{
    SegmentedButton::new(&state.inner)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a selection of choices for choosing between.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[must_use]
pub fn vertical_segmented_selection<Selection, Message, Data>(
    state: &State<Selection, Data>,
) -> VerticalSegmentedButton<Selection, Message, crate::Renderer>
where
    Selection: Selectable,
{
    SegmentedButton::new(&state.inner)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}

/// Appears as a collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[must_use]
pub fn vertical_view_switcher<Selection, Message, Data>(
    state: &State<Selection, Data>,
) -> VerticalSegmentedButton<Selection, Message, crate::Renderer>
where
    Selection: Selectable,
{
    SegmentedButton::new(&state.inner)
        .button_padding([16, 0, 16, 0])
        .button_height(48)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .font_active(crate::font::FONT_SEMIBOLD)
}
