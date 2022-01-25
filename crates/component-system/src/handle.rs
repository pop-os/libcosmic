// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::*;

/// Handle to an active widget component in the system.
pub struct Handle<W, I> {
    /// The widget that this component manages.
    pub(crate) widget: W,

    /// Used for emitting events to the component.
    pub(crate) sender: Sender<I>,
}

impl<W, I> Widget<W> for Handle<W, I> {
    fn widget(&self) -> &W {
        &self.widget
    }
}

impl<W, I> Handle<W, I> {
    pub fn emit(&self, event: I) {
        let _ = self.sender.send(event);
    }
}

/// A newly-registered component which supports destructuring the handle
/// by forwarding or ignoring outputs from the component.
pub struct Registered<W: Clone + AsRef<gtk4::Widget>, I, O> {
    /// Handle to the component that was registered.
    pub handle: Handle<W, I>,

    /// The outputs being received by the component.
    pub receiver: Receiver<O>,
}

impl<W: Clone + AsRef<gtk4::Widget>, I: 'static, O: 'static> Registered<W, I, O> {
    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(O) -> X) + 'static>(
        self,
        sender: Sender<X>,
        transform: F,
    ) -> Handle<W, I> {
        let Registered { handle, receiver } = self;
        forward(receiver, sender, transform);
        handle
    }

    pub fn handle<F: FnMut(O) + 'static>(self, mut func: F) -> Handle<W, I> {
        let Registered {
            handle,
            mut receiver,
        } = self;

        spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(event);
            }
        });

        handle
    }

    /// Ignore outputs from the component and take the handle.
    pub fn ignore(self) -> Handle<W, I> {
        self.handle
    }
}
