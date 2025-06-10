// From iced_aw, license MIT

use iced_core::{Widget, widget::Tree};
use iced_widget::core::{
    Alignment, Element, Padding, Point, Size,
    layout::{Limits, Node},
    renderer,
};

use crate::widget::RcElementWrapper;

/// The main axis of a flex layout.
#[derive(Debug)]
pub enum Axis {
    /// The horizontal axis
    Horizontal,

    /// The vertical axis
    #[allow(dead_code)]
    Vertical,
}

impl Axis {
    /// Gets the main Axis
    fn main(&self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    /// Gets the cross Axis
    fn cross(&self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    /// Returns a Packed axis
    fn pack(&self, main: f32, cross: f32) -> (f32, f32) {
        match self {
            Self::Horizontal => (main, cross),
            Self::Vertical => (cross, main),
        }
    }
}

/// Computes the flex layout with the given axis and limits, applying spacing,
/// padding and alignment to the items as needed.
///
/// It returns a new layout [`Node`].
pub fn resolve<'a, E, Message, Renderer>(
    axis: &Axis,
    renderer: &Renderer,
    limits: &Limits,
    padding: Padding,
    spacing: f32,
    align_items: Alignment,
    items: &[E],
    tree: &mut [&mut Tree],
) -> Node
where
    E: std::borrow::Borrow<Element<'a, Message, crate::Theme, Renderer>>,
    Renderer: renderer::Renderer,
{
    let limits = limits.shrink(padding);
    let total_spacing = spacing * items.len().saturating_sub(1) as f32;
    let max_cross = axis.cross(limits.max());

    let mut fill_sum = 0;
    let mut cross = axis.cross(limits.min()).max(axis.cross(Size::INFINITY));
    let mut available = axis.main(limits.max()) - total_spacing;

    let mut nodes: Vec<Node> = Vec::with_capacity(items.len());
    nodes.resize(items.len(), Node::default());

    if align_items == Alignment::Center {
        let mut fill_cross = axis.cross(limits.min());

        for (child, tree) in items.iter().zip(tree.iter_mut()) {
            let child = child.borrow();
            let c_size = child.as_widget().size();
            let cross_fill_factor = match axis {
                Axis::Horizontal => c_size.height,
                Axis::Vertical => c_size.width,
            }
            .fill_factor();

            if cross_fill_factor == 0 {
                let (max_width, max_height) = axis.pack(available, max_cross);

                let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));

                let layout = child.as_widget().layout(tree, renderer, &child_limits);
                let size = layout.size();

                fill_cross = fill_cross.max(axis.cross(size));
            }
        }

        cross = fill_cross;
    }

    for (i, (child, tree)) in items.iter().zip(tree.iter_mut()).enumerate() {
        let child = child.borrow();
        let c_size = child.as_widget().size();
        let fill_factor = match axis {
            Axis::Horizontal => c_size.width,
            Axis::Vertical => c_size.height,
        }
        .fill_factor();

        if fill_factor == 0 {
            let (min_width, min_height) = if align_items == Alignment::Center {
                axis.pack(0.0, cross)
            } else {
                axis.pack(0.0, 0.0)
            };

            let (max_width, max_height) = if align_items == Alignment::Center {
                axis.pack(available, cross)
            } else {
                axis.pack(available, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.as_widget().layout(tree, renderer, &child_limits);
            let size = layout.size();

            available -= axis.main(size);

            if align_items != Alignment::Center {
                cross = cross.max(axis.cross(size));
            }

            nodes[i] = layout;
        } else {
            fill_sum += fill_factor;
        }
    }

    let remaining = available.max(0.0);

    for (i, (child, tree)) in items.iter().zip(tree.iter_mut()).enumerate() {
        let child = child.borrow();
        let c_size = child.as_widget().size();
        let fill_factor = match axis {
            Axis::Horizontal => c_size.width,
            Axis::Vertical => c_size.height,
        }
        .fill_factor();

        if fill_factor != 0 {
            let max_main = remaining * f32::from(fill_factor) / f32::from(fill_sum);
            let min_main = if max_main.is_infinite() {
                0.0
            } else {
                max_main
            };

            let (min_width, min_height) = if align_items == Alignment::Center {
                axis.pack(min_main, cross)
            } else {
                axis.pack(min_main, axis.cross(limits.min()))
            };

            let (max_width, max_height) = if align_items == Alignment::Center {
                axis.pack(max_main, cross)
            } else {
                axis.pack(max_main, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.as_widget().layout(tree, renderer, &child_limits);

            if align_items != Alignment::Center {
                cross = cross.max(axis.cross(layout.size()));
            }

            nodes[i] = layout;
        }
    }

    let pad = axis.pack(padding.left, padding.top);
    let mut main = pad.0;

    for (i, node) in nodes.iter_mut().enumerate() {
        if i > 0 {
            main += spacing;
        }

        let (x, y) = axis.pack(main, pad.1);

        let node_ = node.clone().move_to(Point::new(x, y));

        let node_ = match axis {
            Axis::Horizontal => node_.align(Alignment::Start, align_items, Size::new(0.0, cross)),
            Axis::Vertical => node_.align(align_items, Alignment::Start, Size::new(cross, 0.0)),
        };

        let size = node_.bounds().size();

        *node = node_;

        main += axis.main(size);
    }

    let (width, height) = axis.pack(main - pad.0, cross);
    let size = limits.resolve(width, height, Size::new(width, height));

    Node::with_children(size.expand(padding), nodes)
}

/// Computes the flex layout with the given axis and limits, applying spacing,
/// padding and alignment to the items as needed.
///
/// It returns a new layout [`Node`].
pub fn resolve_wrapper<'a, Message>(
    axis: &Axis,
    renderer: &crate::Renderer,
    limits: &Limits,
    padding: Padding,
    spacing: f32,
    align_items: Alignment,
    items: &[&RcElementWrapper<Message>],
    tree: &mut [&mut Tree],
) -> Node {
    let limits = limits.shrink(padding);
    let total_spacing = spacing * items.len().saturating_sub(1) as f32;
    let max_cross = axis.cross(limits.max());

    let mut fill_sum = 0;
    let mut cross = axis.cross(limits.min()).max(axis.cross(Size::INFINITY));
    let mut available = axis.main(limits.max()) - total_spacing;

    let mut nodes: Vec<Node> = Vec::with_capacity(items.len());
    nodes.resize(items.len(), Node::default());

    if align_items == Alignment::Center {
        let mut fill_cross = axis.cross(limits.min());

        for (child, tree) in items.iter().zip(tree.iter_mut()) {
            let c_size = child.size();
            let cross_fill_factor = match axis {
                Axis::Horizontal => c_size.height,
                Axis::Vertical => c_size.width,
            }
            .fill_factor();

            if cross_fill_factor == 0 {
                let (max_width, max_height) = axis.pack(available, max_cross);

                let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));

                let layout = child.layout(tree, renderer, &child_limits);
                let size = layout.size();

                fill_cross = fill_cross.max(axis.cross(size));
            }
        }

        cross = fill_cross;
    }

    for (i, (child, tree)) in items.iter().zip(tree.iter_mut()).enumerate() {
        let c_size = child.size();
        let fill_factor = match axis {
            Axis::Horizontal => c_size.width,
            Axis::Vertical => c_size.height,
        }
        .fill_factor();

        if fill_factor == 0 {
            let (min_width, min_height) = if align_items == Alignment::Center {
                axis.pack(0.0, cross)
            } else {
                axis.pack(0.0, 0.0)
            };

            let (max_width, max_height) = if align_items == Alignment::Center {
                axis.pack(available, cross)
            } else {
                axis.pack(available, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.layout(tree, renderer, &child_limits);
            let size = layout.size();

            available -= axis.main(size);

            if align_items != Alignment::Center {
                cross = cross.max(axis.cross(size));
            }

            nodes[i] = layout;
        } else {
            fill_sum += fill_factor;
        }
    }

    let remaining = available.max(0.0);

    for (i, (child, tree)) in items.iter().zip(tree.iter_mut()).enumerate() {
        let c_size = child.size();
        let fill_factor = match axis {
            Axis::Horizontal => c_size.width,
            Axis::Vertical => c_size.height,
        }
        .fill_factor();

        if fill_factor != 0 {
            let max_main = remaining * f32::from(fill_factor) / f32::from(fill_sum);
            let min_main = if max_main.is_infinite() {
                0.0
            } else {
                max_main
            };

            let (min_width, min_height) = if align_items == Alignment::Center {
                axis.pack(min_main, cross)
            } else {
                axis.pack(min_main, axis.cross(limits.min()))
            };

            let (max_width, max_height) = if align_items == Alignment::Center {
                axis.pack(max_main, cross)
            } else {
                axis.pack(max_main, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.layout(tree, renderer, &child_limits);

            if align_items != Alignment::Center {
                cross = cross.max(axis.cross(layout.size()));
            }

            nodes[i] = layout;
        }
    }

    let pad = axis.pack(padding.left, padding.top);
    let mut main = pad.0;

    for (i, node) in nodes.iter_mut().enumerate() {
        if i > 0 {
            main += spacing;
        }

        let (x, y) = axis.pack(main, pad.1);

        let node_ = node.clone().move_to(Point::new(x, y));

        let node_ = match axis {
            Axis::Horizontal => node_.align(Alignment::Start, align_items, Size::new(0.0, cross)),
            Axis::Vertical => node_.align(align_items, Alignment::Start, Size::new(cross, 0.0)),
        };

        let size = node_.bounds().size();

        *node = node_;

        main += axis.main(size);
    }

    let (width, height) = axis.pack(main - pad.0, cross);
    let size = limits.resolve(width, height, Size::new(width, height));

    Node::with_children(size.expand(padding), nodes)
}
