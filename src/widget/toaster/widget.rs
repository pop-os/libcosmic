// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MIT

use iced::{Limits, Size};
use iced_core::layout::Node;

use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer::{self};
use iced_core::widget::tree::Tree;
use iced_core::widget::Operation;
use iced_core::Element;
use iced_core::Overlay;
use iced_core::{Clipboard, Layout, Length, Point, Rectangle, Shell, Vector, Widget};
use iced_renderer::core::widget::OperationOutputWrapper;

pub struct Toaster<'a, Message, Theme, Renderer> {
    toasts: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    is_empty: bool,
}

impl<'a, Message, Theme, Renderer> Toaster<'a, Message, Theme, Renderer> {
    pub fn new(
        toasts: Element<'a, Message, Theme, Renderer>,
        content: Element<'a, Message, Theme, Renderer>,
        is_empty: bool,
    ) -> Self {
        Self {
            toasts,
            content,
            is_empty,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toaster<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.toasts)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.content, &mut self.toasts]);
    }

    fn operate<'b>(
        &'b self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        self.content
            .as_widget()
            .operate(&mut state.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if self.is_empty {
            None
        } else {
            let bounds = layout.bounds();

            Some(overlay::Element::new(
                bounds.position(),
                Box::new(ToasterOverlay::new(
                    &mut state.children[1],
                    &mut self.toasts,
                )),
            ))
        }
    }
}

struct ToasterOverlay<'a, 'b, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    state: &'b mut Tree,
    element: &'b mut Element<'a, Message, Theme, Renderer>,
}

impl<'a, 'b, Message, Theme, Renderer> ToasterOverlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn new(state: &'b mut Tree, element: &'b mut Element<'a, Message, Theme, Renderer>) -> Self {
        Self { state, element }
    }
}

impl<'a, 'b, Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for ToasterOverlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
        _translation: Vector,
    ) -> Node {
        let limits = Limits::new(Size::ZERO, bounds);

        let mut node = self
            .element
            .as_widget()
            .layout(self.state, renderer, &limits);

        let offset = 15.;

        let position = Point::new(
            (bounds.width / 2.) - (node.size().width / 2.),
            bounds.height - (node.size().height + offset),
        );

        node.move_to_mut(position);

        node
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.element
            .as_widget()
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        self.element.as_widget_mut().on_event(
            self.state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element
            .as_widget()
            .mouse_interaction(self.state, layout, cursor, viewport, renderer)
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.element
            .as_widget_mut()
            .overlay(self.state, layout, renderer)
    }
}

impl<'a, Message, Theme, Renderer> From<Toaster<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: 'a,
    Message: 'a,
{
    fn from(
        toaster: Toaster<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(toaster)
    }
}
