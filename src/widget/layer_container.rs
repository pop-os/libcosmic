use crate::Theme;
use cosmic_theme::LayeredTheme;
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

pub fn layer_container<'a, Message: 'static, E>(
    content: E,
) -> LayerContainer<'a, Message, crate::Renderer>
where
    E: Into<Element<'a, Message, Theme, crate::Renderer>>,
{
    LayerContainer::new(content)
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct LayerContainer<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    layer: Option<cosmic_theme::Layer>,
    container: Container<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Renderer> LayerContainer<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    // iced_widget::container::Style: From<crate::theme::Container>,
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
        self.class(match layer {
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
    #[inline]
    pub fn width(mut self, width: Length) -> Self {
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`LayerContainer`].
    #[must_use]
    #[inline]
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`LayerContainer`].
    #[must_use]
    #[inline]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`LayerContainer`] in pixels.
    #[must_use]
    #[inline]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`LayerContainer`].
    #[must_use]
    #[inline]
    pub fn align_x(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`LayerContainer`].
    #[must_use]
    #[inline]
    pub fn align_y(mut self, alignment: Alignment) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`LayerContainer`].
    #[must_use]
    #[inline]
    pub fn center_x(mut self, width: Length) -> Self {
        self.container = self.container.center_x(width);
        self
    }

    /// Centers the contents in the vertical axis of the [`LayerContainer`].
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

    /// Sets the style of the [`LayerContainer`].
    #[must_use]
    pub fn class(mut self, style: impl Into<crate::style::iced::Container<'a>>) -> Self {
        self.container = self.container.class(style);
        self
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for LayerContainer<'_, Message, Renderer>
where
    Renderer: iced_core::Renderer,
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
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
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

    fn id(&self) -> Option<crate::widget::Id> {
        Widget::id(&self.container)
    }

    fn set_id(&mut self, id: crate::widget::Id) {
        self.container.set_id(id);
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &Tree,
        p: iced::mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        self.container.a11y_nodes(layout, state, p)
    }
}

impl<'a, Message, Renderer> From<LayerContainer<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
{
    fn from(
        column: LayerContainer<'a, Message, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(column)
    }
}
