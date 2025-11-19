use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use iced::Vector;

use crate::{
    Element,
    iced::{
        Event, Length, Rectangle,
        clipboard::{
            dnd::{self, DndAction, DndDestinationRectangle, DndEvent, OfferEvent},
            mime::AllowedMimeTypes,
        },
        event,
        id::Internal,
        mouse, overlay,
    },
    iced_core::{
        self, Clipboard, Shell, layout,
        widget::{Tree, tree},
    },
    widget::{Id, Widget},
};

pub fn dnd_destination<'a, Message: 'static>(
    child: impl Into<Element<'a, Message>>,
    mimes: Vec<Cow<'static, str>>,
) -> DndDestination<'a, Message> {
    DndDestination::new(child, mimes)
}

pub fn dnd_destination_for_data<'a, T: AllowedMimeTypes, Message: 'static>(
    child: impl Into<Element<'a, Message>>,
    on_finish: impl Fn(Option<T>, DndAction) -> Message + 'static,
) -> DndDestination<'a, Message> {
    DndDestination::for_data(child, on_finish)
}

static DRAG_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
const DND_DEST_LOG_TARGET: &str = "libcosmic::widget::dnd_destination";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragId(pub u128);

impl DragId {
    pub fn new() -> Self {
        DragId(u128::from(u64::MAX) + u128::from(DRAG_ID_COUNTER.fetch_add(1, Ordering::Relaxed)))
    }
}

#[allow(clippy::new_without_default)]
impl Default for DragId {
    fn default() -> Self {
        DragId::new()
    }
}

pub struct DndDestination<'a, Message> {
    id: Id,
    drag_id: Option<u64>,
    preferred_action: DndAction,
    action: DndAction,
    container: Element<'a, Message>,
    mime_types: Vec<Cow<'static, str>>,
    forward_drag_as_cursor: bool,
    on_hold: Option<Box<dyn Fn(f64, f64) -> Message>>,
    on_drop: Option<Box<dyn Fn(f64, f64) -> Message>>,
    on_enter: Option<Box<dyn Fn(f64, f64, Vec<String>) -> Message>>,
    on_leave: Option<Box<dyn Fn() -> Message>>,
    on_motion: Option<Box<dyn Fn(f64, f64) -> Message>>,
    on_action_selected: Option<Box<dyn Fn(DndAction) -> Message>>,
    on_data_received: Option<Box<dyn Fn(String, Vec<u8>) -> Message>>,
    on_finish: Option<Box<dyn Fn(String, Vec<u8>, DndAction, f64, f64) -> Message>>,
}

impl<'a, Message: 'static> DndDestination<'a, Message> {
    fn mime_matches(&self, offered: &[String]) -> bool {
        self.mime_types.is_empty()
            || offered
                .iter()
                .any(|mime| self.mime_types.iter().any(|allowed| allowed == mime))
    }
    pub fn new(child: impl Into<Element<'a, Message>>, mimes: Vec<Cow<'static, str>>) -> Self {
        Self {
            id: Id::unique(),
            drag_id: None,
            mime_types: mimes,
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: None,
        }
    }

    pub fn for_data<T: AllowedMimeTypes>(
        child: impl Into<Element<'a, Message>>,
        on_finish: impl Fn(Option<T>, DndAction) -> Message + 'static,
    ) -> Self {
        Self {
            id: Id::unique(),
            drag_id: None,
            mime_types: T::allowed().iter().cloned().map(Cow::Owned).collect(),
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: Some(Box::new(move |mime, data, action, _, _| {
                on_finish(T::try_from((data, mime)).ok(), action)
            })),
        }
    }

    #[must_use]
    pub fn data_received_for<T: AllowedMimeTypes>(
        mut self,
        f: impl Fn(Option<T>) -> Message + 'static,
    ) -> Self {
        self.on_data_received = Some(Box::new(
            move |mime, data| f(T::try_from((data, mime)).ok()),
        ));
        self
    }

    pub fn with_id(
        child: impl Into<Element<'a, Message>>,
        id: Id,
        mimes: Vec<Cow<'static, str>>,
    ) -> Self {
        Self {
            id,
            drag_id: None,
            mime_types: mimes,
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: None,
        }
    }

    #[must_use]
    pub fn drag_id(mut self, id: u64) -> Self {
        self.drag_id = Some(id);
        self
    }

    #[must_use]
    pub fn action(mut self, action: DndAction) -> Self {
        self.action = action;
        self
    }

    #[must_use]
    pub fn preferred_action(mut self, action: DndAction) -> Self {
        self.preferred_action = action;
        self
    }

    #[must_use]
    pub fn forward_drag_as_cursor(mut self, forward: bool) -> Self {
        self.forward_drag_as_cursor = forward;
        self
    }

    #[must_use]
    pub fn on_hold(mut self, f: impl Fn(f64, f64) -> Message + 'static) -> Self {
        self.on_hold = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_drop(mut self, f: impl Fn(f64, f64) -> Message + 'static) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_enter(mut self, f: impl Fn(f64, f64, Vec<String>) -> Message + 'static) -> Self {
        self.on_enter = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_leave(mut self, m: impl Fn() -> Message + 'static) -> Self {
        self.on_leave = Some(Box::new(m));
        self
    }

    #[must_use]
    pub fn on_finish(
        mut self,
        f: impl Fn(String, Vec<u8>, DndAction, f64, f64) -> Message + 'static,
    ) -> Self {
        self.on_finish = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_motion(mut self, f: impl Fn(f64, f64) -> Message + 'static) -> Self {
        self.on_motion = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_action_selected(mut self, f: impl Fn(DndAction) -> Message + 'static) -> Self {
        self.on_action_selected = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_data_received(mut self, f: impl Fn(String, Vec<u8>) -> Message + 'static) -> Self {
        self.on_data_received = Some(Box::new(f));
        self
    }

    /// Returns the drag id of the destination.
    ///
    /// # Panics
    /// Panics if the destination has been assigned a Set id, which is invalid.
    #[must_use]
    pub fn get_drag_id(&self) -> u128 {
        u128::from(self.drag_id.unwrap_or_else(|| match &self.id.0 {
            Internal::Unique(id) | Internal::Custom(id, _) => *id,
            Internal::Set(_) => panic!("Invalid Id assigned to dnd destination."),
        }))
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

impl<Message: 'static> Widget<Message, crate::Theme, crate::Renderer>
    for DndDestination<'_, Message>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.container)]
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        tree::Tag::of::<State<()>>()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.children[0].diff(self.container.as_widget_mut());
    }

    fn state(&self) -> iced_core::widget::tree::State {
        tree::State::new(State::<()>::new())
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
        self.container
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        self.container
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    #[allow(clippy::too_many_lines)]
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
        let s = self.container.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if matches!(s, event::Status::Captured) {
            return event::Status::Captured;
        }

        let state = tree.state.downcast_mut::<State<()>>();

        let my_id = self.get_drag_id();

        log::trace!(
            target: DND_DEST_LOG_TARGET,
            "dnd_destination id={:?}: event {:?}",
            self.drag_id.unwrap_or_default(),
            event
        );
        match event {
            Event::Dnd(DndEvent::Offer(
                id,
                OfferEvent::Enter {
                    x, y, mime_types, ..
                },
            )) if id == Some(my_id) => {
                if !self.mime_matches(&mime_types) {
                    log::trace!(
                        target: DND_DEST_LOG_TARGET,
                        "offer enter id={my_id:?} ignored (mimes={mime_types:?} not in {:?})",
                        self.mime_types
                    );
                    return event::Status::Ignored;
                }
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer enter id={my_id:?} coords=({x},{y}) mimes={mime_types:?}"
                );
                if let Some(msg) = state.on_enter(
                    x,
                    y,
                    mime_types,
                    self.on_enter.as_ref().map(std::convert::AsRef::as_ref),
                    (),
                ) {
                    shell.publish(msg);
                }
                if self.forward_drag_as_cursor {
                    #[allow(clippy::cast_possible_truncation)]
                    let drag_cursor = mouse::Cursor::Available((x as f32, y as f32).into());
                    let event = Event::Mouse(mouse::Event::CursorMoved {
                        position: drag_cursor.position().unwrap(),
                    });
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(_, OfferEvent::Leave)) => {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer leave id={:?}",
                    my_id
                );
                if let Some(msg) =
                    state.on_leave(self.on_leave.as_ref().map(std::convert::AsRef::as_ref))
                {
                    shell.publish(msg);
                }

                if self.forward_drag_as_cursor {
                    let drag_cursor = mouse::Cursor::Unavailable;
                    let event = Event::Mouse(mouse::Event::CursorLeft);
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Ignored;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Motion { x, y })) if id == Some(my_id) => {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer motion id={my_id:?} coords=({x},{y})"
                );
                if let Some(msg) = state.on_motion(
                    x,
                    y,
                    self.on_motion.as_ref().map(std::convert::AsRef::as_ref),
                    self.on_enter.as_ref().map(std::convert::AsRef::as_ref),
                    (),
                ) {
                    shell.publish(msg);
                }

                if self.forward_drag_as_cursor {
                    #[allow(clippy::cast_possible_truncation)]
                    let drag_cursor = mouse::Cursor::Available((x as f32, y as f32).into());
                    let event = Event::Mouse(mouse::Event::CursorMoved {
                        position: drag_cursor.position().unwrap(),
                    });
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(_, OfferEvent::LeaveDestination)) => {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer leave-destination id={:?}",
                    my_id
                );
                if let Some(msg) =
                    state.on_leave(self.on_leave.as_ref().map(std::convert::AsRef::as_ref))
                {
                    shell.publish(msg);
                }
                return event::Status::Ignored;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Drop)) if id == Some(my_id) => {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer drop id={my_id:?}"
                );
                if let Some(msg) =
                    state.on_drop(self.on_drop.as_ref().map(std::convert::AsRef::as_ref))
                {
                    shell.publish(msg);
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::SelectedAction(action)))
                if id == Some(my_id) =>
            {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer selected-action id={my_id:?} action={action:?}"
                );
                if let Some(msg) = state.on_action_selected(
                    action,
                    self.on_action_selected
                        .as_ref()
                        .map(std::convert::AsRef::as_ref),
                ) {
                    shell.publish(msg);
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Data { data, mime_type }))
                if id == Some(my_id) =>
            {
                log::trace!(
                    target: DND_DEST_LOG_TARGET,
                    "offer data id={my_id:?} mime={mime_type:?} bytes={}",
                    data.len()
                );
                if let (Some(msg), ret) = state.on_data_received(
                    mime_type,
                    data,
                    self.on_data_received
                        .as_ref()
                        .map(std::convert::AsRef::as_ref),
                    self.on_finish.as_ref().map(std::convert::AsRef::as_ref),
                ) {
                    shell.publish(msg);
                    return ret;
                }
                return event::Status::Captured;
            }
            _ => {}
        }
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
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
        renderer_style: &iced_core::renderer::Style,
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
        let bounds = layout.bounds();
        let my_id = self.get_drag_id();
        log::trace!(
            target: DND_DEST_LOG_TARGET,
            "register destination id={:?} bounds=({:.2},{:.2},{:.2},{:.2}) mimes={:?}",
            my_id,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            self.mime_types
        );
        let my_dest = DndDestinationRectangle {
            id: my_id,
            rectangle: dnd::Rectangle {
                x: f64::from(bounds.x),
                y: f64::from(bounds.y),
                width: f64::from(bounds.width),
                height: f64::from(bounds.height),
            },
            mime_types: self.mime_types.clone(),
            actions: self.action,
            preferred: self.preferred_action,
        };
        dnd_rectangles.push(my_dest);

        if let Some(child_layout) = layout.children().next() {
            self.container.as_widget().drag_destinations(
                &state.children[0],
                child_layout.with_virtual_offset(layout.virtual_offset()),
                renderer,
                dnd_rectangles,
            );
        }
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

#[derive(Default)]
pub struct State<T> {
    pub drag_offer: Option<DragOffer<T>>,
}

pub struct DragOffer<T> {
    pub x: f64,
    pub y: f64,
    pub dropped: bool,
    pub selected_action: DndAction,
    pub data: T,
}

impl<T> State<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { drag_offer: None }
    }

    pub fn on_enter<Message>(
        &mut self,
        x: f64,
        y: f64,
        mime_types: Vec<String>,
        on_enter: Option<impl Fn(f64, f64, Vec<String>) -> Message>,
        data: T,
    ) -> Option<Message> {
        self.drag_offer = Some(DragOffer {
            x,
            y,
            dropped: false,
            selected_action: DndAction::empty(),
            data,
        });
        on_enter.map(|f| f(x, y, mime_types))
    }

    pub fn on_leave<Message>(&mut self, on_leave: Option<&dyn Fn() -> Message>) -> Option<Message> {
        if self.drag_offer.as_ref().is_some_and(|d| !d.dropped) {
            self.drag_offer = None;
            on_leave.map(|f| f())
        } else {
            None
        }
    }

    pub fn on_motion<Message>(
        &mut self,
        x: f64,
        y: f64,
        on_motion: Option<impl Fn(f64, f64) -> Message>,
        on_enter: Option<impl Fn(f64, f64, Vec<String>) -> Message>,
        data: T,
    ) -> Option<Message> {
        if let Some(s) = self.drag_offer.as_mut() {
            s.x = x;
            s.y = y;
        } else {
            self.drag_offer = Some(DragOffer {
                x,
                y,
                dropped: false,
                selected_action: DndAction::empty(),
                data,
            });
            if let Some(f) = on_enter {
                return Some(f(x, y, vec![]));
            }
        }
        on_motion.map(|f| f(x, y))
    }

    pub fn on_drop<Message>(
        &mut self,
        on_drop: Option<impl Fn(f64, f64) -> Message>,
    ) -> Option<Message> {
        if let Some(offer) = self.drag_offer.as_mut() {
            offer.dropped = true;
            if let Some(f) = on_drop {
                return Some(f(offer.x, offer.y));
            }
        }
        None
    }

    pub fn on_action_selected<Message>(
        &mut self,
        action: DndAction,
        on_action_selected: Option<impl Fn(DndAction) -> Message>,
    ) -> Option<Message> {
        if let Some(s) = self.drag_offer.as_mut() {
            s.selected_action = action;
        }
        if let Some(f) = on_action_selected {
            f(action).into()
        } else {
            None
        }
    }

    pub fn on_data_received<Message>(
        &mut self,
        mime: String,
        data: Vec<u8>,
        on_data_received: Option<impl Fn(String, Vec<u8>) -> Message>,
        on_finish: Option<impl Fn(String, Vec<u8>, DndAction, f64, f64) -> Message>,
    ) -> (Option<Message>, event::Status) {
        let Some(dnd) = self.drag_offer.as_ref() else {
            self.drag_offer = None;
            return (None, event::Status::Ignored);
        };

        if dnd.dropped {
            let ret = (
                on_finish.map(|f| f(mime, data, dnd.selected_action, dnd.x, dnd.y)),
                event::Status::Captured,
            );
            self.drag_offer = None;
            ret
        } else if let Some(f) = on_data_received {
            (Some(f(mime, data)), event::Status::Captured)
        } else {
            (None, event::Status::Ignored)
        }
    }
}

impl<'a, Message: 'static> From<DndDestination<'a, Message>> for Element<'a, Message> {
    fn from(wrapper: DndDestination<'a, Message>) -> Self {
        Element::new(wrapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum TestMsg {
        Data,
        Finished,
    }

    #[test]
    fn data_before_drop_invokes_data_handler_only() {
        let mut state: State<()> = State::new();
        assert!(state.drag_offer.is_none());
        state.on_enter::<TestMsg>(
            4.0,
            2.0,
            vec!["text/plain".into()],
            Option::<fn(_, _, _) -> TestMsg>::None,
            (),
        );
        let (message, status) = state.on_data_received(
            "text/plain".into(),
            vec![1],
            Some(|mime, data| {
                assert_eq!(mime, "text/plain");
                assert_eq!(data, vec![1]);
                TestMsg::Data
            }),
            Option::<fn(_, _, _, _, _) -> TestMsg>::None,
        );
        assert!(matches!(message, Some(TestMsg::Data)));
        assert_eq!(status, event::Status::Captured);
        assert!(state.drag_offer.is_some());
    }

    #[test]
    fn finish_only_emits_after_drop() {
        let mut state: State<()> = State::new();
        state.on_enter::<TestMsg>(
            5.0,
            -1.0,
            vec![],
            Option::<fn(_, _, _) -> TestMsg>::None,
            (),
        );
        state.on_action_selected::<TestMsg>(DndAction::Move, Option::<fn(_) -> TestMsg>::None);
        state.on_drop::<TestMsg>(Option::<fn(_, _) -> TestMsg>::None);

        let (message, status) = state.on_data_received(
            "application/x-test".into(),
            vec![7],
            Option::<fn(_, _) -> TestMsg>::None,
            Some(|mime, data, action, x, y| {
                assert_eq!(mime, "application/x-test");
                assert_eq!(data, vec![7]);
                assert_eq!(action, DndAction::Move);
                assert_eq!(x, 5.0);
                assert_eq!(y, -1.0);
                TestMsg::Finished
            }),
        );
        assert!(matches!(message, Some(TestMsg::Finished)));
        assert_eq!(status, event::Status::Captured);
        assert!(state.drag_offer.is_none());
    }
}
