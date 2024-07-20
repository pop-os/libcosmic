// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Subscribe to common application keyboard shortcuts.

use iced::{event, keyboard, mouse, Command, Event, Subscription};
use iced_core::{
    keyboard::key::Named,
    widget::{operation, Id, Operation},
    Rectangle,
};
use iced_futures::event::listen_raw;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Escape,
    FocusNext,
    FocusPrevious,
    Fullscreen,
    Search,
}

pub fn subscription() -> Subscription<Message> {
    listen_raw(|event, status| {
        if event::Status::Ignored != status {
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                modifiers,
                ..
            }) => match key {
                Named::Tab => {
                    return Some(if modifiers.shift() {
                        Message::FocusPrevious
                    } else {
                        Message::FocusNext
                    });
                }

                Named::Escape => {
                    return Some(Message::Escape);
                }

                Named::F11 => {
                    return Some(Message::Fullscreen);
                }

                _ => (),
            },
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if c == "f" && modifiers.control() => {
                return Some(Message::Search);
            }

            _ => (),
        }

        None
    })
}
