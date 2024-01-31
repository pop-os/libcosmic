// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced_core::layout::{Limits, Node};
use iced_core::widget::Tree;
use iced_core::{Padding, Point, Size};

pub fn resolve<Message>(
    renderer: &Renderer,
    limits: &Limits,
    items: &[Element<'_, Message>],
    padding: Padding,
    column_spacing: f32,
    row_spacing: f32,
    tree: &mut [Tree],
) -> Node {
    let limits = limits.shrink(padding);

    let mut nodes = Vec::with_capacity(items.len());

    let max_flex_width = limits.max().width;
    let mut flex_width = 0.0f32;
    let mut flex_height = 0.0f32;

    let mut current_row_width = 0.0f32;
    let mut current_row_height = 0.0f32;

    let mut row_buffer = Vec::<Node>::with_capacity(8);

    for (child, tree) in items.iter().zip(tree.iter_mut()) {
        // Calculate the dimensions of the item.
        let child_node = child.as_widget().layout(tree, renderer, &limits);
        let size = child_node.size();

        // Calculate the required additional width to fit the item into the current row.
        let mut required_width = size.width
            + if row_buffer.is_empty() {
                0.0
            } else {
                row_spacing
            };

        // If it fits, add it to the current row, or create a new one.
        if current_row_width + required_width > max_flex_width {
            if flex_height != 0.0f32 {
                flex_height += column_spacing;
            }

            let mut pos_x = 0.0f32;
            let pos_y = flex_height;

            for mut child_node in row_buffer.drain(..) {
                child_node = child_node.move_to(Point::new(pos_x, pos_y));
                pos_x += row_spacing + child_node.size().width;
                nodes.push(child_node);
            }

            flex_height += current_row_height;
            flex_width = flex_width.max(current_row_width);
            required_width -= row_spacing;
            current_row_width = 0.0;
        }

        current_row_width += required_width;
        current_row_height = current_row_height.max(size.height);

        row_buffer.push(child_node);
    }

    if !row_buffer.is_empty() {
        if flex_height != 0.0f32 {
            flex_height += column_spacing;
        }

        let mut pos_x = 0.0f32;
        let pos_y = flex_height;

        for mut child_node in row_buffer.drain(..) {
            child_node = child_node.move_to(Point::new(pos_x, pos_y));
            pos_x += row_spacing + child_node.size().width;
            nodes.push(child_node);
        }

        flex_height += current_row_height;
        flex_width = flex_width.max(current_row_width);
    }

    let flex_size = limits.resolve(flex_width, flex_height, Size::new(flex_width, flex_height));
    Node::with_children(flex_size.expand(padding), nodes)
}
