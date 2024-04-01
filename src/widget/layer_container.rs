use cosmic_theme::LayeredTheme;
use iced::widget::Container;
use iced_core::alignment;
use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::Tree;
use iced_core::{Clipboard, Element, Layout, Length, Padding, Rectangle, Shell, Widget};
pub use iced_style::container::{Appearance, StyleSheet};

pub fn layer_container<'a, Message: 'static, Theme, E>(
    content: E,
) -> LayerContainer<'a, Message, Theme, crate::Renderer>
where
    E: Into<Element<'a, Message, Theme, crate::Renderer>>,
    Theme: iced_style::container::StyleSheet + LayeredTheme,
    <Theme as iced_widget::container::StyleSheet>::Style: From<crate::theme::Container>,
{
    LayerContainer::new(content)
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct LayerContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: iced_style::container::StyleSheet + LayeredTheme,
{
    layer: Option<cosmic_theme::Layer>,
    container: Container<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> LayerContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: iced_style::container::StyleSheet + LayeredTheme,
    <Theme as iced_style::container::StyleSheet>::Style: From<crate::theme::Container>,
{
    /// Creates an empty [`Container`].
    pub(crate) fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        LayerContainer {
            layer: None,
            container: Container::new(content),
        }
    }

    /// Sets the [`Layer`] of the [`LayerContainer`].
    #[must_use]
    pub fn layer(mut self, layer: cosmic_theme::Layer) -> Self {
        self.layer = Some(layer);
        self.style(match layer {
            cosmic_theme::Layer::Background => crate::theme::Container::Background,
            cosmic_theme::Layer::Primary => crate::theme::Container::Primary,
            cosmic_theme::Layer::Secondary => crate::theme::Container::Secondary,
        })
    }

    /// Sets the [`Padding`] of the [`LayerContainer`].
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

    /// Sets the height of the [`LayerContainer`].
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`LayerContainer`].
    #[must_use]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`LayerContainer`] in pixels.
    #[must_use]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`LayerContainer`].
    #[must_use]
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`LayerContainer`].
    #[must_use]
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`LayerContainer`].
    #[must_use]
    pub fn center_x(mut self) -> Self {
        self.container = self.container.center_x();
        self
    }

    /// Centers the contents in the vertical axis of the [`LayerContainer`].
    #[must_use]
    pub fn center_y(mut self) -> Self {
        self.container = self.container.center_y();
        self
    }

    /// Sets the style of the [`LayerContainer`].
    #[must_use]
    pub fn style(mut self, style: impl Into<<Theme as StyleSheet>::Style>) -> Self {
        self.container = self.container.style(style);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for LayerContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
    Theme: iced_style::container::StyleSheet + LayeredTheme + Clone,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        self.container.tag()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.diff(tree);
    }

    fn state(&self) -> iced_core::widget::tree::State {
        self.container.state()
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
        self.container.layout(tree, renderer, limits)
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
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let theme = if let Some(layer) = self.layer {
            let mut theme = theme.clone();
            theme.set_layer(layer);
            theme
        } else {
            theme.clone()
        };
        self.container.draw(
            tree,
            renderer,
            &theme,
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
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.container.overlay(tree, layout, renderer)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        dnd_rectangles: &mut iced_style::core::clipboard::DndDestinationRectangles,
    ) {
        self.container
            .drag_destinations(state, layout, dnd_rectangles);
    }
}

impl<'a, Message, Theme, Renderer> From<LayerContainer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    Theme: iced_style::container::StyleSheet + LayeredTheme + 'a + Clone,
{
    fn from(
        column: LayerContainer<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(column)
    }
}
