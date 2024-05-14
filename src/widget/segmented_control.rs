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
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode, Message>,
) -> HorizontalSegmentedButton<SelectionMode, Message>
where
    Model<SelectionMode, Message>: Selectable,
{
    let theme = crate::theme::active();
    let space_s = theme.cosmic().space_s();
    let space_xxs = theme.cosmic().space_xxs();

    segmented_button::horizontal(model)
        .button_alignment(iced::Alignment::Center)
        .dividers(true)
        .button_height(32)
        .button_padding([space_s, 0, space_s, 0])
        .button_spacing(space_xxs)
        .style(crate::theme::SegmentedButton::Control)
        .font_active(Some(crate::font::FONT_SEMIBOLD))
}

/// A selection of multiple choices appearing as a conjoined button.
///
/// The data for the widget comes from a model that is maintained the application.
///
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn vertical<SelectionMode, Message>(
    model: &Model<SelectionMode, Message>,
) -> VerticalSegmentedButton<SelectionMode, Message>
where
    Model<SelectionMode, Message>: Selectable,
    SelectionMode: Default,
{
    let theme = crate::theme::active();
    let space_s = theme.cosmic().space_s();
    let space_xxs = theme.cosmic().space_xxs();

    segmented_button::vertical(model)
        .button_alignment(iced::Alignment::Center)
        .dividers(true)
        .button_height(32)
        .button_padding([space_s, 0, space_s, 0])
        .button_spacing(space_xxs)
        .style(crate::theme::SegmentedButton::Control)
        .font_active(Some(crate::font::FONT_SEMIBOLD))
}
