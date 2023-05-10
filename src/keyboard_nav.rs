use iced::{
    event,
    keyboard::{self, KeyCode},
    mouse, subscription, Command, Event, Subscription,
};
use iced_core::widget::{operation, Id, Operation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Escape,
    FocusNext,
    FocusPrevious,
    Unfocus,
    Search,
}

pub fn subscription() -> Subscription<Message> {
    subscription::events_with(|event, status| match (event, status) {
        // Focus
        (
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Tab,
                modifiers,
                ..
            }),
            event::Status::Ignored,
        ) => Some(if modifiers.shift() {
            Message::FocusPrevious
        } else {
            Message::FocusNext
        }),
        // Escape
        (
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Escape,
                ..
            }),
            _,
        ) => Some(Message::Escape),
        // Search
        (
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::F,
                modifiers,
            }),
            event::Status::Ignored,
        ) => {
            if modifiers.control() {
                Some(Message::Search)
            } else {
                None
            }
        }
        // Unfocus
        (Event::Mouse(mouse::Event::ButtonPressed { .. }), event::Status::Ignored) => {
            Some(Message::Unfocus)
        }
        _ => None,
    })
}

/// Unfocuses any actively-focused widget.
pub fn unfocus<Message: 'static>() -> Command<Message> {
    Command::<Message>::widget(unfocus_operation())
}

#[must_use]
fn unfocus_operation<T>() -> impl Operation<T> {
    struct Unfocus {}

    impl<T> Operation<T> for Unfocus {
        fn focusable(&mut self, state: &mut dyn operation::Focusable, _id: Option<&Id>) {
            if state.is_focused() {
                state.unfocus();
            }
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self);
        }
    }

    Unfocus {}
}
