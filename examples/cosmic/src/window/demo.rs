use apply::Apply;
use cosmic::{
    cosmic_theme,
    iced::widget::{checkbox, column, pick_list, progress_bar, radio, slider, text, text_input},
    iced::{id, Alignment, Length},
    theme::{self, Button as ButtonTheme, Theme, ThemeType},
    widget::{
        button, container, icon, segmented_button, segmented_selection, settings, spin_button,
        toggler, view_switcher,
    },
    Element,
};
use fraction::{Decimal, ToPrimitive};
use once_cell::sync::Lazy;

use super::{Page, Window};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ThemeVariant {
    Light,
    Dark,
    HighContrastDark,
    HighContrastLight,
    Custom,
}

impl From<&ThemeType> for ThemeVariant {
    fn from(theme: &ThemeType) -> Self {
        match theme {
            ThemeType::Light => ThemeVariant::Light,
            ThemeType::Dark => ThemeVariant::Dark,
            ThemeType::HighContrastDark => ThemeVariant::HighContrastDark,
            ThemeType::HighContrastLight => ThemeVariant::HighContrastLight,
            ThemeType::Custom(_) => ThemeVariant::Custom,
        }
    }
}

impl From<ThemeType> for ThemeVariant {
    fn from(theme: ThemeType) -> Self {
        ThemeVariant::from(&theme)
    }
}

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
static INPUT_ID: Lazy<id::Id> = Lazy::new(id::Id::unique);

#[derive(Clone, Debug)]
pub enum Message {
    ButtonPressed,
    CheckboxToggled(bool),
    Debug(bool),
    IconTheme(segmented_button::Entity),
    MultiSelection(segmented_button::Entity),
    PickListSelected(&'static str),
    RowSelected(usize),
    ScalingFactor(spin_button::Message),
    Selection(segmented_button::Entity),
    SliderChanged(f32),
    SpinButton(spin_button::Message),
    ThemeChanged(ThemeVariant),
    ToggleWarning,
    TogglerToggled(bool),
    ViewSwitcher(segmented_button::Entity),
    InputChanged(String),
}

pub enum Output {
    Debug(bool),
    ScalingFactor(f32),
    ThemeChanged(ThemeVariant),
    ToggleWarning,
}

pub struct State {
    pub checkbox_value: bool,
    pub icon_themes: segmented_button::SingleSelectModel,
    pub multi_selection: segmented_button::MultiSelectModel,
    pub pick_list_selected: Option<&'static str>,
    pub pick_list_options: Vec<&'static str>,
    pub scaling_value: spin_button::Model<Decimal>,
    pub selection: segmented_button::SingleSelectModel,
    pub slider_value: f32,
    pub spin_button: spin_button::Model<i32>,
    pub toggler_value: bool,
    pub view_switcher: segmented_button::SingleSelectModel,
    pub entry_value: String,
}

impl Default for State {
    fn default() -> State {
        State {
            checkbox_value: false,
            pick_list_selected: Some("Option 1"),
            pick_list_options: vec!["Option 1", "Option 2", "Option 3", "Option 4"],
            scaling_value: spin_button::Model::default()
                .value(1.0)
                .min(0.5)
                .max(2.0)
                .step(0.25),
            slider_value: 50.0,
            spin_button: spin_button::Model::default().min(-10).max(10),
            toggler_value: false,
            icon_themes: segmented_button::Model::builder()
                .insert(|b| b.text("Pop").activate())
                .insert(|b| b.text("Adwaita"))
                .build(),
            selection: segmented_button::Model::builder()
                .insert(|b| b.text("Choice A").activate())
                .insert(|b| b.text("Choice B"))
                .insert(|b| b.text("Choice C"))
                .build(),
            multi_selection: segmented_button::Model::builder()
                .insert(|b| b.text("Option A").data(MultiOption::OptionA).activate())
                .insert(|b| b.text("Option B").data(MultiOption::OptionB))
                .insert(|b| b.text("Option C").data(MultiOption::OptionC))
                .insert(|b| b.text("Option D").data(MultiOption::OptionD))
                .insert(|b| b.text("Option E").data(MultiOption::OptionE))
                .build(),
            view_switcher: segmented_button::Model::builder()
                .insert(|b| b.text("Controls").data(DemoView::TabA).activate())
                .insert(|b| b.text("Segmented Button").data(DemoView::TabB))
                .insert(|b| b.text("Tab C").data(DemoView::TabC))
                .build(),
            entry_value: String::new(),
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
            Message::ScalingFactor(message) => {
                self.scaling_value.update(message);
                if let Some(factor) = self.scaling_value.value.to_f32() {
                    return Some(Output::ScalingFactor(factor));
                }
            }
            Message::Selection(key) => self.selection.activate(key),
            Message::SliderChanged(value) => self.slider_value = value,
            Message::SpinButton(msg) => self.spin_button.update(msg),
            Message::ThemeChanged(theme) => return Some(Output::ThemeChanged(theme)),
            Message::ToggleWarning => return Some(Output::ToggleWarning),
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::ViewSwitcher(key) => self.view_switcher.activate(key),
            Message::IconTheme(key) => {
                self.icon_themes.activate(key);
                if let Some(theme) = self.icon_themes.text(key) {
                    cosmic::settings::set_default_icon_theme(theme);
                }
            }
            Message::InputChanged(s) => {
                self.entry_value = s;
            }
        }

        None
    }

    pub(super) fn view<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        let choose_theme = [
            ThemeVariant::Light,
            ThemeVariant::Dark,
            ThemeVariant::HighContrastLight,
            ThemeVariant::HighContrastLight,
            ThemeVariant::Custom,
        ]
        .into_iter()
        .fold(
            column![].spacing(10).align_items(Alignment::Center),
            |row, theme| {
                row.push(radio(
                    format!("{:?}", theme),
                    theme,
                    if ThemeVariant::from(&window.theme.theme_type) == theme {
                        Some(theme)
                    } else {
                        None
                    },
                    Message::ThemeChanged,
                ))
            },
        );

        let choose_icon_theme =
            segmented_selection::horizontal(&self.icon_themes).on_activate(Message::IconTheme);

        settings::view_column(vec![
            window.page_title(Page::Demo),
            view_switcher::horizontal(&self.view_switcher)
                .on_activate(Message::ViewSwitcher)
                .into(),
            match self.view_switcher.active_data() {
                None => panic!("no tab is active"),
                Some(DemoView::TabA) => settings::view_column(vec![
                    settings::view_section("Debug")
                        .add(settings::item("Debug theme", choose_theme))
                        .add(settings::item("Debug icon theme", choose_icon_theme))
                        .add(settings::item(
                            "Debug layout",
                            toggler(None, window.debug, Message::Debug),
                        ))
                        .add(settings::item(
                            "Scaling Factor",
                            spin_button(&window.scale_factor_string, Message::ScalingFactor),
                        ))
                        .add(settings::item_row(vec![button(ButtonTheme::Destructive)
                            .on_press(Message::ToggleWarning)
                            .custom(vec![
                                icon("dialog-warning-symbolic", 16)
                                    .style(theme::Svg::SymbolicPrimary)
                                    .into(),
                                text("Do Not Touch").into(),
                            ])
                            .into()]))
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
                                &self.pick_list_options,
                                self.pick_list_selected,
                                Message::PickListSelected,
                            )
                            .padding([8, 0, 8, 16]),
                        ))
                        .add(settings::item(
                            "Slider",
                            slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                                .width(Length::Fixed(250.0))
                                .height(38),
                        ))
                        .add(settings::item(
                            "Progress",
                            progress_bar(0.0..=100.0, self.slider_value)
                                .width(Length::Fixed(250.0))
                                .height(Length::Fixed(4.0)),
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
                            spin_button(format!("{}", self.spin_button.value), Message::SpinButton),
                        ))
                        .into(),
                ])
                .padding(0)
                .into(),
                Some(DemoView::TabB) => settings::view_column(vec![
                    text("Selection").font(cosmic::font::FONT_SEMIBOLD).into(),
                    text("Horizontal").into(),
                    segmented_selection::horizontal(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Horizontal With Spacing").into(),
                    segmented_selection::horizontal(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Horizontal Multi-Select").into(),
                    segmented_selection::horizontal(&self.multi_selection)
                        .spacing(8)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Vertical").into(),
                    segmented_selection::vertical(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Vertical Multi-Select Shrunk").into(),
                    segmented_selection::vertical(&self.multi_selection)
                        .width(Length::Shrink)
                        .on_activate(Message::MultiSelection)
                        .apply(container)
                        .center_x()
                        .width(Length::Fill)
                        .into(),
                    text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        segmented_selection::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        segmented_selection::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        segmented_selection::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                    ])
                    .spacing(12)
                    .width(Length::Fill)
                    .into(),
                    text("View Switcher")
                        .font(cosmic::font::FONT_SEMIBOLD)
                        .into(),
                    text("Horizontal").into(),
                    view_switcher::horizontal(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Horizontal Multi-Select").into(),
                    view_switcher::horizontal(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Horizontal With Spacing").into(),
                    view_switcher::horizontal(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Vertical").into(),
                    view_switcher::vertical(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Vertical Multi-Select").into(),
                    view_switcher::vertical(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        view_switcher::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        view_switcher::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        view_switcher::vertical(&self.selection)
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
                        .add(text("Nothing here yet").width(Length::Fill))
                        .into()])
                    .padding(0)
                    .into()
                }
            },
            container(text("Background container with some text").size(24))
                .layer(cosmic_theme::Layer::Background)
                .padding(8)
                .width(Length::Fill)
                .into(),
            container(text("Primary container with some text").size(24))
                .layer(cosmic_theme::Layer::Primary)
                .padding(8)
                .width(Length::Fill)
                .into(),
            container(text("Secondary container with some text").size(24))
                .layer(cosmic_theme::Layer::Secondary)
                .padding(8)
                .width(Length::Fill)
                .into(),
            text_input(
                "Type to search apps or type “?” for more options...",
                &self.entry_value,
            )
            .on_input(Message::InputChanged)
            // .on_submit(Message::Activate(None))
            .padding(8)
            .size(20)
            .id(INPUT_ID.clone())
            .into(),
        ])
        .into()
    }
}
