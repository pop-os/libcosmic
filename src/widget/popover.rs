// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget showing a popup in an overlay positioned relative to another widget.

use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::touch;
use iced_core::widget::{tree, Operation, OperationOutputWrapper, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
};
use std::cell::RefCell;

pub use iced_style::container::{Appearance, StyleSheet};

pub fn popover<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, crate::Theme, Renderer>>,
) -> Popover<'a, Message, Renderer> {
    Popover::new(content)
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Position {
    #[default]
    Center,
    Bottom,
    Point(Point),
}

#[must_use]
pub struct Popover<'a, Message, Renderer> {
    content: Element<'a, Message, crate::Theme, Renderer>,
    modal: bool,
    // XXX Avoid refcell; improve iced overlay API?
    popup: Option<RefCell<Element<'a, Message, crate::Theme, Renderer>>>,
    position: Position,
    on_close: Option<Message>,
}

impl<'a, Message, Renderer> Popover<'a, Message, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, crate::Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            modal: false,
            popup: None,
            position: Position::Center,
            on_close: None,
        }
    }

    /// A modal popup interrupts user inputs and demands action.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Emitted when the popup is closed.
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.on_close = Some(on_close);
        self
    }

    pub fn popup(mut self, popup: impl Into<Element<'a, Message, crate::Theme, Renderer>>) -> Self {
        self.popup = Some(RefCell::new(popup.into()));
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    // TODO More options for positioning similar to GdkPopup, xdg_popup
}

impl<'a, Message: Clone, Renderer> Widget<Message, crate::Theme, Renderer>
    for Popover<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State { is_open: true })
    }

    fn children(&self) -> Vec<Tree> {
        if let Some(popup) = &self.popup {
            vec![Tree::new(&self.content), Tree::new(&*popup.borrow())]
        } else {
            vec![Tree::new(&self.content)]
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        if let Some(popup) = &mut self.popup {
            tree.diff_children(&mut [&mut self.content, &mut popup.borrow_mut()]);
        } else {
            tree.diff_children(&mut [&mut self.content]);
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
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
        if !self.modal
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(_))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            )
        {
            let state = tree.state.downcast_mut::<State>();
            let was_open = state.is_open;
            state.is_open = cursor_position.is_over(layout.bounds());

            if let Some(on_close) = self.on_close.clone() {
                if was_open && !state.is_open {
                    shell.publish(on_close);
                }
            }
        }

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
        theme: &crate::Theme,
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
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        if !tree.state.downcast_mut::<State>().is_open {
            return None;
        }

        if let Some(popup) = &self.popup {
            let bounds = layout.bounds();

            // Calculate overlay position from relative position
            let mut overlay_position = match self.position {
                Position::Center => Point::new(
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                ),
                Position::Bottom => {
                    Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height)
                }
                Position::Point(relative) => {
                    bounds.position() + Vector::new(relative.x, relative.y)
                }
            };

            // Round position to prevent rendering issues
            overlay_position.x = overlay_position.x.round();
            overlay_position.y = overlay_position.y.round();

            // XXX needed to use RefCell to get &mut for popup element
            Some(overlay::Element::new(
                overlay_position,
                Box::new(Overlay {
                    tree: &mut tree.children[1],
                    content: popup,
                    position: self.position,
                }),
            ))
        } else {
            self.content
                .as_widget_mut()
                .overlay(&mut tree.children[0], layout, renderer)
        }
    }

    fn drag_destinations(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &tree.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }
}

impl<'a, Message, Renderer> From<Popover<'a, Message, Renderer>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: 'static + Clone,
    Renderer: iced_core::Renderer + 'static,
{
    fn from(popover: Popover<'a, Message, Renderer>) -> Self {
        Self::new(popover)
    }
}

pub struct Overlay<'a, 'b, Message, Renderer> {
    tree: &'a mut Tree,
    content: &'a RefCell<Element<'b, Message, crate::Theme, Renderer>>,
    position: Position,
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, crate::Theme, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
where
    Message: Clone,
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
        let node = self
            .content
            .borrow()
            .as_widget()
            .layout(self.tree, renderer, &limits);
        match self.position {
            Position::Center => {
                // Position is set to the center of the widget
                let width = node.size().width;
                let height = node.size().height;
                position.x = (position.x - width / 2.0).clamp(0.0, bounds.width - width);
                position.y = (position.y - height / 2.0).clamp(0.0, bounds.height - height);
            }
            Position::Bottom => {
                // Position is set to the center bottom of the widget
                let width = node.size().width;
                position.x = (position.x - width / 2.0).clamp(0.0, bounds.width - width);
            }
            Position::Point(_) => {
                // Position is using context menu logic
                let size = node.size();
                position.x = position.x.clamp(0.0, bounds.width - size.width);
                if position.y + size.height > bounds.height {
                    position.y = (position.y - size.height).clamp(0.0, bounds.height - size.height);
                }
            }
        }

        // Round position to prevent rendering issues
        position.x = position.x.round();
        position.y = position.y.round();

        node.move_to(position)
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
        theme: &crate::Theme,
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

/// The local state of a [`Popover`].
#[derive(Debug, Default)]
struct State {
    is_open: bool,
}
