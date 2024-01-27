// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Implementation details for the horizontal layout of a segmented button.

use super::model::{Entity, Model, Selectable};
use super::style::StyleSheet;
use super::widget::{LocalState, SegmentedButton, SegmentedVariant};

use iced::{Length, Rectangle, Size};
use iced_core::layout;
use iced_core::text::Renderer;

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
    ) -> impl Iterator<Item = (Entity, Rectangle)> {
        let num = state.buttons_visible;
        let spacing = f32::from(self.spacing);
        let mut homogenous_width = 0.0;

        if Length::Shrink != self.width || state.collapsed {
            let mut width_offset = 0.0;
            if state.collapsed {
                bounds.x += f32::from(self.button_height);
                width_offset = f32::from(self.button_height) * 2.0;
            }

            homogenous_width = ((num as f32).mul_add(-spacing, bounds.width - width_offset)
                + spacing)
                / num as f32;
        }

        self.model
            .order
            .iter()
            .copied()
            .enumerate()
            .skip(state.buttons_offset)
            .take(state.buttons_visible)
            .map(move |(nth, key)| {
                let mut this_bounds = bounds;

                if !state.collapsed && Length::Shrink == self.width {
                    this_bounds.width = state.internal_layout[nth].width;
                } else {
                    this_bounds.width = homogenous_width;
                }

                bounds.x += this_bounds.width + spacing;
                (key, this_bounds)
            })
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
        let num = self.model.order.len();
        let mut total_width = 0.0;
        let spacing = f32::from(self.spacing);
        let limits = limits.width(self.width);
        let mut size;

        if state.known_length != num {
            if state.known_length > num {
                state.buttons_offset -= state.buttons_offset.min(state.known_length - num);
            } else {
                state.buttons_offset += num - state.known_length;
            }

            state.known_length = num;
        }

        if let Length::Shrink = self.width {
            // Buttons will be rendered at their smallest widths possible.
            state.internal_layout.clear();

            let font = renderer.default_font();
            let mut total_height = 0.0f32;

            for &button in &self.model.order {
                let (mut width, height) = self.button_dimensions(state, font, button);
                width = f32::from(self.minimum_button_width).max(width);
                total_width += width + spacing;
                total_height = total_height.max(height);

                state.internal_layout.push(Size::new(width, height));
            }

            // Get the max available width for placing buttons into.
            let max_size = limits
                .height(Length::Fixed(total_height))
                .resolve(Size::new(f32::MAX, total_height));

            let mut visible_width = f32::from(self.button_height) * 2.0;
            state.buttons_visible = 0;

            for button_size in &state.internal_layout {
                visible_width += button_size.width;

                if max_size.width >= visible_width {
                    state.buttons_visible += 1;
                } else {
                    break;
                }

                visible_width += spacing;
            }

            state.collapsed = num > 1 && state.buttons_visible != num;

            // If collapsed, use the maximum width available.
            visible_width = if state.collapsed {
                max_size.width
            } else {
                total_width
            };

            size = limits
                .width(Length::Fixed(visible_width))
                .height(Length::Fixed(total_height))
                .resolve(Size::new(visible_width, total_height));
        } else {
            // Buttons will be rendered with equal widths.
            state.buttons_visible = self.model.items.len();
            let (width, height) = self.max_button_dimensions(state, renderer, limits.max());
            let total_width = (state.buttons_visible as f32) * (width + spacing);

            size = limits
                .height(Length::Fixed(height))
                .resolve(Size::new(total_width, height));

            let actual_width = size.width as usize;
            let minimum_width = state.buttons_visible * self.minimum_button_width as usize;
            state.collapsed = actual_width < minimum_width;

            if state.collapsed {
                size = limits
                    .height(Length::Fixed(height))
                    .resolve(Size::new(f32::MAX, height));

                state.buttons_visible =
                    (actual_width / self.minimum_button_width as usize).min(state.buttons_visible);
            }
        }

        if !state.collapsed {
            state.buttons_offset = 0;
        }

        layout::Node::new(size)
    }
}
