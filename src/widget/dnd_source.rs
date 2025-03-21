use std::any::Any;

use iced_core::window;

use crate::{
    Element,
    iced::{
        Event, Length, Point, Rectangle, Vector,
        clipboard::dnd::{DndAction, DndEvent, SourceEvent},
        event, mouse, overlay,
    },
    iced_core::{
        self, Clipboard, Shell, layout, renderer,
        widget::{Tree, tree},
    },
    widget::{Id, Widget, container},
};

pub fn dnd_source<
    'a,
    Message: Clone + 'static,
    D: iced::clipboard::mime::AsMimeTypes + Send + 'static,
>(
    child: impl Into<Element<'a, Message>>,
) -> DndSource<'a, Message, D> {
    DndSource::new(child)
}

pub struct DndSource<'a, Message, D> {
    id: Id,
    action: DndAction,
    container: Element<'a, Message>,
    window: Option<window::Id>,
    drag_content: Option<Box<dyn Fn() -> D>>,
    drag_icon: Option<Box<dyn Fn(Vector) -> (Element<'static, ()>, tree::State, Vector)>>,
    on_start: Option<Message>,
    on_cancelled: Option<Message>,
    on_finish: Option<Message>,
    drag_threshold: f32,
}

impl<
    'a,
    Message: Clone + 'static,
    D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static,
> DndSource<'a, Message, D>
{
    pub fn new(child: impl Into<Element<'a, Message>>) -> Self {
        Self {
            id: Id::unique(),
            window: None,
            action: DndAction::Copy | DndAction::Move,
            container: container(child).into(),
            drag_content: None,
            drag_icon: None,
            drag_threshold: 8.0,
            on_start: None,
            on_cancelled: None,
            on_finish: None,
        }
    }

    pub fn with_id(child: impl Into<Element<'a, Message>>, id: Id) -> Self {
        Self {
            id,
            window: None,
            action: DndAction::Copy | DndAction::Move,
            container: container(child).into(),
            drag_content: None,
            drag_icon: None,
            drag_threshold: 8.0,
            on_start: None,
            on_cancelled: None,
            on_finish: None,
        }
    }

    #[must_use]
    pub fn action(mut self, action: DndAction) -> Self {
        self.action = action;
        self
    }

    #[must_use]
    pub fn drag_content(mut self, f: impl Fn() -> D + 'static) -> Self {
        self.drag_content = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn drag_icon(
        mut self,
        f: impl Fn(Vector) -> (Element<'static, ()>, tree::State, Vector) + 'static,
    ) -> Self {
        self.drag_icon = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold;
        self
    }

    pub fn start_dnd(&self, clipboard: &mut dyn Clipboard, bounds: Rectangle, offset: Vector) {
        let Some(content) = self.drag_content.as_ref().map(|f| f()) else {
            return;
        };

        iced_core::clipboard::start_dnd(
            clipboard,
            false,
            if let Some(window) = self.window.as_ref() {
                Some(iced_core::clipboard::DndSource::Surface(*window))
            } else {
                Some(iced_core::clipboard::DndSource::Widget(self.id.clone()))
            },
            self.drag_icon.as_ref().map(|f| {
                let (icon, state, offset) = f(offset);
                iced_core::clipboard::IconSurface::new(
                    container(icon)
                        .width(Length::Fixed(bounds.width))
                        .height(Length::Fixed(bounds.height))
                        .into(),
                    state,
                    offset,
                )
            }),
            Box::new(content),
            self.action,
        );
    }

    pub fn on_start(mut self, on_start: Option<Message>) -> Self {
        self.on_start = on_start;
        self
    }

    pub fn on_cancel(mut self, on_cancelled: Option<Message>) -> Self {
        self.on_cancelled = on_cancelled;
        self
    }

    pub fn on_finish(mut self, on_finish: Option<Message>) -> Self {
        self.on_finish = on_finish;
        self
    }

    pub fn window(mut self, window: window::Id) -> Self {
        self.window = Some(window);
        self
    }
}

impl<Message: Clone + 'static, D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static>
    Widget<Message, crate::Theme, crate::Renderer> for DndSource<'_, Message, D>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.container)]
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.children[0].diff(self.container.as_widget_mut());
    }

    fn state(&self) -> iced_core::widget::tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.container.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let node = self
            .container
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits);
        state.cached_bounds = node.bounds();
        node
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        operation.custom((&mut tree.state) as &mut dyn Any, Some(&self.id));
        operation.container(Some(&self.id), layout.bounds(), &mut |operation| {
            self.container
                .as_widget()
                .operate(&mut tree.children[0], layout, renderer, operation)
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let ret = self.container.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position() {
                        if !state.hovered {
                            return ret;
                        }

                        state.left_pressed_position = Some(position);
                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left)
                    if state.left_pressed_position.is_some() =>
                {
                    state.left_pressed_position = None;
                    return event::Status::Captured;
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor.position() {
                        if state.hovered {
                            // We ignore motion if we do not possess drag content by now.
                            if self.drag_content.is_none() {
                                state.left_pressed_position = None;
                                return ret;
                            }
                            if let Some(left_pressed_position) = state.left_pressed_position {
                                if position.distance(left_pressed_position) > self.drag_threshold {
                                    if let Some(on_start) = self.on_start.as_ref() {
                                        shell.publish(on_start.clone())
                                    }
                                    let offset = Vector::new(
                                        left_pressed_position.x - layout.bounds().x,
                                        left_pressed_position.y - layout.bounds().y,
                                    );
                                    self.start_dnd(clipboard, state.cached_bounds, offset);
                                    state.is_dragging = true;
                                    state.left_pressed_position = None;
                                }
                            }
                            if !cursor.is_over(layout.bounds()) {
                                state.hovered = false;

                                return ret;
                            }
                        } else if cursor.is_over(layout.bounds()) {
                            state.hovered = true;
                        }
                        return event::Status::Captured;
                    }
                }
                _ => return ret,
            },
            Event::Dnd(DndEvent::Source(SourceEvent::Cancelled)) => {
                if state.is_dragging {
                    if let Some(m) = self.on_cancelled.as_ref() {
                        shell.publish(m.clone());
                    }
                    state.is_dragging = false;
                    return event::Status::Captured;
                }
                return ret;
            }
            Event::Dnd(DndEvent::Source(SourceEvent::Finished)) => {
                if state.is_dragging {
                    if let Some(m) = self.on_finish.as_ref() {
                        shell.publish(m.clone());
                    }
                    state.is_dragging = false;
                    return event::Status::Captured;
                }
                return ret;
            }
            _ => return ret,
        }
        ret
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.is_dragging {
            return mouse::Interaction::Grabbing;
        }
        self.container.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        renderer_style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.container.as_widget().draw(
            &tree.children[0],
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
        layout: layout::Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.container
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, translation)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: layout::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.container.as_widget().drag_destinations(
            &state.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let c_state = &state.children[0];
        self.container.as_widget().a11y_nodes(layout, c_state, p)
    }
}

impl<
    'a,
    Message: Clone + 'static,
    D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static,
> From<DndSource<'a, Message, D>> for Element<'a, Message>
{
    fn from(e: DndSource<'a, Message, D>) -> Element<'a, Message> {
        Element::new(e)
    }
}

/// Local state of the [`MouseListener`].
#[derive(Debug, Default)]
struct State {
    hovered: bool,
    left_pressed_position: Option<Point>,
    is_dragging: bool,
    cached_bounds: Rectangle,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}
