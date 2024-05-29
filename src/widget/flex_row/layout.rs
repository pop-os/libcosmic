// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced_core::layout::{Limits, Node};
use iced_core::widget::Tree;
use iced_core::{Length, Padding, Point, Size};
use taffy::geometry::Rect;
use taffy::style::{AlignItems, Dimension, Display, Style};
use taffy::style_helpers::length;
use taffy::TaffyTree;

#[allow(clippy::too_many_lines)]
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
    let max_size = limits.max();

    let mut leafs = Vec::with_capacity(items.len());
    let mut nodes = Vec::with_capacity(items.len());

    let mut taffy_tree = TaffyTree::<()>::with_capacity(items.len() + 1);

    let style = taffy::Style {
        display: Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        flex_wrap: taffy::FlexWrap::Wrap,

        gap: taffy::geometry::Size {
            width: length(column_spacing),
            height: length(row_spacing),
        },

        min_size: taffy::geometry::Size {
            width: length(max_size.width),
            height: Dimension::Auto,
        },

        padding: Rect {
            left: length(padding.left),
            right: length(padding.right),
            top: length(padding.top),
            bottom: length(padding.bottom),
        },

        ..taffy::Style::default()
    };

    for (child, tree) in items.iter().zip(tree.iter_mut()) {
        // Calculate the dimensions of the item.
        let child_widget = child.as_widget();
        let child_node = child_widget.layout(tree, renderer, &limits);
        let size = child_node.size();

        nodes.push(child_node);

        let c_size = child_widget.size();
        let (width, justify_self) = match c_size.width {
            Length::Fill | Length::FillPortion(_) => (Dimension::Auto, Some(AlignItems::Stretch)),
            _ => (length(size.width), None),
        };

        let child_style = Style {
            size: taffy::geometry::Size {
                width,
                height: match c_size.height {
                    Length::Fill | Length::FillPortion(_) => Dimension::Auto,
                    _ => length(size.height),
                },
            },
            justify_self,
            ..Style::default()
        };

        leafs.push(match taffy_tree.new_leaf(child_style) {
            Ok(leaf) => leaf,
            Err(why) => {
                tracing::error!(?why, "failed to add child element to flex row");
                continue;
            }
        });
    }

    let root = match taffy_tree.new_with_children(style, &leafs) {
        Ok(root) => root,
        Err(why) => {
            tracing::error!(?why, "flex row style is invalid");
            return Node::new(Size::ZERO);
        }
    };

    if let Err(why) = taffy_tree.compute_layout(
        root,
        taffy::geometry::Size {
            width: length(max_size.width),
            height: length(max_size.height),
        },
    ) {
        tracing::error!(?why, "flex row layout invalid");
        return Node::new(Size::ZERO);
    }

    let flex_layout = match taffy_tree.layout(root) {
        Ok(layout) => layout,
        Err(why) => {
            tracing::error!(?why, "cannot get flex row layout");
            return Node::new(Size::ZERO);
        }
    };

    leafs
        .into_iter()
        .zip(items.iter())
        .zip(nodes.iter_mut())
        .zip(tree)
        .for_each(|(((leaf, child), node), tree)| {
            let Ok(leaf_layout) = taffy_tree.layout(leaf) else {
                return;
            };

            let child_widget = child.as_widget();
            let c_size = child_widget.size();
            match c_size.width {
                Length::Fill | Length::FillPortion(_) => {
                    *node =
                        child_widget.layout(tree, renderer, &limits.width(leaf_layout.size.width));
                }
                _ => (),
            }

            *node = node.clone().move_to(Point {
                x: leaf_layout.location.x,
                y: leaf_layout.location.y,
            });
        });

    let size = Size {
        width: flex_layout.content_size.width,
        height: flex_layout.content_size.height,
    };

    Node::with_children(size.expand(padding), nodes)
}
