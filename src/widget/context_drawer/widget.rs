// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::{
    button, column, container, icon, row, scrollable, text, LayerContainer, Space,
};
use crate::{Apply, Element, Renderer, Theme};

use super::overlay::Overlay;

use iced_core::alignment;
use iced_core::event::{self, Event};
use iced_core::widget::{Operation, Tree};
use iced_core::{
    layout, mouse, overlay as iced_overlay, renderer, Clipboard, Layout, Length, Padding,
    Rectangle, Shell, Widget,
};

use iced_renderer::core::widget::OperationOutputWrapper;

#[must_use]
pub struct ContextDrawer<'a, Message> {
    id: Option<iced_core::widget::Id>,
    content: Element<'a, Message>,
    drawer: Element<'a, Message>,
    on_close: Option<Message>,
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    pub fn new_inner<Drawer>(
        header: &'a str,
        drawer: Drawer,
        on_close: Message,
        max_width: f32,
    ) -> Element<'a, Message>
    where
        Drawer: Into<Element<'a, Message>>,
    {
        let header = row::with_capacity(3)
            .padding(Padding {
                top: 0.0,
                bottom: 0.0,
                left: 32.0,
                right: 32.0,
            })
            .push(Space::new(Length::FillPortion(1), Length::Fixed(0.0)))
            .push(
                text::heading(header)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .push(
                button::text("Close")
                    .trailing_icon(icon::from_name("go-next-symbolic"))
                    .on_press(on_close)
                    .style(crate::theme::Button::Link)
                    .apply(container)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .align_x(alignment::Horizontal::Right)
                    .center_y(),
            )
            // XXX must be done after pushing elements or it may be overwritten by size hints from contents
            .height(Length::Fixed(80.0))
            .width(Length::Fixed(480.0));

        let pane = column::with_capacity(2)
            .push(header.height(Length::Fixed(80.)))
            .push(
                scrollable(container(drawer.into()).padding(Padding {
                    top: 0.0,
                    left: 32.0,
                    right: 32.0,
                    bottom: 32.0,
                }))
                .height(Length::Fill)
                .width(Length::Shrink),
            );

        // XXX new limits do not exactly handle the max width well for containers
        // XXX this is a hack to get around that
        container(
            LayerContainer::new(pane)
                .layer(cosmic_theme::Layer::Primary)
                .style(crate::style::Container::ContextDrawer)
                .width(Length::Fill)
                .height(Length::Fill)
                .max_width(max_width),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .into()
    }

    /// Creates an empty [`ContextDrawer`].
    pub fn new<Content, Drawer>(
        header: &'a str,
        content: Content,
        drawer: Drawer,
        on_close: Message,
        max_width: f32,
    ) -> Self
    where
        Content: Into<Element<'a, Message>>,
        Drawer: Into<Element<'a, Message>>,
    {
        let drawer = Self::new_inner(header, drawer, on_close, max_width);

        ContextDrawer {
            id: None,
            content: content.into(),
            drawer,
            on_close: None,
        }
    }

    /// Sets the [`Id`] of the [`ContextDrawer`].
    pub fn id(mut self, id: iced_core::widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    // Optionally assigns message to `on_close` event.
    pub fn on_close_maybe(mut self, message: Option<Message>) -> Self {
        self.on_close = message;
        self
    }
}

impl<'a, Message: Clone> Widget<Message, crate::Theme, Renderer> for ContextDrawer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.drawer)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.content, &mut self.drawer]);
    }

    fn size(&self) -> iced_core::Size<Length> {
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
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
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
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<iced_overlay::Element<'b, Message, crate::Theme, Renderer>> {
        let bounds = layout.bounds();

        Some(iced_overlay::Element::new(
            layout.position(),
            Box::new(Overlay {
                content: &mut self.drawer,
                tree: &mut tree.children[1],
                width: bounds.width,
            }),
        ))
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
        self.content.as_widget().a11y_nodes(c_layout, c_state, p)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &state.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }

    fn id(&self) -> Option<iced_core::widget::Id> {
        self.id.clone()
    }

    fn set_id(&mut self, id: iced_core::widget::Id) {
        self.id = Some(id);
    }
}

impl<'a, Message: 'a + Clone> From<ContextDrawer<'a, Message>> for Element<'a, Message> {
    fn from(widget: ContextDrawer<'a, Message>) -> Element<'a, Message> {
        Element::new(widget)
    }
}
