// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Implementation details for the horizontal layout of a segmented button.

use super::model::{Model, Selectable};
use super::style::StyleSheet;
use super::widget::{ItemBounds, LocalState, SegmentedButton, SegmentedVariant};

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
pub fn horizontal<SelectionMode: Default, Message>(
    model: &Model<SelectionMode>,
) -> SegmentedButton<'_, Horizontal, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
{
    SegmentedButton::new(model)
}

impl<SelectionMode, Message> SegmentedVariant
    for SegmentedButton<'_, Horizontal, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    const VERTICAL: bool = false;

    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance {
        theme.horizontal(style)
    }

    #[allow(clippy::cast_precision_loss)]
    fn variant_bounds<'b>(
        &'b self,
        state: &'b LocalState,
        mut bounds: Rectangle,
    ) -> Box<dyn Iterator<Item = ItemBounds> + 'b> {
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

        let is_control = matches!(self.style, crate::theme::SegmentedButton::Control);

        Box::new(
            self.model
                .order
                .iter()
                .copied()
                .enumerate()
                .skip(state.buttons_offset)
                .take(state.buttons_visible)
                .flat_map(move |(nth, key)| {
                    let mut layout_bounds = bounds;

                    let layout_size = &state.internal_layout[nth].0;

                    if !state.collapsed && Length::Shrink == self.width {
                        layout_bounds.width = layout_size.width;
                    } else {
                        layout_bounds.width = homogenous_width;
                    }

                    bounds.x += layout_bounds.width + spacing;

                    let button_bounds = ItemBounds::Button(key, layout_bounds);
                    let mut divider = None;

                    if self.dividers && is_control && nth + 1 < num {
                        divider = Some(ItemBounds::Divider(
                            Rectangle {
                                width: 1.0,
                                ..bounds
                            },
                            true,
                        ));

                        bounds.x += 1.0;
                    }

                    std::iter::once(button_bounds).chain(divider)
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
        let num = self.model.order.len();
        let spacing = f32::from(self.spacing);
        let size;

        let mut reduce_button_offset = false;
        if state.known_length != num {
            if state.known_length > num {
                state.buttons_offset -= state.buttons_offset.min(state.known_length - num);
            } else {
                reduce_button_offset = true;
            }

            state.known_length = num;
        }

        if let Length::Shrink = self.width {
            // Buttons will be rendered at their smallest widths possible.
            let max_height = self.max_button_dimensions(state, renderer).1;

            // Get the max available width for placing buttons into.
            let max_size = limits.height(Length::Fixed(max_height)).resolve(
                Length::Fill,
                max_height,
                Size::new(limits.max().width, max_height),
            );

            let mut visible_width = 0.0;
            state.buttons_visible = 0;

            for (button_size, _actual_size) in &state.internal_layout {
                visible_width += button_size.width;

                if max_size.width - spacing >= visible_width {
                    state.buttons_visible += 1;
                } else {
                    visible_width = max_size.width - max_height;
                    break;
                }

                visible_width += spacing;
            }

            visible_width -= spacing;

            state.collapsed = num > 1 && state.buttons_visible != num;

            size = limits
                .height(Length::Fixed(max_height))
                .min_width(visible_width)
                .min();
        } else {
            // Buttons will be rendered with equal widths.
            state.buttons_visible = self.model.items.len();

            let mut width = 0.0f32;
            let font = renderer.default_font();

            for key in self.model.order.iter().copied() {
                let (button_width, button_height) = self.button_dimensions(state, font, key);

                state.internal_layout.push((
                    Size::new(button_width, button_height),
                    Size::new(
                        button_width
                            - f32::from(self.button_padding[0])
                            - f32::from(self.button_padding[2]),
                        button_height,
                    ),
                ));

                width = width.max(button_width);
            }

            let height = f32::from(self.button_height);

            size = limits.height(Length::Fixed(height)).max();

            let actual_width = size.width as usize;
            let minimum_width = state.buttons_visible * self.minimum_button_width as usize;
            state.collapsed = actual_width < minimum_width;

            if state.collapsed {
                state.buttons_visible =
                    (actual_width / self.minimum_button_width as usize).min(state.buttons_visible);
            }
        }

        if !state.collapsed {
            state.buttons_offset = 0;
        } else if reduce_button_offset {
            state.buttons_offset = num - state.buttons_visible;
        }

        size
    }
}
