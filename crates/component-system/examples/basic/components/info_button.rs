// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use ccs::*;
use gtk::prelude::*;

pub enum InfoButtonInput {
    SetDescription(String),
}

pub enum InfoButtonOutput {
    Clicked,
}

component! {
    #[derive(Default)]
    pub struct InfoButton((String, String, gtk::SizeGroup)) {

    }

    pub struct InfoButtonWidgets(gtk::Box) {
        description: gtk::Label,
    }

    type Input = InfoButtonInput;
    type Output = InfoButtonOutput;

    fn init_view(self, args, _input, output) {
        let (desc, button_label, sg) = args;
        ccs::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_start: 20,
                set_margin_end: 20,
                set_margin_top: 8,
                set_margin_bottom: 8,
                set_spacing: 24,

                append: description = &gtk::Label {
                    set_label: &desc,
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                },

                append: button = &gtk::Button {
                    set_label: &button_label,

                    connect_clicked(output) => move |_| {
                        let _ = output.send(InfoButtonOutput::Clicked);
                    }
                }
            }
        }

        sg.add_widget(&button);

        (InfoButtonWidgets { description }, root)
    }

    fn update(self, widgets, message, _input, _output) {
        match message {
            InfoButtonInput::SetDescription(value) => {
                widgets.description.set_text(&value);
            }
        }
    }
}
