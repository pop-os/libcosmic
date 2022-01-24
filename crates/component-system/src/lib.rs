// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod macros;

use gtk4::prelude::*;
use tokio::sync::mpsc;

pub use gtk4 as gtk;
pub use relm4_macros::view;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

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

/// Handle to an active widget component in the system.
pub struct Handle<W, I> {
    /// The widget that this component manages.
    widget: W,

    /// Used for emitting events to the component.
    sender: Sender<I>,
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

/// Used to drop the component's event loop when the managed widget is destroyed.
enum InnerMessage<T> {
    Drop,
    Message(T),
}

/// Provides a convenience function for getting a widget out of a type.
pub trait Widget<W> {
    fn widget(&self) -> &W;
}

/// The basis of a COSMIC widget.
///
/// A component takes care of constructing the UI of a widget, managing an event-loop
/// which handles signals from within the widget, and supports forwarding messages to
/// the consumer of the component.
pub trait Component: Sized + 'static {
    /// The arguments that are passed to the init_view method.
    type InitialArgs;

    /// The message type that the component accepts as inputs.
    type Input: 'static;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The widget that was constructed by the component.
    type RootWidget: Clone + AsRef<gtk4::Widget>;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the component and attaches it to the default local executor.
    ///
    /// Spawns an event loop on `glib::MainContext::default()`, which exists
    /// for as long as the root widget remains alive.
    fn register(
        mut self,
        args: Self::InitialArgs,
    ) -> Registered<Self::RootWidget, Self::Input, Self::Output> {
        let (mut sender, in_rx) = mpsc::unbounded_channel::<Self::Input>();
        let (mut out_tx, output) = mpsc::unbounded_channel::<Self::Output>();

        let (mut widgets, widget) = self.init_view(args, &mut sender, &mut out_tx);

        let handle = Handle {
            widget,
            sender: sender.clone(),
        };

        let (inner_tx, mut inner_rx) = mpsc::unbounded_channel::<InnerMessage<Self::Input>>();

        handle.widget.as_ref().connect_destroy({
            let sender = inner_tx.clone();
            move |_| {
                let _ = sender.send(InnerMessage::Drop);
            }
        });

        spawn_local(async move {
            while let Some(event) = inner_rx.recv().await {
                match event {
                    InnerMessage::Message(event) => {
                        self.update(&mut widgets, event, &mut sender, &mut out_tx);
                    }

                    InnerMessage::Drop => break,
                }
            }
        });

        forward(in_rx, inner_tx, |event| InnerMessage::Message(event));

        Registered {
            handle,
            receiver: output,
        }
    }

    /// Creates the initial view and root widget.
    fn init_view(
        &mut self,
        args: Self::InitialArgs,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> (Self::Widgets, Self::RootWidget);

    /// Handles input messages and enables the programmer to update the model and view.
    #[allow(unused_variables)]
    fn update(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
    }
}

/// Convenience function for `Component::register()`.
pub fn register<C: Component>(
    model: C,
    args: C::InitialArgs,
) -> Registered<C::RootWidget, C::Input, C::Output> {
    model.register(args)
}

/// Convenience function for forwarding events from a receiver to different sender.
pub fn forward<I: 'static, O: 'static, F: (Fn(I) -> O) + 'static>(
    mut receiver: Receiver<I>,
    sender: Sender<O>,
    transformer: F,
) {
    spawn_local(async move {
        while let Some(event) = receiver.recv().await {
            if sender.send(transformer(event)).is_err() {
                break;
            }
        }
    })
}

/// Convenience function for launching an application.
pub fn run<F: Fn(gtk4::Application) + 'static>(func: F) {
    use gtk4::prelude::*;
    let app = gtk4::Application::new(None, Default::default());

    app.connect_activate(move |app| func(app.clone()));

    app.run();
}

/// Convenience function for spawning tasks on the local executor
pub fn spawn_local<F: std::future::Future<Output = ()> + 'static>(func: F) {
    gtk4::glib::MainContext::default().spawn_local(func);
}
