// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget that slides its content in and out of view using a supplied progress value.

use crate::widget::Id;
use crate::{Element, Renderer, Theme};
use iced_core::renderer::{self, Renderer as _};
use iced_core::widget::{self, Tree};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Size, Vector, Widget, layout, mouse, overlay,
};

/// A widget that slides its content in and out of view.
///
/// The `progress` value is provided by the application, so the reserved space
/// and the content's position always stay in sync with other animated values
/// derived from the same state.
#[must_use]
pub struct Slide<'a, Message> {
    content: Element<'a, Message>,
    progress: f32,
}

impl<'a, Message> Slide<'a, Message> {
    /// Creates a horizontal [`Slide`] widget.
    pub(crate) fn new(content: impl Into<Element<'a, Message>>, progress: f32) -> Self {
        Self {
            content: content.into(),
            progress,
        }
    }

    fn child_layout<'b>(layout: Layout<'b>) -> Layout<'b> {
        match layout.children().next() {
            Some(child) => child,
            None => layout,
        }
    }

    fn visible_cursor(layout: Layout<'_>, cursor: mouse::Cursor) -> mouse::Cursor {
        match cursor.position() {
            Some(position) if layout.bounds().contains(position) => cursor,
            _ => mouse::Cursor::Unavailable,
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Slide<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.content.as_widget_mut().diff(tree);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let progress = self.progress.clamp(0.0, 1.0);
        let child = self.content.as_widget_mut().layout(tree, renderer, limits);
        let full_width = child.size().width;

        // Keep the content's trailing edge in place while the slot grows, so
        // it slides into view from its own side. The draw layer clips the rest.
        let child = child.translate(Vector::new(-full_width * (1.0 - progress), 0.0));

        layout::Node::with_children(
            Size::new(full_width * progress, child.size().height),
            vec![child],
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.progress <= 0.0 {
            return;
        }

        self.content.as_widget_mut().update(
            tree,
            event,
            Self::child_layout(layout),
            Self::visible_cursor(layout, cursor),
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        if self.progress <= 0.0 {
            return;
        }

        self.content
            .as_widget_mut()
            .operate(tree, Self::child_layout(layout), renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.progress <= 0.0 {
            return mouse::Interaction::default();
        }

        self.content.as_widget().mouse_interaction(
            tree,
            Self::child_layout(layout),
            Self::visible_cursor(layout, cursor),
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if self.progress <= 0.0 {
            return;
        }

        renderer.with_layer(layout.bounds(), |renderer| {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                style,
                Self::child_layout(layout),
                Self::visible_cursor(layout, cursor),
                viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // Popovers are drawn outside the clipped layer, so only expose them
        // once the panel has reached its final position.
        if self.progress < 1.0 {
            return None;
        }

        self.content.as_widget_mut().overlay(
            tree,
            Self::child_layout(layout),
            renderer,
            viewport,
            translation,
        )
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        cursor: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        if self.progress <= 0.0 {
            return iced_accessibility::A11yTree::default();
        }

        self.content.as_widget().a11y_nodes(
            Self::child_layout(layout),
            state,
            Self::visible_cursor(layout, cursor),
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        if self.progress <= 0.0 {
            return;
        }

        self.content.as_widget().drag_destinations(
            state,
            Self::child_layout(layout),
            renderer,
            dnd_rectangles,
        );
    }

    fn id(&self) -> Option<Id> {
        self.content.as_widget().id()
    }

    fn set_id(&mut self, id: Id) {
        self.content.as_widget_mut().set_id(id);
    }
}

impl<'a, Message: 'a + Clone> From<Slide<'a, Message>> for Element<'a, Message> {
    fn from(slide: Slide<'a, Message>) -> Self {
        Element::new(slide)
    }
}
