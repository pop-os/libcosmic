# COSMIC Component System

This library is a prototyping area for a rewrite of Relm4's component system.

## Using a Macro to Define a Component

The simplest way to define a component is to use the `component!()` macro.

```rs
pub enum MyCustomInputMessage {
    Variant1,
    Variant2,
}

pub enum MyCustomCommand {
    Action
}

component! {
    // The model stores the state of this component.
    pub struct MyCustomModel {
        pub state: String,
    }

    // Widgets managed by the view are stored here.
    pub struct MyCustomWidgets {
        description: gtk::Label,
    }

    // The type of the input sender
    type Input = MyCustomInputMessage;

    // The type of the output sender
    type Output = ();

    // Declares the root widget and how it should be constructed.
    type Root = gtk::Box {
        ccs::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
            }
        }

        root
    };

    // Constructs the inner component's model and widgets, using the
    // initial parameter given by `args`.
    fn init(args: (), root, input, output) {
        let description = gtk::Label::new();

        root.append(&description);

        Fuselage {
            model: MyCustomModel { state: String::new() },
            widgets: MyCustomWidgets { description },
        }
    }

    // Updates the model and view. `self` is the model.
    fn update(&mut self, widgets, event, input, output) {
        match event {
            MyCustomInputMessage::Variant1 => {

            }

            MyCustomInputMessage::Variant2 => {

            }
        }

        Some(MyCustomCommand::Action)
    }

    async fn command(command: MyCustomCommand, input) {

    }
}
```

Components can be created and have their output events forwarded:

```rs
let counter = InfoButton::init()
    .launch("Clicked 0 times".into(), "Click".into())
    .forward(input.clone(), |event| match event {
        InfoButtonOutput::Clicked => AppEvent::Increment
    });
```

The handle returned can be used to emit inputs to it, and to get the root widget.

```rs
counter.emit(InfoButtonInput::SetDescription(format!("Clicked {} times", count)));

box.append(&counter.widget);
```