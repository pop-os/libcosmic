// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{App, Message};
use cosmic::iced_core::{Alignment, Length};
use cosmic::widget::{
    checkbox, column, container, inline_input, pick_list, progress_bar, row, search_input,
    secure_input, segmented_selection, slider, text, text_input, view_switcher,
};
use cosmic::{Apply, Element};

static PLACEHOLDER_TEXT: &str = "placeholder text";

impl App
where
    Self: cosmic::Application,
{
    pub fn view_text_input(&self) -> Element<Message> {
        column()
            .spacing(24)
            .push(text::title1("Checkbox"))
            .push(
                checkbox(
                    "Checkbox",
                    self.checkbox_value,
                    Message::CheckboxToggled,
                )
            )
            .push(text::title1("Pick List"))
            .push(
                pick_list(
                    &self.pick_list_options,
                    self.pick_list_selected,
                    Message::PickListSelected,
                )
            )
            .push(text::title1("Progress Bar"))
            .push(
                progress_bar(0.0..=100.0, self.slider_value)
                    .width(Length::Fixed(250.0))
                    .height(Length::Fixed(4.0)),
            )
            .push(text::title1("Segmented Buttons"))
            .push(text::title2("Segmented Selection"))
            .push(
                row()
                    .spacing(12)
                    .push(text::body("Horizontal"))
                    .push(
                        segmented_selection::horizontal(&self.selection)
                            .on_activate(Message::Selection),
                    )
            )
            .push(
                row()
                    .spacing(12)
                    .align_items(Alignment::Center)
                    .push(text::body("Vertical"))
                    .push(
                        segmented_selection::vertical(&self.selection)
                            .on_activate(Message::Selection),
                    )
            )

            .push(text::title2("View Switcher"))
            .push(
                row()
                    .spacing(12)
                    .push(text::body("Horizontal"))
                    .push(
                        view_switcher::horizontal(&self.selection)
                            .on_activate(Message::Selection),
                    )
            )
            .push(
                row()
                    .spacing(12)
                    .align_items(Alignment::Center)
                    .push(text::body("Vertical"))
                    .push(
                        view_switcher::vertical(&self.selection)
                            .on_activate(Message::Selection),
                    )
            )
            .push(text::title1("Slider"))
            .push(
                slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                    .width(Length::Fixed(250.0))
                    .height(38)
            )
            .push(text::title1("Spin Button"))
            .push(text::title1("Text Inputs"))
            .push(text::body("Collection of different text input variants."))
            .push(text::title2("Text Input"))
            .push(text::body("The standard text input widget."))
            .push(
                row()
                    .align_items(Alignment::Center)
                    .push(text::body("Enabled"))
                    .spacing(12)
                    .push(
                        text_input(PLACEHOLDER_TEXT, &self.text_input_value)
                            .width(Length::Fill)
                            .on_input(Message::TextInputChanged)
                    )
                    .push(text::body("Disabled"))
                    .push(
                        text_input(PLACEHOLDER_TEXT, &self.text_input_value)
                            .width(Length::Fill)
                    )
            )
            .push(text::title2("Search Input"))
            .push(text::body("Search inputs should be used where search functionality is desired. They differ from the standard text input by displaying a search icon and a clickable search clear button"))
            .push(
                row()
                    .align_items(Alignment::Center)
                    .push(text::body("Enabled"))
                    .spacing(12)
                    .push(
                        search_input(
                            PLACEHOLDER_TEXT,
                            &self.text_input_value,
                            Some(Message::TextInputChanged("".to_string())),
                        )
                        .width(Length::Fill)
                        .on_input(Message::TextInputChanged)
                    )
                    .push(text::body("Disabled"))
                    .push(
                        search_input(
                            "",
                            &self.text_input_value,
                            Some(Message::TextInputChanged("".to_string())),
                        )
                        .width(Length::Fill)
                    )
            )
            .push(text::title2("Secure Input"))
            .push(
                row()
                    .align_items(Alignment::Center)
                    .push(text::body("Enabled"))
                    .spacing(12)
                    .push(
                        secure_input(
                            PLACEHOLDER_TEXT,
                            &self.text_input_value,
                            Some(Message::SecureInputToggled),
                            !self.secure_input_visible,
                        )
                        .label("Test Secure Input Label")
                        .helper_text("Helper Text")
                        .width(Length::Fill)
                        .on_input(Message::TextInputChanged)
                    )
                    .push(text::body("Disabled"))
                    .push(
                        secure_input(
                            "",
                            &self.text_input_value,
                            Some(Message::SecureInputToggled),
                            !self.secure_input_visible,
                        )
                        .label("Test Secure Input Label")
                        .helper_text("Helper Text")
                        .width(Length::Fill)
                    )

            )
            .push(text::title2("Inline Input"))
            .push(text::body("Inline Text Input should be used only inside other widgets, like ListItem or in situations when the input is a quick and transient action (for example, to quickly enter a value in some sort of editing program). "))
            .push(
                row()
                    .align_items(Alignment::Center)
                    .push(text::body("Enabled"))
                    .spacing(12)
                    .push(
                        inline_input(&self.text_input_value)
                            .width(Length::Fill)
                            .on_input(Message::TextInputChanged)
                    )
                    .push(text::body("Disabled"))
                    .push(
                        inline_input(&self.text_input_value)
                            .width(Length::Fill)
                    )
            )
            .push(text::title1("Toggle"))
            .apply(container)
            .max_width(800)
            .into()
    }
}
