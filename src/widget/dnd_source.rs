use crate::{
    iced::{
        clipboard::dnd::{DndAction, DndEvent, SourceEvent},
        event, mouse, overlay, Event, Length, Point, Rectangle,
    },
    iced_core::{
        self, layout, renderer,
        widget::{tree, Tree},
        Clipboard, Shell,
    },
    iced_style,
    widget::{container, Id, Widget},
    Element,
};

pub fn dnd_source<
    'a,
    Message: 'static,
    AppMessage: 'static,
    D: iced::clipboard::mime::AsMimeTypes + Send + 'static,
>(
    child: impl Into<Element<'a, Message>>,
) -> DndSource<'a, Message, AppMessage, D> {
    DndSource::new(child)
}

pub struct DndSource<'a, Message, AppMessage, D> {
    id: Id,
    action: DndAction,
    container: Element<'a, Message>,
    drag_content: Option<Box<dyn Fn() -> D>>,
    drag_icon: Option<Box<dyn Fn() -> (Element<'static, AppMessage>, tree::State)>>,
    drag_threshold: f32,
    _phantom: std::marker::PhantomData<AppMessage>,
}

impl<
        'a,
        Message: 'static,
        AppMessage: 'static,
        D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static,
    > DndSource<'a, Message, AppMessage, D>
{
    pub fn new(child: impl Into<Element<'a, Message>>) -> Self {
        Self {
            id: Id::unique(),
            action: DndAction::Copy | DndAction::Move,
            container: container(child).into(),
            drag_content: None,
            drag_icon: None,
            drag_threshold: 8.0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_id(child: impl Into<Element<'a, Message>>, id: Id) -> Self {
        Self {
            id,
            action: DndAction::Copy | DndAction::Move,
            container: container(child).into(),
            drag_content: None,
            drag_icon: None,
            drag_threshold: 8.0,
            _phantom: std::marker::PhantomData,
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
        f: impl Fn() -> (Element<'static, AppMessage>, tree::State) + 'static,
    ) -> Self {
        self.drag_icon = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold;
        self
    }

    pub fn start_dnd(&self, clipboard: &mut dyn Clipboard, bounds: Rectangle) {
        let Some(content) = self.drag_content.as_ref().map(|f| f()) else {
            return;
        };
        iced_core::clipboard::start_dnd(
            clipboard,
            false,
            Some(iced_core::clipboard::DndSource::Widget(self.id.clone())),
            self.drag_icon.as_ref().map(|f| {
                let (icon, state) = f();
                (
                    container(icon)
                        .width(Length::Fixed(bounds.width))
                        .height(Length::Fixed(bounds.height))
                        .into(),
                    state,
                )
            }),
            Box::new(content),
            self.action,
        );
    }
}

impl<
        'a,
        Message: 'static,
        AppMessage: 'static,
        D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static,
    > Widget<Message, crate::Theme, crate::Renderer> for DndSource<'a, Message, AppMessage, D>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.container)]
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.as_widget_mut().diff(&mut tree.children[0]);
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
        operation: &mut dyn iced_core::widget::Operation<
            iced_core::widget::OperationOutputWrapper<Message>,
        >,
    ) {
        self.container
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
                                    self.start_dnd(clipboard, state.cached_bounds);
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
            Event::Dnd(DndEvent::Source(SourceEvent::Cancelled | SourceEvent::Finished)) => {
                if state.is_dragging {
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
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.container
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: layout::Layout<'_>,
        dnd_rectangles: &mut iced_style::core::clipboard::DndDestinationRectangles,
    ) {
        self.container
            .as_widget()
            .drag_destinations(&state.children[0], layout, dnd_rectangles);
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<
        'a,
        Message: 'static,
        AppMessage: 'static,
        D: iced::clipboard::mime::AsMimeTypes + std::marker::Send + 'static,
    > From<DndSource<'a, Message, AppMessage, D>> for Element<'a, Message>
{
    fn from(e: DndSource<'a, Message, AppMessage, D>) -> Element<'a, Message> {
        Element::new(e)
    }
}

/// Local state of the [`MouseListener`].
#[derive(Default)]
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
