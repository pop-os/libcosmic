// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::components::{InfoButton, InfoButtonInput, InfoButtonOutput};
use ccs::*;
use gtk::prelude::*;

/// An input event that is used to update the model.
pub enum AppEvent {
    Destroy,
    Increment,
}

component! {
    /// The model where component state is stored.
    #[derive(Default)]
    pub struct App {
        pub counter: usize,
    }

    /// Widgets that are initialized in the view.
    pub struct AppWidgets {
        list: gtk::ListBox,
        destroyable: Option<Controller<gtk::Box, InfoButtonInput>>,
        counter: Controller<gtk::Box, InfoButtonInput>,
    }

    type Input = AppEvent;
    type Output = ();
    type Root = gtk::Box {
        ccs::view! {
            root = gtk::Box {
                set_halign: gtk::Align::Center,
                set_size_request: args!(400, -1),
                set_orientation: gtk::Orientation::Vertical,
            }
        }

        root
    };

    fn init(window: gtk::ApplicationWindow, root, input, _output) {
        let button_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        let destroyable = InfoButton::init()
            .launch_stateful((String::new(), "Destroy".into(), button_group.clone()))
            .forward(input.clone(), |event| match event {
                InfoButtonOutput::Clicked => AppEvent::Destroy,
            });

        // Instruct the component to update its description.
        let _ = destroyable.emit(InfoButtonInput::SetDescription(
            "Click this button to destroy me".into(),
        ));

        // Create a counter component, too.
        let counter = InfoButton::init()
            .launch_stateful(("Click me too".into(), "Click".into(), button_group))
            .forward(input.clone(), |event| match event {
                InfoButtonOutput::Clicked => AppEvent::Increment,
            });

        ccs::view! {
            list = gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::None,
                set_hexpand: true,

                append: &destroyable.widget,
                append: &counter.widget,
            }
        }

        root.append(&list);
        window.set_child(Some(root));

        Fuselage {
            model: Self::default(),
            widgets: AppWidgets { list, destroyable: Some(destroyable), counter },
        }
    }

    /// Updates the view
    fn update(&mut self, widgets, event, _input, _output) {
        match event {
            AppEvent::Increment => {
                self.counter += 1;

                widgets
                    .counter
                    .emit(InfoButtonInput::SetDescription(format!(
                        "Clicked {} times",
                        self.counter
                    )));
            }

            AppEvent::Destroy => {
                // Components are kept alive by their root GTK widget.
                if let Some(handle) = widgets.destroyable.take() {
                    if let Some(parent) = handle.widget.parent() {
                        widgets.list.remove(&parent);
                    }
                }
            }
        }

        None
    }

    async fn command(_message: (), _input) {

    }
}
