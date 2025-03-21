// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced_core::event::{self, Event};
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Layout, Length, Padding, Rectangle, Shell, Vector, Widget, layout, mouse, overlay,
    renderer,
};

/// Responsively generates rows and columns of widgets based on its dimensions.
#[derive(Setters)]
#[must_use]
pub struct FlexRow<'a, Message> {
    #[setters(skip)]
    children: Vec<Element<'a, Message>>,
    /// Sets the padding around the widget.
    #[setters(into)]
    padding: Padding,
    /// Sets the space between each column of items.
    column_spacing: u16,
    /// Sets the space between each item in a row.
    row_spacing: u16,
    /// Sets the width.
    width: Length,
    /// Sets minimum width of items that grow.
    #[setters(into)]
    min_item_width: Option<f32>,
    /// Sets the max width
    max_width: f32,
    /// Defines how content will be aligned horizontally.
    #[setters(skip)]
    align_items: Option<taffy::AlignItems>,
    /// Defines how content will be aligned vertically.
    #[setters(skip)]
    justify_items: Option<taffy::AlignItems>,
    /// Defines how the content will be justified.
    #[setters(into)]
    justify_content: Option<crate::widget::JustifyContent>,
}

impl<'a, Message> FlexRow<'a, Message> {
    pub(crate) const fn new(children: Vec<Element<'a, Message>>) -> Self {
        Self {
            children,
            padding: Padding::ZERO,
            column_spacing: 4,
            row_spacing: 4,
            width: Length::Shrink,
            min_item_width: None,
            max_width: f32::INFINITY,
            align_items: None,
            justify_items: None,
            justify_content: None,
        }
    }

    /// Defines how content will be aligned horizontally.
    pub fn align_items(mut self, alignment: iced::Alignment) -> Self {
        self.align_items = Some(match alignment {
            iced::Alignment::Center => taffy::AlignItems::Center,
            iced::Alignment::Start => taffy::AlignItems::Start,
            iced::Alignment::End => taffy::AlignItems::End,
        });
        self
    }

    /// Defines how content will be aligned vertically.
    pub fn justify_items(mut self, alignment: iced::Alignment) -> Self {
        self.justify_items = Some(match alignment {
            iced::Alignment::Center => taffy::AlignItems::Center,
            iced::Alignment::Start => taffy::AlignItems::Start,
            iced::Alignment::End => taffy::AlignItems::End,
        });
        self
    }

    /// Sets the space between each column and row.
    #[inline]
    pub const fn spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self.row_spacing = spacing;
        self
    }
}

impl<Message: 'static + Clone> Widget<Message, crate::Theme, Renderer> for FlexRow<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(self.children.as_mut_slice());
    }

    fn size(&self) -> iced_core::Size<Length> {
        iced_core::Size::new(self.width, Length::Shrink)
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
            self.padding,
            f32::from(self.column_spacing),
            f32::from(self.row_spacing),
            self.min_item_width,
            self.align_items,
            self.justify_items,
            self.justify_content,
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
                .map(|((c, c_layout), state)| c.as_widget().a11y_nodes(c_layout, state, p)),
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        for ((e, layout), state) in self
            .children
            .iter()
            .zip(layout.children())
            .zip(state.children.iter())
        {
            e.as_widget()
                .drag_destinations(state, layout, renderer, dnd_rectangles);
        }
    }
}

impl<'a, Message: 'static + Clone> From<FlexRow<'a, Message>> for Element<'a, Message> {
    fn from(flex_row: FlexRow<'a, Message>) -> Self {
        Self::new(flex_row)
    }
}
