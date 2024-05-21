# Commands

[Commands][command] are short-lived async tasks that are spawned onto an async executor on a background thread. They must return a message back to the application upon completion, and cannot directly send messages back to the application until they return.

> **NOTE**: While it is not possible for a command to directly send messages before completion, it is possible to create a subscription from a channel which passes its sender to the application, which may then pass that sender into its commands.

## Future

Commands may be created from futures using [cosmic::command::future](future).

```rs
fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
        Message::Clicked => {
            self.counter += 1;
            self.counter_text = format!("Clicked {} times", self.counter);
            
            // Await for 3 seconds in the background, and then request to decrease the counter.
            return cosmic::command::future(async move {
                tokio::time::sleep(Duration::from_millis(3000)).await;
                Message::Decrease
            });
        }
        
        Message::Decrease =>  {
            self.counter -= 1;
            self.counter_text = format!("Clicked {} times", self.counter);
        }
    }
    
    Command::none()
}
```

## Batches

They can also be [batched][batch] for concurrent execution, where messages will be received in the order of completion.

```rs
fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
        Message::BatchStarted => {
            eprintln!("started handling batch");
        }

        Message::Clicked => {
            self.counter += 1;
            self.counter_text = format!("Clicked {} times", self.counter);
            
            // Run two async tasks concurrently.
            return cosmic::command::batch(vec![
                // Await for 3 seconds in the background, and then request to decrease the counter.
                cosmic::command::future(async move {
                    tokio::time::sleep(Duration::from_millis(3000)).await;
                    Message::Decrease
                }),
                // Immediately returns a message without waiting.
                cosmic::command::message(Message::BatchStarted)
            ]);
        }
        
        Message::Decrease =>  {
            self.counter -= 1;
            self.counter_text = format!("Clicked {} times", self.counter);
        }
    }
    
    Command::none()
}
```

## Widget Operations

They can also be used to perform an operation onto a widget, such as focusing a button or text input.

```rs
return cosmic::widget::button::focus(self.BUTTON_ID);
```

[batch]: https://pop-os.github.io/libcosmic/cosmic/command/fn.batch.html
[command]: https://pop-os.github.io/libcosmic/cosmic/iced_winit/runtime/struct.Command.html
[cosmic-commands]: https://pop-os.github.io/libcosmic/cosmic/app/command/index.html#functions
[future]: https://pop-os.github.io/libcosmic/cosmic/command/fn.future.html