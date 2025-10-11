// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::widget::Assignment;
use crate::{Element, Renderer};
use iced_core::layout::{Limits, Node};
use iced_core::widget::Tree;
use iced_core::{Alignment, Length, Padding, Point, Size};

use taffy::geometry::{Line, Rect};
use taffy::style::{AlignItems, Dimension, Display, GridPlacement, Style};
use taffy::style_helpers::{auto, length};
use taffy::{AlignContent, TaffyTree};

#[allow(clippy::too_many_arguments)]
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
    justify_content: Option<AlignContent>,
    column_spacing: f32,
    row_spacing: f32,
    tree: &mut [Tree],
) -> Node {
    let max_size = limits.max();

    let mut leafs = Vec::with_capacity(items.len());
    let mut nodes = Vec::with_capacity(items.len());

    let mut taffy = TaffyTree::<()>::with_capacity(items.len() + 1);

    // Attach widgets as child nodes.
    for ((child, assignment), tree) in items.iter().zip(assignments.iter()).zip(tree.iter_mut()) {
        // Calculate the dimensions of the item.
        let child_widget = child.as_widget();
        let child_node = child_widget.layout(tree, renderer, limits);
        let size = child_node.size();

        nodes.push(child_node);

        let c_size = child_widget.size();
        let (width, flex_grow, justify_self) = match c_size.width {
            Length::Fill | Length::FillPortion(_) => {
                (Dimension::auto(), 1.0, Some(AlignItems::Stretch))
            }
            _ => (length(size.width), 0.0, None),
        };

        // Attach widget as leaf to be later assigned to grid.
        let leaf = taffy.new_leaf(Style {
            flex_grow,

            grid_column: Line {
                start: GridPlacement::Line((assignment.column as i16).into()),
                end: GridPlacement::Line(
                    (assignment.column as i16 + assignment.width as i16).into(),
                ),
            },

            grid_row: Line {
                start: GridPlacement::Line((assignment.row as i16).into()),
                end: GridPlacement::Line((assignment.row as i16 + assignment.height as i16).into()),
            },

            size: taffy::geometry::Size {
                width,
                height: match c_size.height {
                    Length::Fill | Length::FillPortion(_) => Dimension::auto(),
                    _ => length(size.height),
                },
            },

            justify_self,

            ..Style::default()
        });

        match leaf {
            Ok(leaf) => leafs.push(leaf),
            Err(why) => {
                tracing::error!(?why, "cannot add leaf node to grid");
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
                width: length(column_spacing),
                height: length(row_spacing),
            },

            justify_items: Some(match height {
                Length::Fill | Length::FillPortion(_) => AlignItems::Stretch,
                _ => match column_alignment {
                    Alignment::Start => AlignItems::Start,
                    Alignment::Center => AlignItems::Center,
                    Alignment::End => AlignItems::End,
                },
            }),

            justify_content,

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
            tracing::error!(?why, "grid root style invalid");
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
        tracing::error!(?why, "grid layout did not compute");
        return Node::new(Size::ZERO);
    }

    let grid_layout = match taffy.layout(root) {
        Ok(layout) => layout,
        Err(why) => {
            tracing::error!(?why, "cannot get layout of grid");
            return Node::new(Size::ZERO);
        }
    };

    for (((leaf, child), node), tree) in leafs
        .into_iter()
        .zip(items.iter())
        .zip(nodes.iter_mut())
        .zip(tree)
    {
        if let Ok(leaf_layout) = taffy.layout(leaf) {
            let child_widget = child.as_widget();
            let c_size = child_widget.size();
            match c_size.width {
                Length::Fill | Length::FillPortion(_) => {
                    *node =
                        child_widget.layout(tree, renderer, &limits.width(leaf_layout.size.width));
                }
                _ => (),
            }

            node.move_to_mut(Point {
                x: leaf_layout.location.x,
                y: leaf_layout.location.y,
            })
        }
    }

    let grid_size = Size {
        width: grid_layout.content_size.width,
        height: grid_layout.content_size.height,
    };

    Node::with_children(grid_size, nodes)
}
