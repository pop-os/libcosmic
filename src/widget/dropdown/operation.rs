// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0 AND MIT
//! Operate on dropdown widgets.

use super::State;
use iced::Rectangle;
use iced_core::widget::{Id, Operation};

pub trait Dropdown {
    fn close(&mut self);
    fn open(&mut self);
}

/// Produces a [`Task`] that closes a [`Dropdown`] popup.
pub fn close<T>(id: Id) -> impl Operation<T> {
    struct Close(Id);

    impl<T> Operation<T> for Close {
        fn custom(&mut self, state: &mut dyn std::any::Any, id: Option<&Id>) {
            if id.map_or(true, |id| id != &self.0) {
                return;
            }

            let Some(state) = state.downcast_mut::<State>() else {
                return;
            };

            state.close();
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }
    }

    Close(id)
}

/// Produces a [`Task`] that opens a [`Dropdown`] popup.
pub fn open<T>(id: Id) -> impl Operation<T> {
    struct Open(Id);

    impl<T> Operation<T> for Open {
        fn custom(&mut self, state: &mut dyn std::any::Any, id: Option<&Id>) {
            if id.map_or(true, |id| id != &self.0) {
                return;
            }

            let Some(state) = state.downcast_mut::<State>() else {
                return;
            };

            state.open();
        }

        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }
    }

    Open(id)
}
