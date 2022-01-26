use super::*;

#[async_trait::async_trait]
/// Elm-style variant of a Component with view updates separated from input updates
pub trait ElmComponent: Sized + 'static {
    /// The arguments that are passed to the init_view method.
    type InitParams;

    /// The message type that the component accepts as inputs.
    type Input: 'static + Send;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// Internal commands to perform
    type Command: 'static + Send;

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
        let (out_tx, out_rx) = mpsc::unbounded_channel::<Self::Output>();

        let root = Self::init_root();

        let ComponentInner {
            mut model,
            mut widgets,
            mut input,
            mut output,
        } = Self::init_inner(params, &root, sender, out_tx);

        let handle = Handle {
            widget: root,
            sender: input.clone(),
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
                        if let Some(command) = model.update(event, &mut input, &mut output) {
                            let input = input.clone();
                            tokio::spawn(async move {
                                if let Some(event) = Self::command(command).await {
                                    let _ = input.send(event);
                                }
                            });
                        }

                        model.update_view(&mut widgets, &mut input, &mut output);
                    }

                    InnerMessage::Drop => break,
                }
            }
        });

        forward(in_rx, inner_tx, InnerMessage::Message);

        Registered {
            handle,
            receiver: out_rx,
        }
    }

    /// Handles input messages and enables the programmer to update the model and view.
    fn update(
        &mut self,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command>;

    /// Update the UI
    fn update_view(
        &mut self,
        widgets: &mut Self::Widgets,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    );

    /// A command to perform in a background thread.
    async fn command(message: Self::Command) -> Option<Self::Input>;
}
