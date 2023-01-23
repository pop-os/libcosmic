use iced::{event, keyboard, mouse, subscription, Command, Event, Subscription};
use iced_native::widget::{operation, Id, Operation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    FocusNext,
    FocusPrevious,
    Unfocus,
}

#[must_use]
pub fn subscription() -> Subscription<Message> {
    subscription::events_with(|event, status| match (event, status) {
        (
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Tab,
                modifiers,
                ..
            }),
            event::Status::Ignored,
        ) => Some(if modifiers.shift() {
            Message::FocusPrevious
        } else {
            Message::FocusNext
        }),
        (Event::Mouse(mouse::Event::ButtonPressed { .. }), event::Status::Ignored) => {
            Some(Message::Unfocus)
        }
        _ => None,
    })
}

/// Unfocuses any actively-focused widget.
#[must_use]
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
