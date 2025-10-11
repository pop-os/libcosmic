use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::{Id, Tree};
use iced_core::{Clipboard, Element, Layout, Length, Rectangle, Shell, Vector, Widget};
pub use iced_widget::container::{Catalog, Style};

pub fn id_container<'a, Message: 'static, Theme, E>(
    content: E,
    id: Id,
) -> IdContainer<'a, Message, Theme, crate::Renderer>
where
    E: Into<Element<'a, Message, Theme, crate::Renderer>>,
    Theme: iced_widget::container::Catalog,
    <Theme as iced_widget::container::Catalog>::Class<'a>: From<crate::theme::Container<'a>>,
{
    IdContainer::new(content, id)
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct IdContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    id: Id,
}

impl<'a, Message, Theme, Renderer> IdContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// Creates an empty [`IdContainer`].
    pub(crate) fn new<T>(content: T, id: Id) -> Self
    where
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        IdContainer {
            content: content.into(),
            id,
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for IdContainer<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.children[0].diff(&mut self.content);
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
        let node = self
            .content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits);
        let size = node.size();
        layout::Node::with_children(size, vec![node])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        operation.container(Some(&self.id), layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
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
        self.content.as_widget_mut().on_event(
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
        layout: Layout<'_>,
        renderer: &Renderer,
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

    fn id(&self) -> Option<crate::widget::Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: crate::widget::Id) {
        self.id = id;
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

impl<'a, Message, Theme, Renderer> From<IdContainer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    Theme: 'a,
{
    fn from(c: IdContainer<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(c)
    }
}
