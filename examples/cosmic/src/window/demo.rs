use apply::Apply;
use cosmic::{
    iced::widget::{checkbox, pick_list, progress_bar, radio, row, slider},
    iced::{widget::container, Alignment, Length},
    theme::{Button as ButtonTheme, Theme},
    widget::{
        button,
        segmented_button::{
            self,
            cosmic::{
                horizontal_segmented_selection, horizontal_view_switcher,
                vertical_segmented_selection, vertical_view_switcher,
            },
        },
        settings,
        spin_button::{SpinButtonModel, SpinMessage},
        toggler,
    },
    Element,
};

use super::{Page, Window};

pub enum DemoView {
    TabA,
    TabB,
    TabC,
}

#[allow(dead_code)]
pub enum MultiOption {
    OptionA,
    OptionB,
    OptionC,
    OptionD,
    OptionE,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    ButtonPressed,
    CheckboxToggled(bool),
    Debug(bool),
    IconTheme(segmented_button::Key),
    MultiSelection(segmented_button::Key),
    PickListSelected(&'static str),
    RowSelected(usize),
    Selection(segmented_button::Key),
    SliderChanged(f32),
    SpinButton(SpinMessage),
    ThemeChanged(Theme),
    TogglerToggled(bool),
    ViewSwitcher(segmented_button::Key),
}

pub enum Output {
    Debug(bool),
    ThemeChanged(Theme),
}

pub struct State {
    pub checkbox_value: bool,
    pub icon_theme: segmented_button::SingleSelectModel<&'static str>,
    pub multi_selection: segmented_button::MultiSelectModel<MultiOption>,
    pub pick_list_selected: Option<&'static str>,
    pub selection: segmented_button::SingleSelectModel<()>,
    pub slider_value: f32,
    pub spin_button: SpinButtonModel<i32>,
    pub toggler_value: bool,
    pub view_switcher: segmented_button::SingleSelectModel<DemoView>,
}

impl Default for State {
    fn default() -> State {
        State {
            checkbox_value: false,
            pick_list_selected: Some("Option 1"),
            slider_value: 50.0,
            spin_button: SpinButtonModel::default().min(-10).max(10),
            toggler_value: false,
            icon_theme: segmented_button::Model::builder()
                .insert_active("Pop", "Pop")
                .insert("Adwaita", "Adwaita")
                .build(),
            selection: segmented_button::Model::builder()
                .insert_active("Choice A", ())
                .insert("Choice B", ())
                .insert("Choice C", ())
                .build(),
            multi_selection: segmented_button::Model::builder()
                .insert_active("Option A", MultiOption::OptionA)
                .insert("Option B", MultiOption::OptionB)
                .insert("Option C", MultiOption::OptionB)
                .insert("Option D", MultiOption::OptionC)
                .insert("Option E", MultiOption::OptionE)
                .build(),
            view_switcher: segmented_button::Model::builder()
                .insert_active("Controls", DemoView::TabA)
                .insert("Segmented Button", DemoView::TabB)
                .insert("Tab C", DemoView::TabC)
                .build(),
        }
    }
}

impl State {
    pub(super) fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::ButtonPressed => (),
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::Debug(value) => return Some(Output::Debug(value)),
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::RowSelected(row) => println!("Selected row {row}"),
            Message::MultiSelection(key) => self.multi_selection.activate(key),
            Message::Selection(key) => self.selection.activate(key),
            Message::SliderChanged(value) => self.slider_value = value,
            Message::SpinButton(msg) => self.spin_button.update(msg),
            Message::ThemeChanged(theme) => return Some(Output::ThemeChanged(theme)),
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::ViewSwitcher(key) => self.view_switcher.activate(key),
            Message::IconTheme(key) => {
                self.icon_theme.activate(key);
                if let Some(theme) = self.icon_theme.component(key) {
                    cosmic::settings::set_default_icon_theme(*theme);
                }
            }
        }

        None
    }

    pub(super) fn view<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        let choose_theme = [Theme::Light, Theme::Dark].iter().fold(
            row![].spacing(10).align_items(Alignment::Center),
            |row, theme| {
                row.push(radio(
                    format!("{:?}", theme),
                    *theme,
                    Some(window.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        let choose_icon_theme =
            horizontal_segmented_selection(&self.icon_theme).on_activate(Message::IconTheme);

        settings::view_column(vec![
            window.page_title(Page::Demo),
            horizontal_view_switcher(&self.view_switcher)
                .on_activate(Message::ViewSwitcher)
                .into(),
            match self.view_switcher.active_component() {
                None => panic!("no tab is active"),
                Some(DemoView::TabA) => settings::view_column(vec![
                    settings::view_section("Debug")
                        .add(settings::item("Debug theme", choose_theme))
                        .add(settings::item("Debug icon theme", choose_icon_theme))
                        .add(settings::item(
                            "Debug layout",
                            toggler(None, window.debug, Message::Debug),
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
                Some(DemoView::TabB) => settings::view_column(vec![
                    cosmic::iced::widget::text("Selection")
                        .font(cosmic::font::FONT_SEMIBOLD)
                        .into(),
                    cosmic::iced::widget::text("Horizontal").into(),
                    horizontal_segmented_selection(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Horizontal With Spacing").into(),
                    horizontal_segmented_selection(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Horizontal Multi-Select").into(),
                    horizontal_segmented_selection(&self.multi_selection)
                        .spacing(8)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    cosmic::iced::widget::text("Vertical").into(),
                    vertical_segmented_selection(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Vertical Multi-Select Shrunk").into(),
                    vertical_segmented_selection(&self.multi_selection)
                        .width(Length::Shrink)
                        .on_activate(Message::MultiSelection)
                        .apply(container)
                        .center_x()
                        .width(Length::Fill)
                        .into(),
                    cosmic::iced::widget::text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        vertical_segmented_selection(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        vertical_segmented_selection(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        vertical_segmented_selection(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                    ])
                    .spacing(12)
                    .width(Length::Fill)
                    .into(),
                    cosmic::iced::widget::text("View Switcher")
                        .font(cosmic::font::FONT_SEMIBOLD)
                        .into(),
                    cosmic::iced::widget::text("Horizontal").into(),
                    horizontal_view_switcher(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Horizontal Multi-Select").into(),
                    horizontal_view_switcher(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    cosmic::iced::widget::text("Horizontal With Spacing").into(),
                    horizontal_view_switcher(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Vertical").into(),
                    vertical_view_switcher(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    cosmic::iced::widget::text("Vertical Multi-Select").into(),
                    vertical_view_switcher(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    cosmic::iced::widget::text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        vertical_view_switcher(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        vertical_view_switcher(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        vertical_view_switcher(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                    ])
                    .spacing(12)
                    .width(Length::Fill)
                    .into(),
                ])
                .padding(0)
                .into(),
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
