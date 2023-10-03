// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::widget::Assignment;
use crate::{Element, Renderer};
use iced_core::layout::{Limits, Node};
use iced_core::{Alignment, Length, Padding, Point, Size};

use taffy::geometry::{Line, Rect};
use taffy::style::{AlignContent, AlignItems, Display, GridPlacement, Style};
use taffy::style_helpers::{auto, length};
use taffy::Taffy;

#[allow(clippy::too_many_lines)]
pub fn resolve<Message>(
    renderer: &Renderer,
    limits: &Limits,
    items: &[Element<'_, Message>],
    assignments: &[Assignment],
    width: Length,
    height: Length,
    padding: Padding,
    column_alignment: Alignment,
    row_alignment: Alignment,
    column_spacing: f32,
    row_spacing: f32,
) -> Node {
    let max_size = limits.max();

    let mut leafs = Vec::with_capacity(items.len());
    let mut nodes = Vec::with_capacity(items.len());

    let mut taffy = Taffy::with_capacity(items.len() + 1);

    // Attach widgets as child nodes.
    for (child, assignment) in items.iter().zip(assignments.iter()) {
        // Calculate the dimensions of the item.
        let child_node = child.as_widget().layout(renderer, limits);
        let size = child_node.size();

        nodes.push(child_node);

        // Attach widget as leaf to be later assigned to grid.
        let leaf = taffy.new_leaf(Style {
            grid_column: Line {
                start: GridPlacement::Line((assignment.column as i16).into()),
                end: GridPlacement::Span(assignment.height.into()),
            },
            grid_row: Line {
                start: GridPlacement::Line((assignment.row as i16).into()),
                end: GridPlacement::Span(assignment.width.into()),
            },
            size: taffy::geometry::Size {
                width: length(size.width),
                height: length(size.height),
            },

            ..Style::default()
        });

        match leaf {
            Ok(leaf) => leafs.push(leaf),
            Err(why) => {
                tracing::error!(%why, "cannot add leaf node to grid");
                continue;
            }
        }
    }

    let root = taffy.new_with_children(
        Style {
            align_items: Some(match width {
                Length::Fill | Length::FillPortion(_) => AlignItems::Stretch,
                _ => match row_alignment {
                    Alignment::Start => AlignItems::Start,
                    Alignment::Center => AlignItems::Center,
                    Alignment::End => AlignItems::End,
                },
            }),

            display: Display::Grid,

            gap: taffy::geometry::Size {
                width: length(row_spacing),
                height: length(column_spacing),
            },

            justify_items: Some(match height {
                Length::Fill | Length::FillPortion(_) => AlignItems::Stretch,
                _ => match column_alignment {
                    Alignment::Start => AlignItems::Start,
                    Alignment::Center => AlignItems::Center,
                    Alignment::End => AlignItems::End,
                },
            }),

            padding: Rect {
                left: length(padding.left),
                right: length(padding.right),
                top: length(padding.top),
                bottom: length(padding.bottom),
            },

            size: taffy::geometry::Size {
                width: match width {
                    Length::Fixed(fixed) => length(fixed),
                    _ => auto(),
                },
                height: match height {
                    Length::Fixed(fixed) => length(fixed),
                    _ => auto(),
                },
            },

            ..Style::default()
        },
        &leafs,
    );

    let root = match root {
        Ok(root) => root,
        Err(why) => {
            tracing::error!(%why, "grid root style invalid");
            return Node::new(Size::ZERO);
        }
    };

    if let Err(why) = taffy.compute_layout(
        root,
        taffy::geometry::Size {
            width: length(max_size.width),
            height: length(max_size.height),
        },
    ) {
        tracing::error!(%why, "grid layout did not compute");
        return Node::new(Size::ZERO);
    }

    let grid_layout = match taffy.layout(root) {
        Ok(layout) => layout,
        Err(why) => {
            tracing::error!(%why, "cannot get layout of grid");
            return Node::new(Size::ZERO);
        }
    };

    for (leaf, node) in leafs.into_iter().zip(nodes.iter_mut()) {
        if let Ok(leaf_layout) = taffy.layout(leaf) {
            let location = leaf_layout.location;
            node.move_to(Point {
                x: location.x,
                y: location.y,
            });
        }
    }

    let grid_size = Size {
        width: grid_layout.size.width,
        height: grid_layout.size.height,
    };

    Node::with_children(grid_size.pad(padding), nodes)
}
