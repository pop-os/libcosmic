// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::event::Event;
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Vector, Widget, layout, mouse, overlay,
    renderer, touch,
};

/// Wrap `content` so that a pointer press already captured by an earlier
/// widget (such as the window header) is not delivered to it.
pub fn capture_guard<'a, Message, Theme, E>(
    content: E,
) -> CaptureGuard<'a, Message, Theme, crate::Renderer>
where
    E: Into<Element<'a, Message, Theme, crate::Renderer>>,
{
    CaptureGuard {
        content: content.into(),
    }
}

/// See [`capture_guard`].
#[allow(missing_debug_implementations)]
pub struct CaptureGuard<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
}

fn is_pointer_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for CaptureGuard<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = node.size();
        layout::Node::with_children(size, vec![node])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout
                    .children()
                    .next()
                    .unwrap()
                    .with_virtual_offset(layout.virtual_offset()),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // If the header (or any earlier sibling) already captured this press,
        // do not let the content act on it. Otherwise a title-bar / window
        // control click would defocus the focused content widget.
        if shell.is_event_captured() && is_pointer_press(event) {
            return;
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content_layout.with_virtual_offset(layout.virtual_offset()),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            content_layout.with_virtual_offset(layout.virtual_offset()),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
            renderer,
            viewport,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        let content_layout = layout.children().next().unwrap();
        self.content.as_widget().drag_destinations(
            &state.children[0],
            content_layout.with_virtual_offset(layout.virtual_offset()),
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
        let c_layout = layout.children().next().unwrap();
        let c_state = &state.children[0];
        self.content.as_widget().a11y_nodes(
            c_layout.with_virtual_offset(layout.virtual_offset()),
            c_state,
            p,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<CaptureGuard<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    Theme: 'a,
{
    fn from(
        guard: CaptureGuard<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(guard)
    }
}
