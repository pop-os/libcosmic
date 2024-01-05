// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget showing a popup in an overlay positioned relative to another widget.

use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::{Operation, OperationOutputWrapper, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
};
use std::cell::RefCell;

pub use iced_style::container::{Appearance, StyleSheet};

pub fn popover<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    popup: impl Into<Element<'a, Message, Renderer>>,
) -> Popover<'a, Message, Renderer> {
    Popover::new(content, popup)
}

pub struct Popover<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    // XXX Avoid refcell; improve iced overlay API?
    popup: RefCell<Element<'a, Message, Renderer>>,
    position: Option<Point>,
    show_popup: bool,
}

impl<'a, Message, Renderer> Popover<'a, Message, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        popup: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            popup: RefCell::new(popup.into()),
            position: None,
            show_popup: true,
        }
    }

    pub fn position(mut self, position: Point) -> Self {
        self.position = Some(position);
        self
    }

    pub fn show_popup(mut self, show_popup: bool) -> Self {
        self.show_popup = show_popup;
        self
    }

    // TODO More options for positioning similar to GdkPopup, xdg_popup
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Popover<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&*self.popup.borrow())]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.content, &mut self.popup.borrow_mut()]);
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tree = &mut tree.children[0];
        self.content.as_widget().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        if !self.show_popup {
            return None;
        }

        let bounds = layout.bounds();
        let (position, centered) = match self.position {
            Some(relative) => (
                bounds.position() + Vector::new(relative.x, relative.y),
                false,
            ),
            None => {
                // Set position to center of bottom edge
                (
                    Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height),
                    true,
                )
            }
        };

        // XXX needed to use RefCell to get &mut for popup element
        Some(overlay::Element::new(
            position,
            Box::new(Overlay {
                tree: &mut tree.children[1],
                content: &self.popup,
                centered,
            }),
        ))
    }
}

impl<'a, Message, Renderer> From<Popover<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'static,
    Renderer: iced_core::Renderer + 'static,
    Renderer::Theme: StyleSheet,
{
    fn from(popover: Popover<'a, Message, Renderer>) -> Self {
        Self::new(popover)
    }
}

pub struct Overlay<'a, 'b, Message, Renderer> {
    tree: &'a mut Tree,
    content: &'a RefCell<Element<'b, Message, Renderer>>,
    centered: bool,
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
        mut position: Point,
        _translation: iced::Vector,
    ) -> layout::Node {
        let limits = layout::Limits::new(Size::UNIT, bounds);
        let mut node = self
            .content
            .borrow()
            .as_widget()
            .layout(self.tree, renderer, &limits);
        if self.centered {
            // Position is set to the center bottom of the lower widget
            let width = node.size().width;
            position.x = (position.x - width / 2.0).clamp(0.0, bounds.width - width);
        }
        node.move_to(position);

        node
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        self.content
            .borrow()
            .as_widget()
            .operate(self.tree, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.borrow_mut().as_widget_mut().on_event(
            self.tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.borrow().as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.content.borrow().as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &bounds,
        );
    }
}
