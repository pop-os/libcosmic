// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced_core::event::{self, Event};
use iced_core::widget::{Operation, Tree};
use iced_core::{
    layout, mouse, overlay, renderer, Alignment, Clipboard, Layout, Length, Padding, Rectangle,
    Shell, Widget,
};
use iced_renderer::core::widget::OperationOutputWrapper;

/// Responsively generates rows and columns of widgets based on its dimmensions.
#[must_use]
#[derive(Setters)]
pub struct Grid<'a, Message> {
    #[setters(skip)]
    children: Vec<Element<'a, Message>>,
    /// Where children shall be assigned in the grid.
    #[setters(skip)]
    assignments: Vec<Assignment>,
    /// Sets the padding around the widget.
    padding: Padding,
    /// Alignment across columns
    column_alignment: Alignment,
    /// Alignment across rows
    row_alignment: Alignment,
    /// Sets the space between each column of items.
    column_spacing: u16,
    /// Sets the space between each item in a row.
    row_spacing: u16,
    /// Sets the width of the grid.
    width: Length,
    /// Sets the height of the grid.
    height: Length,
    /// Sets the max width
    max_width: f32,
    #[setters(skip)]
    column: u16,
    #[setters(skip)]
    row: u16,
}

impl<'a, Message> Grid<'a, Message> {
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
            assignments: Vec::new(),
            padding: Padding::ZERO,
            column_alignment: Alignment::Start,
            row_alignment: Alignment::Start,
            column_spacing: 4,
            row_spacing: 4,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            column: 1,
            row: 1,
        }
    }

    /// Attach a new element with a given grid assignment.
    pub fn push(mut self, widget: impl Into<Element<'a, Message>>) -> Self {
        self.children.push(widget.into());

        self.assignments.push(Assignment {
            column: self.column,
            row: self.row,
            width: 1,
            height: 1,
        });

        self.column += 1;

        self
    }

    /// Attach a new element with custom properties
    pub fn push_with<W, S>(mut self, widget: W, setup: S) -> Self
    where
        W: Into<Element<'a, Message>>,
        S: Fn(Assignment) -> Assignment,
    {
        self.children.push(widget.into());

        self.assignments.push(setup(Assignment {
            column: self.column,
            row: self.row,
            width: 1,
            height: 1,
        }));

        self.column += 1;

        self
    }

    pub fn insert_row(mut self) -> Self {
        self.row += 1;
        self.column = 1;
        self
    }
}

impl<'a, Message: 'static + Clone> Widget<Message, Renderer> for Grid<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(self.children.as_mut_slice());
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .width(self.width())
            .height(self.height());

        super::layout::resolve(
            renderer,
            &limits,
            &self.children,
            &self.assignments,
            self.width,
            self.height,
            self.padding,
            self.column_alignment,
            self.row_alignment,
            f32::from(self.column_spacing),
            f32::from(self.row_spacing),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer)
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        use iced_accessibility::A11yTree;
        A11yTree::join(
            self.children
                .iter()
                .zip(layout.children())
                .zip(state.children.iter())
                .map(|((c, c_layout), state)| c.as_widget().a11y_nodes(c_layout, state, p)),
        )
    }
}

impl<'a, Message: 'static + Clone> From<Grid<'a, Message>> for Element<'a, Message> {
    fn from(flex_row: Grid<'a, Message>) -> Self {
        Self::new(flex_row)
    }
}

#[derive(Copy, Clone, Debug, Setters)]
#[must_use]
pub struct Assignment {
    pub(super) column: u16,
    pub(super) row: u16,
    pub(super) width: u16,
    pub(super) height: u16,
}

impl Assignment {
    pub const fn new() -> Self {
        Self {
            column: 0,
            row: 0,
            width: 1,
            height: 1,
        }
    }
}

impl From<(u16, u16)> for Assignment {
    fn from((column, row): (u16, u16)) -> Self {
        Self {
            column,
            row,
            width: 1,
            height: 1,
        }
    }
}

impl From<(u16, u16, u16, u16)> for Assignment {
    fn from((column, row, width, height): (u16, u16, u16, u16)) -> Self {
        Self {
            column,
            row,
            width,
            height,
        }
    }
}
