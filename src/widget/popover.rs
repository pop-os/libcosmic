// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A container which displays an overlay when a popup widget is attached.

use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::touch;
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
};

pub use iced_widget::container::{Catalog, Style};

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

/// A container which displays overlays when a popup widget is assigned.
#[must_use]
pub struct Popover<'a, Message, Renderer> {
    content: Element<'a, Message, crate::Theme, Renderer>,
    modal: bool,
    popup: Option<Element<'a, Message, crate::Theme, Renderer>>,
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

    /// A modal popup intercepts user inputs while a popup is active.
    #[inline]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Emitted when the popup is closed.
    #[inline]
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.on_close = Some(on_close);
        self
    }

    #[inline]
    pub fn popup(mut self, popup: impl Into<Element<'a, Message, crate::Theme, Renderer>>) -> Self {
        self.popup = Some(popup.into());
        self
    }

    #[inline]
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}

impl<Message: Clone, Renderer> Widget<Message, crate::Theme, Renderer>
    for Popover<'_, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        if let Some(popup) = &self.popup {
            vec![Tree::new(&self.content), Tree::new(popup)]
        } else {
            vec![Tree::new(&self.content)]
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        if let Some(popup) = &mut self.popup {
            tree.diff_children(&mut [&mut self.content, popup]);
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
        let tree = content_tree_mut(tree);
        self.content.as_widget().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        self.content
            .as_widget()
            .operate(content_tree_mut(tree), layout, renderer, operation);
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
        if self.popup.is_some() {
            if self.modal {
                if matches!(event, Event::Mouse(_) | Event::Touch(_)) {
                    return event::Status::Captured;
                }
            } else if let Some(on_close) = self.on_close.as_ref() {
                if matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(_))
                        | Event::Touch(touch::Event::FingerPressed { .. })
                ) && !cursor_position.is_over(layout.bounds())
                {
                    shell.publish(on_close.clone());
                }
            }
        }

        self.content.as_widget_mut().on_event(
            content_tree_mut(tree),
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
        if self.modal && self.popup.is_some() && cursor_position.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }
        self.content.as_widget().mouse_interaction(
            content_tree(tree),
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
            content_tree(tree),
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
        mut translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        if let Some(popup) = &mut self.popup {
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
            translation.x += overlay_position.x;
            translation.y += overlay_position.y;

            Some(overlay::Element::new(Box::new(Overlay {
                tree: &mut tree.children[1],
                content: popup,
                position: self.position,
                pos: Point::new(translation.x, translation.y),
                modal: self.modal,
            })))
        } else {
            self.content.as_widget_mut().overlay(
                content_tree_mut(tree),
                layout,
                renderer,
                translation,
            )
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
            content_tree(tree),
            layout,
            renderer,
            dnd_rectangles,
        );
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        self.content
            .as_widget()
            .a11y_nodes(layout, content_tree(state), p)
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
    content: &'a mut Element<'b, Message, crate::Theme, Renderer>,
    position: Position,
    pos: Point,
    modal: bool,
}

impl<Message, Renderer> overlay::Overlay<Message, crate::Theme, Renderer>
    for Overlay<'_, '_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let mut position = self.pos;
        let limits = layout::Limits::new(Size::UNIT, bounds);
        let node = self
            .content
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
                let height = node.size().height;
                position.x = (position.x - width / 2.0).clamp(0.0, bounds.width - width);
                position.y = position.y.clamp(0.0, bounds.height - height);
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
        operation: &mut dyn Operation<()>,
    ) {
        self.content
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
        if self.modal
            && matches!(event, Event::Mouse(_) | Event::Touch(_))
            && !cursor_position.is_over(layout.bounds())
        {
            return event::Status::Captured;
        }

        self.content.as_widget_mut().on_event(
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
        if self.modal && !cursor_position.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.content.as_widget().mouse_interaction(
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
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            &bounds,
        );
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, crate::Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(self.tree, layout, renderer, Default::default())
    }
}

/// The local state of a [`Popover`].
#[derive(Debug, Default)]
struct State {
    is_open: bool,
}

/// The first child in [`Popover::children`] is always the wrapped content.
fn content_tree(tree: &Tree) -> &Tree {
    &tree.children[0]
}

/// The first child in [`Popover::children`] is always the wrapped content.
fn content_tree_mut(tree: &mut Tree) -> &mut Tree {
    &mut tree.children[0]
}
