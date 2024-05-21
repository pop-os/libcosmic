# Model-View-Update (MVU)

Iced, and thereby COSMIC, uses a model-view-update approach to GUI development; also known as TEA—[The Elm Architecture][tea]. Similar to Elm, this architecture also emerged naturally in the Rust ecosystem as programmers searched for ways to model applications and services which adhere to Rust's [aliasing XOR mutability rule][aliasing-xor-mutability].

<p align="center">
    <figure>
    <img src="https://book.iced.rs/resources/the-runtime.svg"/>
    <figcaption><a href="https://book.iced.rs/the-runtime.html">The Runtime</a>—from the iced-rs book</figcaption>
    </figure>
</p>

By structuring an application around an event loop which has ownership of a model, each iteration of the loop can immutably borrow the model to create a view, and mutably borrow the model to update the model with received messages. This eliminates the need for shared references, interior mutability, and runtime borrow checking.

> **BACKGROUND**: Before working on COSMIC, the [Pop!_OS][pop-os] team at [System76][system76] was modeling each of their GTK applications with TEA. This would be eventually be formalized into [Relm4][Relm4], which is now the best way to build GTK4 applications in Rust. Event loops are spawned onto the glib runtime for the application and its components. These event loops await messages from a channel, whose senders would be attached to GTK widgets to enable them to publish messages when triggered.

## Model

Every application begins with a struct that implements the [cosmic::Application][app-trait] trait—which will serve as the application's model. All application state will be stored in this model, and it will be wise to cache data that will be needed by your application's widgets.

```rs
struct AppModel {
    counter: u32,
    counter_text: String,
}

impl cosmic::Application for AppModel {}
```

## View

Whenever application or UI state changes; such as the movement of a mouse; the [view method][view-method] will be called to create a view which describes the current state of the UI. The view defines the layout of the interface, how it is to be drawn, and what messages widgets will emit when triggered. The runtime will pass UI events through the view and react upon messages that are emitted.

```rs
fn view(&self) -> Element<Self::Message> {
    let button = widget::button(&self.counter_text)
        .on_press(Message::Clicked);
        
    widget::container(button)
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink)
        .center_x()
        .center_y()
        .into()
}
```

## Update

Messages emitted by the view will later be passed through the application's [update method][update-method]. This will use Rust's pattern matching to choose a branch to execute, make any changes necessary to the application's model, and may optionally return one or more commands.

```rs
fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
        Message::Clicked => {
            self.counter += 1;
            self.counter_text = format!("Clicked {} times", self.counter);
        }
    }
    
    Command::none()
}
```

[aliasing-xor-mutability]: https://cmpt-479-982.github.io/week1/safety_features_of_rust.html#the-borrow-checker-and-the-aliasing-xor-mutability-principle
[app-trait]: https://pop-os.github.io/libcosmic/cosmic/app/trait.Application.html
[pop-os]: https://system76.com/pop
[relm4]: https://github.com/Relm4/relm4
[system76]: https://system76.com/
[tea]: https://guide.elm-lang.org/architecture/
[update-method]: https://pop-os.github.io/libcosmic/cosmic/app/trait.Application.html#method.update
[view-method]: https://pop-os.github.io/libcosmic/cosmic/app/trait.Application.html#tymethod.view
