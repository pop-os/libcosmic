// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::*;

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

        forward(in_rx, inner_tx, InnerMessage::Message);

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

/// Used to drop the component's event loop when the managed widget is destroyed.
enum InnerMessage<T> {
    Drop,
    Message(T),
}
