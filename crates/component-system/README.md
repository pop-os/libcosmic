# COSMIC Component System

This library is a GTK4 GUI framework inspired by [Relm](https://github.com/antoyo/relm), which is inspired by [Elm](https://guide.elm-lang.org/architecture/). The philosophy for this framework is to isolate custom widgets into reusable components. You start with a custom `Model` type that implements `Component`, which is used to register a component with an optional argument. On registration, the model is used to construct the view and its widgets in the `init_view()` function. An event-handler is also spawned to handle events from both the component and any component emitting events to it. Those events are received and handled in the `update()` function. Both the `init_view()` and `update()` methods also have access to an outbound sender, which the caller can forward and consume however desired. See the examples directory for a demonstration of how to create a component.

## Using a Macro to Define a Component

The simplest way to define a component is to use the `component!()` macro.

```rs
pub enum MyCustomInputMessage {
    Variant1,
    Variant2,
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

        ComponentInner {
            model: MyCustomModel { state: String::new() },
            widgets: MyCustomWidgets { description },
            input,
            output
        }
    }

    // Where events are received, with `component` is the `ComponentInner`,
    // and `event` is a `MyCustomInputMessage` which was just received.
    fn update(component, event) {
        match event {
            MyCustomInputMessage::Variant1 => {

            }

            MyCustomInputMessage::Variant2 => {

            }
        }
    }
}
```

Components can be created and have their output events forwarded:

```rs
let counter = InfoButton::init("Clicked 0 times".into(), "Click".into())
    .forward(input.clone(), |event| match event {
        InfoButtonOutput::Clicked => AppEvent::Increment
    });
```

The handle returned can be used to emit inputs to it, and to get the root widget.

```rs
counter.emit(InfoButtonInput::SetDescription(format!("Clicked {} times", count)));

box.append(counter.widget());
```


## See Also

[Relm4](https://github.com/AaronErhardt/relm4) uses a similar approach, but closely follows the Elm model. This library was created as an alternative approach that makes developing reusable components with forwardable events simpler.

## License

Licensed under the [Mozilla Public License 2.0](https://choosealicense.com/licenses/mpl-2.0/).

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0).