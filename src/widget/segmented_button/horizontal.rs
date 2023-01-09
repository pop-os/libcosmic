// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Implementation details for the horizontal layout of a segmented button.

use super::model::Model;
use super::selection_modes::Selectable;
use super::style::StyleSheet;
use super::widget::{SegmentedButton, SegmentedVariant};

use iced::{Length, Rectangle, Size};
use iced_native::layout;

/// Horizontal [`SegmentedButton`].
pub type HorizontalSegmentedButton<'a, SelectionMode, Message, Renderer> =
    SegmentedButton<'a, Horizontal, SelectionMode, Message, Renderer>;

/// A type marker defining the horizontal variant of a [`SegmentedButton`].
pub struct Horizontal;

/// Row implementation of the [`SegmentedButton`].
#[must_use]
pub fn horizontal_segmented_button<SelectionMode, Component, Message, Renderer>(
    model: &Model<SelectionMode, Component>,
) -> SegmentedButton<Horizontal, SelectionMode, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer,
    Renderer::Theme: StyleSheet,
    SelectionMode: Selectable,
{
    SegmentedButton::new(model)
}

impl<'a, SelectionMode, Message, Renderer> SegmentedVariant
    for SegmentedButton<'a, Horizontal, SelectionMode, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer,
    Renderer::Theme: StyleSheet,
    SelectionMode: Selectable,
{
    type Renderer = Renderer;

    fn variant_appearance(
        theme: &<Self::Renderer as iced_native::Renderer>::Theme,
        style: &<<Self::Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style,
    ) -> super::Appearance {
        theme.horizontal(style)
    }

    #[allow(clippy::cast_precision_loss)]
    fn variant_button_bounds(&self, mut bounds: Rectangle, nth: usize) -> Rectangle {
        let num = self.model.items.len();
        if num != 0 {
            let spacing = f32::from(self.spacing);
            bounds.width = (bounds.width - (num as f32 * spacing) + spacing) / num as f32;

            if nth != 0 {
                bounds.x += (nth as f32 * bounds.width) + (nth as f32 * spacing);
            }
        }

        bounds
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn variant_layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width);
        let text_size = renderer.default_size();

        let (mut width, height) = self.max_button_dimensions(renderer, text_size, limits.max());

        let num = self.model.items.len();
        let spacing = f32::from(self.spacing);

        if num != 0 {
            width = (num as f32 * width) + (num as f32 * spacing) - spacing;
        }

        let size = limits
            .height(Length::Units(height as u16))
            .resolve(Size::new(width, height));

        layout::Node::new(size)
    }
}
