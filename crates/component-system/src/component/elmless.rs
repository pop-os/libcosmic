// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The basis of a COSMIC widget.
///
/// A component takes care of constructing the UI of a widget, managing an event-loop
/// which handles signals from within the widget, and supports forwarding messages to
/// the consumer of the component.
#[async_trait::async_trait]
pub trait Component: Sized + 'static {
    /// The arguments that are passed to the init_view method.
    type InitParams;

    /// The message type that the component accepts as inputs.
    type Input: 'static;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The widget that was constructed by the component.
    type Root: Clone + AsRef<gtk4::Widget>;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    fn init_inner(
        params: Self::InitParams,
        root_widget: &Self::Root,
        input: Sender<Self::Input>,
        output: Sender<Self::Output>,
    ) -> ComponentInner<Self, Self::Widgets, Self::Input, Self::Output>;

    /// Initializes the component and attaches it to the default local executor.
    ///
    /// Spawns an event loop on `glib::MainContext::default()`, which exists
    /// for as long as the root widget remains alive.
    fn init(params: Self::InitParams) -> Registered<Self::Root, Self::Input, Self::Output> {
        let (sender, in_rx) = mpsc::unbounded_channel::<Self::Input>();
        let (out_tx, output) = mpsc::unbounded_channel::<Self::Output>();

        let root = Self::init_root();

        let mut component = Self::init_inner(params, &root, sender, out_tx);

        let handle = Handle {
            widget: root,
            sender: component.input.clone(),
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
                        Self::update(&mut component, event);
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

    /// Handles input messages and enables the programmer to update the model and view.
    #[allow(unused_variables)]
    fn update(
        component: &mut ComponentInner<Self, Self::Widgets, Self::Input, Self::Output>,
        message: Self::Input,
    );
}
