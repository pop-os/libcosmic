use cosmic::{
    iced::widget::{checkbox, pick_list, progress_bar, radio, row, slider},
    iced::{Alignment, Length},
    theme::{self, Button as ButtonTheme, Theme},
    widget::{button, segmented_button::cosmic::{view_switcher, segmented_selection}, settings, toggler},
    Element,
};

use super::{Message, Page, Window};

pub enum DemoView {
    TabA,
    TabB,
    TabC,
}

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
            view_switcher(&self.demo_view_switcher)
                .on_activate(Message::DemoTabActivate)
                .into(),
            match self.demo_view_switcher.active_data() {
                None => panic!("no tab is active"),
                Some(DemoView::TabA) => settings::view_column(vec![
                    settings::view_section("Debug")
                        .add(settings::item("Debug theme", choose_theme))
                        .add(settings::item(
                            "Debug layout",
                            toggler(None, self.debug, Message::Debug),
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
                                .into(),
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
                        .add(settings::item(
                            "Toggler",
                            toggler(None, self.toggler_value, Message::TogglerToggled),
                        ))
                        .add(settings::item(
                            "Pick List (TODO)",
                            pick_list(
                                vec!["Option 1", "Option 2", "Option 3", "Option 4"],
                                self.pick_list_selected,
                                Message::PickListSelected,
                            )
                            .padding([8, 0, 8, 16]),
                        ))
                        .add(settings::item(
                            "Slider",
                            slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                                .width(Length::Units(250)),
                        ))
                        .add(settings::item(
                            "Progress",
                            progress_bar(0.0..=100.0, self.slider_value)
                                .width(Length::Units(250))
                                .height(Length::Units(4)),
                        ))
                        .add(settings::item_row(vec![checkbox(
                            "Checkbox",
                            self.checkbox_value,
                            Message::CheckboxToggled,
                        )
                        .into()]))
                        .add(settings::item(
                            format!(
                                "Spin Button (Range {}:{})",
                                self.spin_button.min, self.spin_button.max
                            ),
                            self.spin_button.view(Message::SpinButton),
                        ))
                        .into(),
                ])
                .padding(0)
                .into(),
                Some(DemoView::TabB) => {
                    settings::view_column(vec![
                        cosmic::iced::widget::text("SegmentedButton::Selection")
                            .font(cosmic::font::FONT_SEMIBOLD)
                            .into(),
                        segmented_selection(&self.demo_selection)
                            .on_activate(Message::DemoSelectionActivate)
                            .into()
                    ])
                    .padding(0)
                    .into()
                }
                Some(DemoView::TabC) => {
                    settings::view_column(vec![settings::view_section("Tab C")
                        .add(cosmic::iced::widget::text("Nothing here yet").width(Length::Fill))
                        .into()])
                    .padding(0)
                    .into()
                }
            },
        ])
        .into()
    }
}
