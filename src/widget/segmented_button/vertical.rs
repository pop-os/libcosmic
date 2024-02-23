// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Implementation details for the vertical layout of a segmented button.

use super::model::{Entity, Model, Selectable};
use super::style::StyleSheet;
use super::widget::{ItemBounds, LocalState, SegmentedButton, SegmentedVariant};

use iced::{Alignment, Length, Rectangle, Size};
use iced_core::layout;

/// A type marker defining the vertical variant of a [`SegmentedButton`].
pub struct Vertical;

/// Vertical [`SegmentedButton`].
pub type VerticalSegmentedButton<'a, SelectionMode, Message> =
    SegmentedButton<'a, Vertical, SelectionMode, Message>;

/// Vertical implementation of the [`SegmentedButton`].
///
/// For details on the model, see the [`segmented_button`](super) module for more details.
#[must_use]
pub fn vertical<SelectionMode, Message>(
    model: &Model<SelectionMode>,
) -> SegmentedButton<Vertical, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    SegmentedButton::new(model)
}

impl<'a, SelectionMode, Message> SegmentedVariant
    for SegmentedButton<'a, Vertical, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance {
        theme.vertical(style)
    }

    #[allow(clippy::cast_precision_loss)]
    fn variant_bounds<'b>(
        &'b self,
        state: &'b LocalState,
        mut bounds: Rectangle,
    ) -> Box<dyn Iterator<Item = ItemBounds> + 'b> {
        let spacing = f32::from(self.spacing);

        Box::new(
            self.model
                .order
                .iter()
                .copied()
                .enumerate()
                .map(move |(nth, key)| {
                    let mut layout_bounds = bounds;

                    let layout_size = state.internal_layout[nth].0;

                    layout_bounds.height = layout_size.height;

                    bounds.y += layout_bounds.height + spacing;

                    ItemBounds::Button(key, layout_bounds)
                }),
        )
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn variant_layout(
        &self,
        state: &mut LocalState,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> Size {
        state.internal_layout.clear();
        let limits = limits.width(self.width);

        let (width, mut height) = self.max_button_dimensions(state, renderer);

        for (size, actual) in &mut state.internal_layout {
            size.width = width;
            actual.width = height;
        }

        let num = self.model.items.len();
        let spacing = f32::from(self.spacing);

        if num != 0 {
            height = (num as f32 * height) + (num as f32 * spacing) - spacing;
        }

        limits.height(Length::Fixed(height)).resolve(
            self.width,
            self.height,
            Size::new(width, height),
        )
    }
}
