// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::overlay::Overlay;
use crate::widget::{self, LayerContainer, button, column, container, icon, row, scrollable, text};
use crate::{Apply, Element, Renderer, Theme, fl};
use std::borrow::Cow;

use iced_core::Alignment;
use iced_core::event::{self, Event};
use iced_core::widget::{Operation, Tree};
use iced_core::{
    Clipboard, Layout, Length, Rectangle, Shell, Vector, Widget, layout, mouse,
    overlay as iced_overlay, renderer,
};

#[must_use]
pub struct ContextDrawer<'a, Message> {
    id: Option<iced_core::widget::Id>,
    content: Element<'a, Message>,
    drawer: Element<'a, Message>,
    on_close: Option<Message>,
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    pub fn new_inner<Drawer>(
        title: Option<Cow<'a, str>>,
        actions: Option<Element<'a, Message>>,
        header: Option<Element<'a, Message>>,
        footer: Option<Element<'a, Message>>,
        drawer: Drawer,
        on_close: Message,
        max_width: f32,
    ) -> Element<'a, Message>
    where
        Drawer: Into<Element<'a, Message>>,
    {
        #[inline(never)]
        fn inner<'a, Message: Clone + 'static>(
            title: Option<Cow<'a, str>>,
            actions_opt: Option<Element<'a, Message>>,
            header_opt: Option<Element<'a, Message>>,
            footer_opt: Option<Element<'a, Message>>,
            drawer: Element<'a, Message>,
            on_close: Message,
            max_width: f32,
        ) -> Element<'a, Message> {
            let cosmic_theme::Spacing {
                space_xxs,
                space_s,
                space_m,
                space_l,
                ..
            } = crate::theme::spacing();

            let horizontal_padding = if max_width < 392.0 { space_s } else { space_l };

            let (actions_slot, column_title) = if let Some(actions) = actions_opt {
                let actions = actions
                    .apply(container)
                    .width(Length::Fill)
                    .apply(Element::from);
                let title = title.map(|title| text::title4(title).width(Length::Fill));
                (actions, title)
            } else {
                let title = title
                    .map(|title| text::title4(title).width(Length::Fill).apply(Element::from))
                    .unwrap_or_else(|| widget::horizontal_space().apply(Element::from));
                (title, None)
            };

            let header_row = row::with_capacity(2).push(actions_slot).push(
                button::text(fl!("close"))
                    .trailing_icon(icon::from_name("go-next-symbolic"))
                    .on_press(on_close),
            );
            let header = column::with_capacity(3)
                .align_x(Alignment::Center)
                .padding([space_m, horizontal_padding])
                .spacing(space_m)
                .push(header_row)
                .push_maybe(column_title)
                .push_maybe(header_opt);
            let footer = footer_opt.map(|element| {
                container(element)
                    .align_y(Alignment::Center)
                    .padding([space_xxs, horizontal_padding])
            });
            let pane = column::with_capacity(3)
                .push(header)
                .push(
                    container(drawer)
                        .padding([
                            0,
                            horizontal_padding,
                            if footer.is_some() { 0 } else { space_l },
                            horizontal_padding,
                        ])
                        .apply(scrollable)
                        .height(Length::Fill),
                )
                .push_maybe(footer);

            // XXX new limits do not exactly handle the max width well for containers
            // XXX this is a hack to get around that
            container(
                LayerContainer::new(pane)
                    .layer(cosmic_theme::Layer::Primary)
                    .class(crate::style::Container::ContextDrawer)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .max_width(max_width),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .into()
        }

        inner(
            title,
            actions,
            header,
            footer,
            drawer.into(),
            on_close,
            max_width,
        )
    }

    /// Creates an empty [`ContextDrawer`].
    pub fn new<Content, Drawer>(
        title: Option<Cow<'a, str>>,
        actions: Option<Element<'a, Message>>,
        header: Option<Element<'a, Message>>,
        footer: Option<Element<'a, Message>>,
        content: Content,
        drawer: Drawer,
        on_close: Message,
        max_width: f32,
    ) -> Self
    where
        Content: Into<Element<'a, Message>>,
        Drawer: Into<Element<'a, Message>>,
    {
        let drawer = Self::new_inner(title, actions, header, footer, drawer, on_close, max_width);

        ContextDrawer {
            id: None,
            content: content.into(),
            drawer,
            on_close: None,
        }
    }

    /// Sets the [`Id`] of the [`ContextDrawer`].
    #[inline]
    pub fn id(mut self, id: iced_core::widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Map the message type of the context drawer to another
    #[inline]
    pub fn map<Out: Clone + 'static>(
        self,
        on_message: fn(Message) -> Out,
    ) -> ContextDrawer<'a, Out> {
        ContextDrawer {
            id: self.id,
            content: self.content.map(on_message),
            drawer: self.drawer.map(on_message),
            on_close: self.on_close.map(on_message),
        }
    }

    /// Optionally assigns message to `on_close` event.
    #[inline]
    pub fn on_close_maybe(mut self, message: Option<Message>) -> Self {
        self.on_close = message;
        self
    }
}

impl<Message: Clone> Widget<Message, crate::Theme, Renderer> for ContextDrawer<'_, Message> {
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
        operation: &mut dyn Operation<()>,
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
        translation: Vector,
    ) -> Option<iced_overlay::Element<'b, Message, crate::Theme, Renderer>> {
        let bounds = layout.bounds();

        let mut position = layout.position();
        position.x += translation.x;
        position.y += translation.y;

        Some(iced_overlay::Element::new(Box::new(Overlay {
            content: &mut self.drawer,
            tree: &mut tree.children[1],
            width: bounds.width,
            position,
        })))
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let c_state = &state.children[0];
        self.content.as_widget().a11y_nodes(layout, c_state, p)
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
