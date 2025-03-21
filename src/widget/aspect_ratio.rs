//! A container which constraints itself to a specific aspect ratio.

use iced::Size;
use iced::widget::Container;
use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::Tree;
use iced_core::{
    Alignment, Clipboard, Element, Layout, Length, Padding, Rectangle, Shell, Vector, Widget,
};

pub use iced_widget::container::{Catalog, Style};

pub fn aspect_ratio_container<'a, Message: 'static, T>(
    content: T,
    ratio: f32,
) -> AspectRatio<'a, Message, crate::Renderer>
where
    T: Into<Element<'a, Message, crate::Theme, crate::Renderer>>,
{
    AspectRatio::new(content, ratio)
}

/// A container which constraints itself to a specific aspect ratio.
#[allow(missing_debug_implementations)]
pub struct AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    ratio: f32,
    container: Container<'a, Message, crate::Theme, Renderer>,
}

impl<Message, Renderer> AspectRatio<'_, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn constrain_limits(&self, size: Size) -> Size {
        let Size {
            mut width,
            mut height,
        } = size;
        if size.width / size.height > self.ratio {
            width = self.ratio * height;
        } else {
            height = width / self.ratio;
        }
        Size { width, height }
    }
}

impl<'a, Message, Renderer> AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// Creates an empty [`Container`].
    pub(crate) fn new<T>(content: T, ratio: f32) -> Self
    where
        T: Into<Element<'a, Message, crate::Theme, Renderer>>,
    {
        AspectRatio {
            ratio,
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
    #[inline]
    pub fn width(mut self, width: Length) -> Self {
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`Container`].
    #[must_use]
    #[inline]
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`Container`].
    #[must_use]
    #[inline]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    #[must_use]
    #[inline]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    #[must_use]
    #[inline]
    pub fn align_x(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    #[must_use]
    #[inline]
    pub fn align_y(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    #[must_use]
    #[inline]
    pub fn center_x(mut self, width: Length) -> Self {
        self.container = self.container.center_x(width);
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    #[must_use]
    #[inline]
    pub fn center_y(mut self, height: Length) -> Self {
        self.container = self.container.center_y(height);
        self
    }

    /// Centers the contents in the horizontal and vertical axis of the [`Container`].
    #[must_use]
    #[inline]
    pub fn center(mut self, length: Length) -> Self {
        self.container = self.container.center(length);
        self
    }

    /// Sets the style of the [`Container`].
    #[must_use]
    pub fn class(mut self, style: impl Into<crate::style::Container<'a>>) -> Self {
        self.container = self.container.class(style);
        self
    }
}

impl<Message, Renderer> Widget<Message, crate::Theme, Renderer>
    for AspectRatio<'_, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.container.size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let custom_limits = layout::Limits::new(
            self.constrain_limits(limits.min()),
            self.constrain_limits(limits.max()),
        );
        self.container
            .layout(&mut tree.children[0], renderer, &custom_limits)
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
        viewport: &Rectangle,
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

impl<'a, Message, Renderer> From<AspectRatio<'a, Message, Renderer>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
{
    fn from(
        column: AspectRatio<'a, Message, Renderer>,
    ) -> Element<'a, Message, crate::Theme, Renderer> {
        Element::new(column)
    }
}
