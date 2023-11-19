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
    layout, mouse, overlay as iced_overlay, renderer, Clipboard, Color, Layout, Length, Padding,
    Rectangle, Shell, Widget,
};

use iced_renderer::core::widget::OperationOutputWrapper;

#[must_use]
pub struct ContextDrawer<'a, Message> {
    content: Element<'a, Message>,
    drawer: Element<'a, Message>,
    on_close: Option<Message>,
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    /// Creates an empty [`ContextDrawer`].
    pub fn new<Content, Drawer>(
        header: &'a str,
        content: Content,
        drawer: Drawer,
        on_close: Message,
    ) -> Self
    where
        Content: Into<Element<'a, Message>>,
        Drawer: Into<Element<'a, Message>>,
    {
        let header = row::with_capacity(3)
            .height(Length::Fixed(80.0))
            .width(Length::Fixed(480.0))
            .padding(Padding {
                top: 0.0,
                bottom: 0.0,
                left: 32.0,
                right: 32.0,
            })
            .push(Space::new(Length::FillPortion(1), Length::Shrink))
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
                    .apply(container)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .align_x(alignment::Horizontal::Right)
                    .center_y(),
            );

        let pane = column::with_capacity(2).push(header).push(scrollable(
            container(drawer.into()).padding(Padding {
                top: 0.0,
                left: 32.0,
                right: 32.0,
                bottom: 32.0,
            }),
        ));

        ContextDrawer {
            content: content.into(),
            drawer: LayerContainer::new(pane)
                .style(crate::style::Container::custom(move |theme| {
                    let palette = theme.cosmic();

                    container::Appearance {
                        icon_color: Some(Color::from(palette.primary.on)),
                        text_color: Some(Color::from(palette.primary.on)),
                        background: Some(iced::Background::Color(palette.primary.base.into())),
                        border_radius: 8.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    }
                }))
                .layer(cosmic_theme::Layer::Primary)
                .width(Length::Fill)
                .height(Length::Fill)
                .max_width(480.0)
                .into(),
            on_close: None,
        }
    }

    // Optionally assigns message to `on_close` event.
    pub fn on_close_maybe(mut self, message: Option<Message>) -> Self {
        self.on_close = message;
        self
    }
}

impl<'a, Message: Clone> Widget<Message, Renderer> for ContextDrawer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.drawer)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut [&mut self.content, &mut self.drawer]);
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(renderer, limits)
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
    ) -> Option<iced_overlay::Element<'b, Message, Renderer>> {
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
}

impl<'a, Message: 'a + Clone> From<ContextDrawer<'a, Message>> for Element<'a, Message> {
    fn from(widget: ContextDrawer<'a, Message>) -> Element<'a, Message> {
        Element::new(widget)
    }
}
