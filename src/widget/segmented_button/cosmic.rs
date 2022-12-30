// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{SegmentedButton, State};

/// Appears as a collection of tabs for developing a tabbed interface.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[must_use]
pub fn view_switcher<Message, Data>(
    state: &State<Data>,
) -> SegmentedButton<Message, crate::Renderer> {
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
pub fn segmented_selection<Message, Data>(
    state: &State<Data>,
) -> SegmentedButton<Message, crate::Renderer> {
    SegmentedButton::new(&state.inner)
        .button_padding([16, 0, 16, 0])
        .button_height(32)
        .style(crate::theme::SegmentedButton::Selection)
        .font_active(crate::font::FONT_SEMIBOLD)
}
