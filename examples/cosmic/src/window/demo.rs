use cosmic::{
    Element,
    iced::{Alignment, Length},
    iced::widget::{checkbox, pick_list, progress_bar, radio, row, slider, toggler},
    widget::{button, settings},
    theme::{Button as ButtonTheme, Theme},
};

use super::{Message, Page, Window};

impl Window {
    pub(super) fn view_demo(&self) -> Element<Message> {
        let choose_theme = [Theme::Light, Theme::Dark].iter().fold(
            row![].spacing(10).align_items(Alignment::Center),
            |row, theme| {
                row.push(radio(
                    format!("{:?}", theme),
                    *theme,
                    Some(self.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        settings::view_column(vec![
            self.page_title(Page::Demo),

            settings::view_section("Debug")
                .add(settings::item("Debug theme", choose_theme))
                .add(settings::item(
                    "Debug layout",
                    toggler(String::from("Debug layout"), self.debug, Message::Debug)
                ))
                .into(),

            settings::view_section("Buttons")
                .add(settings::item_row(vec![
                    button(ButtonTheme::Primary)
                        .text("Primary")
                        .on_press(Message::ButtonPressed)
                        .into(),
                    button(ButtonTheme::Secondary)
                        .text("Secondary")
                        .on_press(Message::ButtonPressed)
                        .into(),
                    button(ButtonTheme::Positive)
                        .text("Positive")
                        .on_press(Message::ButtonPressed)
                        .into(),
                    button(ButtonTheme::Destructive)
                        .text("Destructive")
                        .on_press(Message::ButtonPressed)
                        .into(),
                    button(ButtonTheme::Text)
                        .text("Text")
                        .on_press(Message::ButtonPressed)
                        .into()
                ]))
                .add(settings::item_row(vec![
                    button(ButtonTheme::Primary).text("Primary").into(),
                    button(ButtonTheme::Secondary).text("Secondary").into(),
                    button(ButtonTheme::Positive).text("Positive").into(),
                    button(ButtonTheme::Destructive).text("Destructive").into(),
                    button(ButtonTheme::Text).text("Text").into(),
                ]))
                .into(),

            settings::view_section("Controls")
                .add(settings::item("Toggler", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item(
                    "Pick List (TODO)",
                    pick_list(
                        vec!["Option 1", "Option 2", "Option 3", "Option 4",],
                        self.pick_list_selected,
                        Message::PickListSelected
                    )
                    .padding([8, 0, 8, 16])
                ))
                .add(settings::item(
                    "Slider",
                    slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                        .width(Length::Units(250))
                ))
                .add(settings::item(
                    "Progress",
                    progress_bar(0.0..=100.0, self.slider_value)
                        .width(Length::Units(250))
                        .height(Length::Units(4))
                ))
                .add(settings::item_row(vec![
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled).into()
                ]))
                .add(settings::item(
                    format!("Spin Button (Range {}:{})", self.spin_button.min, self.spin_button.max),
                    self.spin_button.view(Message::SpinButton),
                ))
                .into()
        ])
        .into()
    }
}
