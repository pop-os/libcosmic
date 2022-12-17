use iced::Size;
use iced::widget::Container;
use iced_native::alignment;
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::{Operation, Tree};
use iced_native::{Clipboard, Element, Layout, Length, Padding, Point, Rectangle, Shell, Widget};

pub use iced_style::container::{Appearance, StyleSheet};

pub fn aspect_ratio_container<'a, Message: 'static, T>(
    content: T,
    ratio: f32
) -> AspectRatio<'a, Message, crate::Renderer>
where
    T: Into<Element<'a, Message, crate::Renderer>>,
{
    AspectRatio::new(content, ratio)
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    ratio: f32,
    container: Container<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn constrain_limits(&self, size: Size) -> Size {
        let Size { mut width, mut height } = size;
        if size.width / size.height > self.ratio {
            width = self.ratio * height;
        } else {
            height = width / self.ratio;
        }
        Size  { width, height }
    }
}

impl<'a, Message, Renderer> AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`Container`].
    pub(crate) fn new<T>(content: T, ratio: f32) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        AspectRatio {
            ratio,
            container: Container::new(content),
        }
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.container = self.container.padding(padding);
        self
    }

    /// Sets the width of the [`self.`].
    pub fn width(mut self, width: Length) -> Self {
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.container = self.container.center_x();
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.container = self.container.center_y();
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.container = self.container.style(style);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for AspectRatio<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.container.diff(tree);
    }

    fn width(&self) -> Length {
        Widget::width(&self.container)
    }

    fn height(&self) -> Length {
        Widget::height(&self.container)
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let custom_limits = layout::Limits::new(self.constrain_limits(limits.min()), self.constrain_limits(limits.max()));
        self.container.layout(renderer, &custom_limits)
    }

    fn operate(&self, tree: &mut Tree, layout: Layout<'_>, operation: &mut dyn Operation<Message>) {
        self.container.operate(tree, layout, operation)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
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
        cursor_position: Point,
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
        cursor_position: Point,
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
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.container.overlay(tree, layout, renderer)
    }
}

impl<'a, Message, Renderer> From<AspectRatio<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        column: AspectRatio<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
