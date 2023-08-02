// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

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
    Fullscreen,
    Unfocus,
    Search,
}

pub fn subscription() -> Subscription<Message> {
    subscription::events_with(|event, status| {
        if event::Status::Ignored != status {
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) => match key_code {
                KeyCode::Tab => {
                    return Some(if modifiers.shift() {
                        Message::FocusPrevious
                    } else {
                        Message::FocusNext
                    });
                }

                KeyCode::Escape => {
                    return Some(Message::Escape);
                }

                KeyCode::F11 => {
                    return Some(Message::Fullscreen);
                }

                KeyCode::F => {
                    return if modifiers.control() {
                        Some(Message::Search)
                    } else {
                        None
                    };
                }

                _ => (),
            },

            Event::Mouse(mouse::Event::ButtonPressed { .. }) => {
                return Some(Message::Unfocus);
            }

            _ => (),
        }

        None
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
