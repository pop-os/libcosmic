use std::{cell::RefCell, rc::Rc};

use apply::Apply;
use cosmic::{
    cosmic_theme,
    iced::widget::{checkbox, column, progress_bar, radio, slider, text},
    iced::{Alignment, Length},
    iced_core::id,
    theme::ThemeType,
    widget::{
        button, color_picker::ColorPickerUpdate, dropdown, icon, layer_container as container,
        segmented_button, segmented_control, settings, spin_button, tab_bar, toggler,
        ColorPickerModel,
    },
    Element,
};
use cosmic_time::{anim, chain, Timeline};
use fraction::{Decimal, ToPrimitive};
use once_cell::sync::Lazy;

use super::{Page, Window};

static CARDS: Lazy<cosmic_time::id::Cards> = Lazy::new(cosmic_time::id::Cards::unique);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ThemeVariant {
    Light,
    Dark,
    HighContrastDark,
    HighContrastLight,
    Custom,
    System,
}

impl From<&ThemeType> for ThemeVariant {
    fn from(theme: &ThemeType) -> Self {
        match theme {
            ThemeType::Light => ThemeVariant::Light,
            ThemeType::Dark => ThemeVariant::Dark,
            ThemeType::HighContrastDark => ThemeVariant::HighContrastDark,
            ThemeType::HighContrastLight => ThemeVariant::HighContrastLight,
            ThemeType::Custom(_) => ThemeVariant::Custom,
            ThemeType::System { .. } => ThemeVariant::System,
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
    DropdownSelect(usize),
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
    DeleteCard(usize),
    ClearAll,
    CardsToggled(bool),
    ColorPickerUpdate(ColorPickerUpdate),
    Hidden,
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
    pub dropdown_selected: Option<usize>,
    pub dropdown_options: Vec<&'static str>,
    pub scaling_value: spin_button::Model<Decimal>,
    pub selection: segmented_button::SingleSelectModel,
    pub slider_value: f32,
    pub spin_button: spin_button::Model<i32>,
    pub toggler_value: bool,
    pub tab_bar: segmented_button::SingleSelectModel,
    pub entry_value: String,
    pub cards_value: bool,
    cards: Vec<String>,
    pub timeline: Rc<RefCell<Timeline>>,
    pub color_picker_model: ColorPickerModel,
    pub hidden: bool,
}

impl Default for State {
    fn default() -> State {
        State {
            checkbox_value: false,
            dropdown_selected: Some(0),
            dropdown_options: vec!["Option 1", "Option 2", "Option 3", "Option 4"],
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
            tab_bar: segmented_button::Model::builder()
                .insert(|b| b.text("Controls").data(DemoView::TabA).activate())
                .insert(|b| b.text("Segmented Button").data(DemoView::TabB))
                .insert(|b| b.text("Tab C").data(DemoView::TabC))
                .build(),
            cards_value: false,
            entry_value: String::new(),
            cards: vec![
                "card 1".to_string(),
                "card 2".to_string(),
                "card 3".to_string(),
                "card 4".to_string(),
            ],
            timeline: Rc::new(RefCell::new(Default::default())),
            color_picker_model: ColorPickerModel::new("Hex", "RGB", None, None),
            hidden: false,
        }
    }
}

impl State {
    pub(super) fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::ButtonPressed => (),
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::Debug(value) => return Some(Output::Debug(value)),
            Message::DropdownSelect(value) => self.dropdown_selected = Some(value),
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
            Message::ViewSwitcher(key) => self.tab_bar.activate(key),
            Message::IconTheme(key) => {
                self.icon_themes.activate(key);
                if let Some(theme) = self.icon_themes.text(key) {
                    cosmic::icon_theme::set_default(theme.to_owned());
                }
            }
            Message::InputChanged(s) => {
                self.entry_value = s;
            }
            Message::ClearAll => {
                self.cards.clear();
            }
            Message::CardsToggled(v) => {
                self.cards_value = v;
                self.update_cards();
            }
            Message::DeleteCard(i) => {
                self.cards.remove(i);
            }
            Message::ColorPickerUpdate(u) => {
                _ = self.color_picker_model.update::<Message>(u);
            }
            Message::Hidden => {
                self.hidden = !self.hidden;
            }
        }

        None
    }

    pub(super) fn view<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        let choose_theme = [
            ThemeVariant::Light,
            ThemeVariant::Dark,
            ThemeVariant::HighContrastLight,
            ThemeVariant::HighContrastDark,
            ThemeVariant::Custom,
            ThemeVariant::System,
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
            segmented_control::horizontal(&self.icon_themes).on_activate(Message::IconTheme);
        let timeline = self.timeline.borrow();
        settings::view_column(vec![
            window.page_title(Page::Demo),
            tab_bar::horizontal(&self.tab_bar)
                .on_activate(Message::ViewSwitcher)
                .into(),
            match self.tab_bar.active_data() {
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
                        .add(settings::item_row(vec![
                            cosmic::widget::button::destructive("Do not Touch")
                                .trailing_icon(icon::from_name("dialog-warning-symbolic").size(16))
                                .on_press(Message::ToggleWarning)
                                .into(),
                        ]))
                        .into(),
                    settings::view_section("Controls")
                        .add(settings::item(
                            "Toggler",
                            toggler(None, self.toggler_value, Message::TogglerToggled),
                        ))
                        .add(settings::item(
                            "Pick List (TODO)",
                            dropdown(
                                &self.dropdown_options,
                                self.dropdown_selected,
                                Message::DropdownSelect,
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
                    text("Selection").font(cosmic::font::semibold()).into(),
                    text("Horizontal").into(),
                    segmented_control::horizontal(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Horizontal With Spacing").into(),
                    segmented_control::horizontal(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Disabled Horizontal With Spacing").into(),
                    segmented_control::horizontal(&self.selection)
                        .spacing(8)
                        .into(),
                    text("Horizontal Multi-Select").into(),
                    segmented_control::horizontal(&self.multi_selection)
                        .spacing(8)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Disabled Horizontal Multi-Select").into(),
                    segmented_control::horizontal(&self.multi_selection)
                        .spacing(8)
                        .into(),
                    text("Vertical").into(),
                    segmented_control::vertical(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Disabled Vertical").into(),
                    segmented_control::vertical(&self.selection).into(),
                    text("Vertical Multi-Select Shrunk").into(),
                    segmented_control::vertical(&self.multi_selection)
                        .width(Length::Shrink)
                        .on_activate(Message::MultiSelection)
                        .apply(container)
                        .center_x()
                        .width(Length::Fill)
                        .into(),
                    text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        segmented_control::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        segmented_control::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        segmented_control::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                    ])
                    .spacing(12)
                    .width(Length::Fill)
                    .into(),
                    text("View Switcher").font(cosmic::font::semibold()).into(),
                    text("Horizontal").into(),
                    tab_bar::horizontal(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Horizontal Multi-Select").into(),
                    tab_bar::horizontal(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Horizontal With Spacing").into(),
                    tab_bar::horizontal(&self.selection)
                        .spacing(8)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Vertical").into(),
                    tab_bar::vertical(&self.selection)
                        .on_activate(Message::Selection)
                        .into(),
                    text("Vertical Multi-Select").into(),
                    tab_bar::vertical(&self.multi_selection)
                        .on_activate(Message::MultiSelection)
                        .into(),
                    text("Vertical With Spacing").into(),
                    cosmic::iced::widget::row(vec![
                        tab_bar::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        tab_bar::vertical(&self.selection)
                            .spacing(8)
                            .on_activate(Message::Selection)
                            .width(Length::FillPortion(1))
                            .into(),
                        tab_bar::vertical(&self.selection)
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
            container(column![
                text(
                    "Primary container with some text and a couple icons testing default fallbacks"
                )
                .size(24),
                icon::from_name("microphone-sensitivity-high-symbolic-test")
                    .size(24)
                    .icon(),
                icon::from_name("microphone-sensitivity-high-symbolic-test")
                    .size(24)
                    .fallback(None)
                    .icon(),
            ])
            .layer(cosmic_theme::Layer::Primary)
            .padding(8)
            .width(Length::Fill)
            .into(),
            container(text("Secondary container with some text").size(24))
                .layer(cosmic_theme::Layer::Secondary)
                .padding(8)
                .width(Length::Fill)
                .into(),
            container(anim!(
                //cards
                CARDS,
                &timeline,
                self.cards
                    .iter()
                    .enumerate()
                    .map(|(i, c)| column![
                        button::text("Delete me").on_press(Message::DeleteCard(i)),
                        text(c).size(24).width(Length::Fill)
                    ]
                    .into())
                    .collect(),
                Message::ClearAll,
                |_, e| Message::CardsToggled(e),
                "Show More",
                "Show Less",
                "Clear All",
                None,
                self.cards_value,
            ))
            .layer(cosmic::cosmic_theme::Layer::Secondary)
            .padding(16)
            .class(cosmic::theme::Container::Background)
            .into(),
            cosmic::widget::text_input::secure_input(
                "Type to search apps or type “?” for more options...",
                &self.entry_value,
                Some(Message::Hidden),
                self.hidden,
            )
            .on_input(Message::InputChanged)
            .size(20)
            .id(INPUT_ID.clone())
            .into(),
            cosmic::widget::text_input("", &self.entry_value)
                .label("Test Input")
                .helper_text("test helper text")
                .on_input(Message::InputChanged)
                .size(20)
                .id(INPUT_ID.clone())
                .into(),
            self.color_picker_model
                .picker_button(Message::ColorPickerUpdate, None)
                .width(Length::Fixed(128.0))
                .height(Length::Fixed(128.0))
                .into(),
            if self.color_picker_model.get_is_active() {
                self.color_picker_model
                    .builder(Message::ColorPickerUpdate)
                    .reset_label("Reset to default")
                    .save_label("Save")
                    .cancel_label("Cancel")
                    .build("Recent Colors", "Copy to clipboard", "Copied to clipboard")
                    .into()
            } else {
                text("The color picker is not active.").into()
            },
        ])
        .into()
    }

    fn update_cards(&mut self) {
        let mut timeline = self.timeline.borrow_mut();
        let chain = if self.cards_value {
            chain::Cards::on(CARDS.clone(), 1.)
        } else {
            chain::Cards::off(CARDS.clone(), 1.)
        };
        timeline.set_chain(chain);
        timeline.start();
    }
}
