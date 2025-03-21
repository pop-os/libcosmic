mod subscription;

use iced::Vector;
use iced::futures::channel::mpsc::UnboundedSender;
use iced::widget::Container;
pub use subscription::*;

use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::Tree;
use iced_core::{Alignment, Clipboard, Element, Layout, Length, Padding, Rectangle, Shell, Widget};
use std::{fmt::Debug, hash::Hash};

pub use iced_widget::container::{Catalog, Style};

pub fn rectangle_tracking_container<'a, Message, I, T>(
    content: T,
    id: I,
    tx: UnboundedSender<(I, Rectangle)>,
) -> RectangleTrackingContainer<'a, Message, crate::Renderer, I>
where
    I: Hash + Copy + Send + Sync + Debug + 'a,
    T: Into<Element<'a, Message, crate::Theme, crate::Renderer>>,
{
    RectangleTrackingContainer::new(content, id, tx)
}

pub fn subscription<
    I: 'static + Hash + Copy + Send + Sync + Debug,
    R: 'static + Hash + Copy + Send + Sync + Debug + Eq,
>(
    id: I,
) -> iced::Subscription<(I, RectangleUpdate<R>)> {
    subscription::rectangle_tracker_subscription(id)
}

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
        T: Into<Element<'a, Message, crate::Theme, crate::Renderer>>,
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
{
    tx: UnboundedSender<(I, Rectangle)>,
    id: I,
    container: Container<'a, Message, crate::Theme, Renderer>,
    ignore_bounds: bool,
}

impl<'a, Message, Renderer, I> RectangleTrackingContainer<'a, Message, Renderer, I>
where
    Renderer: iced_core::Renderer,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    /// Creates an empty [`Container`].
    pub(crate) fn new<T>(content: T, id: I, tx: UnboundedSender<(I, Rectangle)>) -> Self
    where
        T: Into<Element<'a, Message, crate::Theme, Renderer>>,
    {
        RectangleTrackingContainer {
            id,
            tx,
            container: Container::new(content),
            ignore_bounds: false,
        }
    }

    pub fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
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
    pub fn align_x(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    #[must_use]
    pub fn align_y(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    #[must_use]
    pub fn center_x(mut self, width: Length) -> Self {
        self.container = self.container.center_x(width);
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    #[must_use]
    pub fn center_y(mut self, height: Length) -> Self {
        self.container = self.container.center_y(height);
        self
    }

    /// Centers the contents in the horizontal and vertical axis of the [`Container`].
    #[must_use]
    pub fn center(mut self, length: Length) -> Self {
        self.container = self.container.center(length);
        self
    }

    /// Sets the style of the [`Container`].
    #[must_use]
    pub fn style(mut self, style: impl Into<<crate::Theme as Catalog>::Class<'a>>) -> Self {
        self.container = self.container.class(style);
        self
    }

    /// Set to true to ignore parent container bounds when performing layout.
    /// This can be useful for widgets that are in auto-sized surfaces.
    #[must_use]
    pub fn ignore_bounds(mut self, ignore_bounds: bool) -> Self {
        self.ignore_bounds = ignore_bounds;
        self
    }
}

impl<'a, Message, Renderer, I> Widget<Message, crate::Theme, Renderer>
    for RectangleTrackingContainer<'a, Message, Renderer, I>
where
    Renderer: iced_core::Renderer,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn state(&self) -> iced_core::widget::tree::State {
        self.container.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.container.size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.container.layout(
            tree,
            renderer,
            if self.ignore_bounds {
                &layout::Limits::NONE
            } else {
                limits
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
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
        viewport: &iced_core::Rectangle,
    ) -> event::Status {
        self.container.on_event(
            tree,
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
        self.container
            .mouse_interaction(tree, layout, cursor_position, viewport, renderer)
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
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        self.container.overlay(tree, layout, renderer, translation)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.container
            .drag_destinations(state, layout, renderer, dnd_rectangles);
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        self.container.a11y_nodes(layout, state, p)
    }
}

impl<'a, Message, Renderer, I> From<RectangleTrackingContainer<'a, Message, Renderer, I>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    I: 'a + Hash + Copy + Send + Sync + Debug,
{
    fn from(
        column: RectangleTrackingContainer<'a, Message, Renderer, I>,
    ) -> Element<'a, Message, crate::Theme, Renderer> {
        Element::new(column)
    }
}
