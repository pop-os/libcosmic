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
        destroyable: Option<Handle<gtk::Box, InfoButtonInput>>,
        counter: Handle<gtk::Box, InfoButtonInput>,
    }

    type Input = AppEvent;
    type Output = ();
    type Root = gtk::ApplicationWindow {
        Default::default()
    };

    fn init(app: gtk::Application, root, input, output) {
        let button_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        let destroyable = InfoButton::init((String::new(), "Destroy".into(), button_group.clone()))
            .forward(input.clone(), |event| match event {
                InfoButtonOutput::Clicked => AppEvent::Destroy,
            });

        // Instruct the component to update its description.
        let _ = destroyable.emit(InfoButtonInput::SetDescription(
            "Click this button to destroy me".into(),
        ));

        // Create a counter component, too.
        let counter = InfoButton::init(("Click me too".into(), "Click".into(), button_group))
            .forward(input.clone(), |event| match event {
                InfoButtonOutput::Clicked => AppEvent::Increment,
            });

        ccs::view! {
            container = gtk::Box {
                set_halign: gtk::Align::Center,
                set_size_request: args!(400, -1),
                set_orientation: gtk::Orientation::Vertical,

                append: list = &gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    set_hexpand: true,

                    append: destroyable.widget(),
                    append: counter.widget(),
                }
            }
        }

        root.set_application(Some(&app));
        root.set_child(Some(&container));

        root.show();

        ComponentInner {
            model: Self::default(),
            widgets: AppWidgets { list, destroyable: Some(destroyable), counter },
            input,
            output
        }
    }

    /// Updates the view
    fn update(component, event) {
        let &mut ComponentInner { ref mut model, ref mut widgets, .. } = component;

        match event {
            AppEvent::Increment => {
                model.counter += 1;

                widgets
                    .counter
                    .emit(InfoButtonInput::SetDescription(format!(
                        "Clicked {} times",
                        model.counter
                    )));
            }

            AppEvent::Destroy => {
                // Components are kept alive by their root GTK widget.
                if let Some(handle) = widgets.destroyable.take() {
                    if let Some(parent) = handle.widget().parent() {
                        widgets.list.remove(&parent);
                    }
                }
            }
        }
    }
}
