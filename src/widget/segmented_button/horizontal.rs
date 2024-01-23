// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Implementation details for the horizontal layout of a segmented button.

use super::model::{Model, Selectable};
use super::style::StyleSheet;
use super::widget::{LocalState, SegmentedButton, SegmentedVariant};

use iced::{Length, Rectangle, Size};
use iced_core::layout;

/// Horizontal [`SegmentedButton`].
pub type HorizontalSegmentedButton<'a, SelectionMode, Message> =
    SegmentedButton<'a, Horizontal, SelectionMode, Message>;

/// A type marker defining the horizontal variant of a [`SegmentedButton`].
pub struct Horizontal;

/// Horizontal implementation of the [`SegmentedButton`].
///
/// For details on the model, see the [`segmented_button`](super) module for more details.
#[must_use]
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode>,
) -> SegmentedButton<Horizontal, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
{
    SegmentedButton::new(model)
}

impl<'a, SelectionMode, Message> SegmentedVariant
    for SegmentedButton<'a, Horizontal, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance {
        theme.horizontal(style)
    }

    #[allow(clippy::cast_precision_loss)]
    fn variant_button_bounds(
        &self,
        state: &LocalState,
        mut bounds: Rectangle,
        nth: usize,
    ) -> Option<Rectangle> {
        let num = state.buttons_visible;

        // Do not display tabs that are currently hidden due to width constraints.
        if state.collapsed && nth < state.buttons_offset {
            return None;
        }

        if num != 0 {
            let offset_width;
            (bounds.x, offset_width) = if state.collapsed {
                (bounds.x + 16.0, 32.0)
            } else {
                (bounds.x, 0.0)
            };

            let spacing = f32::from(self.spacing);
            bounds.width = ((num as f32).mul_add(-spacing, bounds.width - offset_width) + spacing)
                / num as f32;

            if nth != state.buttons_offset {
                let pos = (nth - state.buttons_offset) as f32;
                bounds.x += pos.mul_add(bounds.width, pos * spacing);
            }
        }

        Some(bounds)
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn variant_layout(
        &self,
        state: &mut LocalState,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);
        let (mut width, height) = self.max_button_dimensions(state, renderer, limits.max());

        let num = self.model.items.len();
        let spacing = f32::from(self.spacing);

        if num != 0 {
            width = (num as f32).mul_add(width, num as f32 * spacing) - spacing;
        }

        let size = limits
            .height(Length::Fixed(height))
            .resolve(Size::new(width, height));

        let actual_width = size.width as usize;
        let minimum_width = self.minimum_button_width as usize * self.model.items.len();

        state.buttons_visible = num;
        state.collapsed = actual_width < minimum_width;
        if state.collapsed {
            state.buttons_visible = (actual_width / self.minimum_button_width as usize).min(num);
        }

        layout::Node::new(size)
    }
}
