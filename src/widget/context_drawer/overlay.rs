// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Element;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Operation};
use iced::advanced::{Clipboard, Shell};
use iced::advanced::{overlay, renderer};
use iced::{Event, Point, Rectangle, Size, event, mouse};
use iced_core::Renderer;

pub(super) struct Overlay<'a, 'b, Message> {
    pub(crate) position: Point,
    pub(super) content: &'b mut Element<'a, Message>,
    pub(super) tree: &'b mut widget::Tree,
    pub(super) width: f32,
}

impl<Message> overlay::Overlay<Message, crate::Theme, crate::Renderer> for Overlay<'_, '_, Message>
where
    Message: Clone,
{
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> layout::Node {
        let position = self.position;
        let limits = layout::Limits::new(Size::ZERO, bounds)
            .width(self.width)
            .height(bounds.height - 8.0 - position.y);

        let node = self
            .content
            .as_widget()
            .layout(self.tree, renderer, &limits);
        let node_size = node.size();

        node.move_to(Point {
            x: if bounds.width > node_size.width - 8.0 {
                bounds.width - node_size.width - 8.0
            } else {
                0.0
            },
            y: if bounds.height > node_size.height - 8.0 {
                bounds.height - node_size.height - 8.0
            } else {
                0.0
            },
        })
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        )
    }

    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        renderer.with_layer(layout.bounds(), |renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout,
                cursor,
                &layout.bounds(),
            );
        })
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(self.tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
    ) -> Option<overlay::Element<'c, Message, crate::Theme, crate::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(self.tree, layout, renderer, iced::Vector::default())
    }
}
