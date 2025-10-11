//! Responsive Container, which will notify of size changes.

use iced::{Limits, Size};
use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::{Id, Tree, tree};
use iced_core::{Clipboard, Element, Layout, Length, Rectangle, Shell, Vector, Widget};

pub(crate) fn responsive_container<'a, Message: 'static, Theme, E>(
    content: E,
    id: Id,
    on_action: impl Fn(crate::surface::Action) -> Message + 'static,
) -> ResponsiveContainer<'a, Message, Theme, crate::Renderer>
where
    E: Into<Element<'a, Message, Theme, crate::Renderer>>,
    Theme: iced_widget::container::Catalog,
    <Theme as iced_widget::container::Catalog>::Class<'a>: From<crate::theme::Container<'a>>,
{
    ResponsiveContainer::new(content, id, on_action)
}

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct ResponsiveContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    id: Id,
    size: Option<Size>,
    on_action: Box<dyn Fn(crate::surface::Action) -> Message>,
}

impl<'a, Message, Theme, Renderer> ResponsiveContainer<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// Creates an empty [`IdContainer`].
    pub(crate) fn new<T>(
        content: T,
        id: Id,
        on_action: impl Fn(crate::surface::Action) -> Message + 'static,
    ) -> Self
    where
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        ResponsiveContainer {
            content: content.into(),
            id,
            size: None,
            on_action: Box::new(on_action),
        }
    }

    pub(crate) fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ResponsiveContainer<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

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
        let state = tree.state.downcast_mut::<State>();
        let unrestricted_size = self.size.unwrap_or_else(|| {
            let node =
                self.content
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, &Limits::NONE);
            node.size()
        });

        let max_size = limits.max();
        let old_max = state.limits.max();
        state.needs_update = (unrestricted_size.width > max_size.width)
            ^ (state.size.width > old_max.width)
            || (unrestricted_size.height > max_size.height) ^ (state.size.height > old_max.height);
        if state.needs_update {
            state.limits = *limits;
            state.size = unrestricted_size;
        }

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
        let state = tree.state.downcast_mut::<State>();

        if state.needs_update {
            shell.publish((self.on_action)(
                crate::surface::Action::ResponsiveMenuBar {
                    menu_bar: self.id.clone(),
                    limits: state.limits,
                    size: state.size,
                },
            ));
            state.needs_update = false;
        }

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

impl<'a, Message, Theme, Renderer> From<ResponsiveContainer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_core::Renderer,
    Theme: 'a,
{
    fn from(
        c: ResponsiveContainer<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(c)
    }
}

#[derive(Debug, Clone, Copy)]
struct State {
    limits: Limits,
    size: Size,
    needs_update: bool,
}

impl State {
    fn new() -> Self {
        Self {
            limits: Limits::NONE,
            size: Size::new(0., 0.),
            needs_update: false,
        }
    }
}
