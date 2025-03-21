// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced_core::event::{self, Event};
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Alignment, Clipboard, Layout, Length, Padding, Rectangle, Shell, Vector, Widget, layout, mouse,
    overlay, renderer,
};

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
    /// Defines how the content will be justified.
    #[setters(into, strip_option)]
    justify_content: Option<crate::widget::JustifyContent>,
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

impl<Message> Default for Grid<'_, Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> Grid<'a, Message> {
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
            assignments: Vec::new(),
            padding: Padding::ZERO,
            column_alignment: Alignment::Start,
            row_alignment: Alignment::Start,
            justify_content: None,
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

    #[inline]
    pub fn insert_row(mut self) -> Self {
        self.row += 1;
        self.column = 1;
        self
    }
}

impl<Message: 'static + Clone> Widget<Message, crate::Theme, Renderer> for Grid<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(self.children.as_mut_slice());
    }

    fn size(&self) -> iced_core::Size<Length> {
        iced_core::Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.size();
        let limits = limits
            .max_width(self.max_width)
            .width(size.width)
            .height(size.height);

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
            self.justify_content,
            f32::from(self.column_spacing),
            f32::from(self.row_spacing),
            &mut tree.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), c_layout)| {
                    child.as_widget().operate(
                        state,
                        c_layout.with_virtual_offset(layout.virtual_offset()),
                        renderer,
                        operation,
                    );
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
            .map(|((child, state), c_layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    c_layout.with_virtual_offset(layout.virtual_offset()),
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
            .map(|((child, state), c_layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    c_layout.with_virtual_offset(layout.virtual_offset()),
                    cursor,
                    viewport,
                    renderer,
                )
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
        for ((child, state), c_layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                c_layout.with_virtual_offset(layout.virtual_offset()),
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, translation)
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
                .map(|((c, c_layout), state)| {
                    c.as_widget().a11y_nodes(
                        c_layout.with_virtual_offset(layout.virtual_offset()),
                        state,
                        p,
                    )
                }),
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        for ((e, c_layout), state) in self
            .children
            .iter()
            .zip(layout.children())
            .zip(state.children.iter())
        {
            e.as_widget().drag_destinations(
                state,
                c_layout.with_virtual_offset(layout.virtual_offset()),
                renderer,
                dnd_rectangles,
            );
        }
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

impl Default for Assignment {
    fn default() -> Self {
        Self::new()
    }
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
