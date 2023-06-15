mod subscription;

use iced::futures::channel::mpsc::UnboundedSender;
use iced::widget::Container;
pub use subscription::*;

use iced_core::alignment;
use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::Tree;
use iced_core::{Clipboard, Element, Layout, Length, Padding, Point, Rectangle, Shell, Widget};
use std::{fmt::Debug, hash::Hash};

pub use iced_style::container::{Appearance, StyleSheet};

#[derive(Clone, Debug)]
pub struct RectangleTracker<I> {
    tx: UnboundedSender<(I, Rectangle)>,
}

impl<I> RectangleTracker<I>
where
    I: Hash + Copy + Send + Sync + Debug,
{
    pub fn container<'a, Message: 'static, T>(
        &self,
        id: I,
        content: T,
    ) -> RectangleTrackingContainer<'a, Message, crate::Renderer, I>
    where
        I: 'a,
        T: Into<Element<'a, Message, crate::Renderer>>,
    {
        RectangleTrackingContainer::new(content, id, self.tx.clone())
    }
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct RectangleTrackingContainer<'a, Message, Renderer, I>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    tx: UnboundedSender<(I, Rectangle)>,
    id: I,
    container: Container<'a, Message, Renderer>,
}

impl<'a, Message, Renderer, I> RectangleTrackingContainer<'a, Message, Renderer, I>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    /// Creates an empty [`Container`].
    pub(crate) fn new<T>(content: T, id: I, tx: UnboundedSender<(I, Rectangle)>) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        RectangleTrackingContainer {
            id,
            tx,
            container: Container::new(content),
        }
    }

    /// Sets the [`Padding`] of the [`Container`].
    #[must_use]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.container = self.container.padding(padding);
        self
    }

    /// Sets the width of the [`self.`].
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`Container`].
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`Container`].
    #[must_use]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    #[must_use]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    #[must_use]
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    #[must_use]
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    #[must_use]
    pub fn center_x(mut self) -> Self {
        self.container = self.container.center_x();
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    #[must_use]
    pub fn center_y(mut self) -> Self {
        self.container = self.container.center_y();
        self
    }

    /// Sets the style of the [`Container`].
    #[must_use]
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.container = self.container.style(style);
        self
    }
}

impl<'a, Message, Renderer, I> Widget<Message, Renderer>
    for RectangleTrackingContainer<'a, Message, Renderer, I>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
    }

    fn width(&self) -> Length {
        Widget::width(&self.container)
    }

    fn height(&self) -> Length {
        Widget::height(&self.container)
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.container.layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<
            iced_core::widget::OperationOutputWrapper<Message>,
        >,
    ) {
        self.container.operate(tree, layout, renderer, operation);
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
    ) -> event::Status {
        self.container.on_event(
            tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
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
        self.container
            .mouse_interaction(tree, layout, cursor_position, viewport, renderer)
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
        let _ = self.tx.unbounded_send((self.id, layout.bounds()));

        self.container.draw(
            tree,
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
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.container.overlay(tree, layout, renderer)
    }
}

impl<'a, Message, Renderer, I> From<RectangleTrackingContainer<'a, Message, Renderer, I>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    Renderer::Theme: StyleSheet,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    fn from(
        column: RectangleTrackingContainer<'a, Message, Renderer, I>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
