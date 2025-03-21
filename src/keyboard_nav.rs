// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Subscribe to common application keyboard shortcuts.

use iced::{Event, Subscription, event, keyboard};
use iced_core::keyboard::key::Named;
use iced_futures::event::listen_raw;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Escape,
    FocusNext,
    FocusPrevious,
    Fullscreen,
    Search,
}

#[cold]
pub fn subscription() -> Subscription<Action> {
    listen_raw(|event, status, _| {
        if event::Status::Ignored != status {
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                modifiers,
                ..
            }) => match key {
                Named::Tab if !modifiers.control() => {
                    return Some(if modifiers.shift() {
                        Action::FocusPrevious
                    } else {
                        Action::FocusNext
                    });
                }

                Named::Escape => {
                    return Some(Action::Escape);
                }

                Named::F11 => {
                    return Some(Action::Fullscreen);
                }

                _ => (),
            },
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if c == "f" && modifiers.control() => {
                return Some(Action::Search);
            }

            _ => (),
        }

        None
    })
}
