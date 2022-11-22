use cosmic::widget::{expander, nav_bar, nav_bar_page, nav_bar_section};
use cosmic::{
    iced::widget::{
        checkbox, column, container, horizontal_space, pick_list, progress_bar, radio, row, slider,
        text,
    },
    iced::{self, Alignment, Application, Color, Command, Length},
    iced_lazy::responsive,
    iced_native::window,
    iced_winit::window::{drag, maximize, minimize},
    list_view, list_view_item, list_view_row, list_view_section, scrollable,
    theme::{self, Theme},
    widget::{button, header_bar, list_box, list_row, list_view::*, toggler},
    Element,
};
use std::collections::BTreeMap;
use cosmic::widget::widget::text_input::Id as TextInputId;
use cosmic::widget::widget::text_input;

#[derive(Default)]
pub struct Window {
    page: u8,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
    pick_list_selected: Option<&'static str>,
    sidebar_toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
    exit: bool,
}

impl Window {
    pub fn sidebar_toggled(mut self, toggled: bool) -> Self {
        self.sidebar_toggled = toggled;
        self
    }

    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    pub fn show_minimize(mut self, show: bool) -> Self {
        self.show_minimize = show;
        self
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Message {
    Page(u8),
    Debug(bool),
    ThemeChanged(Theme),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    PickListSelected(&'static str),
    RowSelected(usize),
    Close,
    ToggleSidebar,
    Drag,
    Minimize,
    Maximize,
    InputChanged,
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut window = Window::default()
            .sidebar_toggled(true)
            .show_maximize(true)
            .show_minimize(true);
        window.slider_value = 50.0;
        //        window.theme = Theme::Light;
        window.pick_list_selected = Some("Option 1");
        (window, Command::none())
    }

    fn title(&self) -> String {
        String::from("COSMIC Design System - Iced")
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::Page(page) => self.page = page,
            Message::Debug(debug) => self.debug = debug,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => {
                self.checkbox_value = value;
                return text_input::focus(TextInputId::new("launcher_entry"));
            },
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::Close => self.exit = true,
            Message::ToggleSidebar => self.sidebar_toggled = !self.sidebar_toggled,
            Message::Drag => return drag(window::Id::new(0)),
            Message::Minimize => return minimize(window::Id::new(0), true),
            Message::Maximize => return maximize(window::Id::new(0), true),
            Message::RowSelected(row) => println!("Selected row {row}"),
            Message::InputChanged => {},

        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut header: Element<Message> = header_bar()
            .title(self.title())
            .nav_title(String::from("Settings"))
            .sidebar_active(self.sidebar_toggled)
            .show_minimize(self.show_minimize)
            .show_maximize(self.show_maximize)
            .on_close(Message::Close)
            .on_drag(Message::Drag)
            .on_maximize(Message::Maximize)
            .on_minimize(Message::Minimize)
            .on_sidebar_toggle(Message::ToggleSidebar)
            .into();

        if self.debug {
            header = header.explain(Color::WHITE);
        }

        // TODO: Adding responsive makes this regenerate on every size change, and regeneration
        // involves allocations for many different items. Ideally, we could only make the nav bar
        // responsive and leave the content to be sized normally.
        let content = responsive(|size| {
            let condensed = size.width < 900.0;

            // cosmic::navbar![
            //     nav_button!("network-wireless", "Network & Wireless", condensed)
            //         .on_press(Message::Page(0))
            //         .style(if self.page == 0 {
            //             theme::Button::Primary
            //         } else {
            //             theme::Button::Text
            //         }),
            //     nav_button!("preferences-desktop", "Bluetooth", condensed)
            //         .on_press(Message::Page(1))
            //         .style(if self.page == 1 {
            //             theme::Button::Primary
            //         } else {
            //             theme::Button::Text
            //         }),
            //     nav_button!("system-software-update", "Personalization", condensed)
            //         .on_press(Message::Page(2))
            //         .style(if self.page == 2 {
            //             theme::Button::Primary
            //         } else {
            //             theme::Button::Text
            //         }),
            // ]

            let sidebar: Element<_> = nav_bar()
                .source(BTreeMap::from([
                    (
                        nav_bar_section()
                            .title("Network & Wireless")
                            .icon("network-wireless"),
                        vec![nav_bar_page("Wi-Fi")],
                    ),
                    (
                        nav_bar_section().title("Bluetooth").icon("cs-bluetooth"),
                        vec![nav_bar_page("Devices")],
                    ),
                    (
                        nav_bar_section()
                            .title("Personalization")
                            .icon("applications-system"),
                        vec![
                            nav_bar_page("Desktop Session"),
                            nav_bar_page("Wallpaper"),
                            nav_bar_page("Appearance"),
                            nav_bar_page("Dock & Top Panel"),
                            nav_bar_page("Workspaces"),
                            nav_bar_page("Notifications"),
                        ],
                    ),
                    (
                        nav_bar_section()
                            .title("Input Devices")
                            .icon("input-keyboard"),
                        vec![nav_bar_page("Keyboard")],
                    ),
                    (
                        nav_bar_section().title("Displays").icon("cs-display"),
                        vec![nav_bar_page("Keyboard")],
                    ),
                    (
                        nav_bar_section().title("Power & Battery").icon("battery"),
                        vec![nav_bar_page("Status")],
                    ),
                    (
                        nav_bar_section().title("Sound").icon("sound"),
                        vec![nav_bar_page("Volume")],
                    ),
                ]))
                .active(self.sidebar_toggled)
                .condensed(condensed)
                .into();

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

            let content: Element<_> = list_view!(
                list_view_section!(
                    "Debug",
                    list_view_item!("Debug theme", choose_theme),
                    list_view_item!(
                        "Debug layout",
                        toggler(String::from("Debug layout"), self.debug, Message::Debug,)
                    )
                ),
                list_view_section!(
                    "Buttons",
                    list_view_row!(
                        button!("Primary")
                            .style(theme::Button::Primary)
                            .on_press(Message::ButtonPressed),
                        button!("Secondary")
                            .style(theme::Button::Secondary)
                            .on_press(Message::ButtonPressed),
                        button!("Positive")
                            .style(theme::Button::Positive)
                            .on_press(Message::ButtonPressed),
                        button!("Destructive")
                            .style(theme::Button::Destructive)
                            .on_press(Message::ButtonPressed),
                        button!("Text")
                            .style(theme::Button::Text)
                            .on_press(Message::ButtonPressed),
                    ),
                    list_view_row!(
                        button!("Primary").style(theme::Button::Primary),
                        button!("Secondary").style(theme::Button::Secondary),
                        button!("Positive").style(theme::Button::Positive),
                        button!("Destructive").style(theme::Button::Destructive),
                        button!("Text").style(theme::Button::Text),
                    ),
                ),
                list_view_section!(
                    "Controls",
                    list_view_item!(
                        "Toggler",
                        toggler(None, self.toggler_value, Message::TogglerToggled)
                    ),
                    list_view_item!(
                        "Pick List (TODO)",
                        pick_list(
                            vec!["Option 1", "Option 2", "Option 3", "Option 4",],
                            self.pick_list_selected,
                            Message::PickListSelected
                        )
                        .padding([8, 0, 8, 16])
                    ),
                    list_view_item!(
                        "Slider",
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                            .width(Length::Units(250))
                    ),
                    list_view_item!(
                        "Progress",
                        progress_bar(0.0..=100.0, self.slider_value)
                            .width(Length::Units(250))
                            .height(Length::Units(4))
                    ),
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled),
                    text_input(
                        "Type something...",
                        "",
                        |_| Message::InputChanged,
                    )
                    .padding(8)
                    .size(20)
                    .id(TextInputId::new("launcher_entry"))
                ),
                list_view_section!(
                    "Expander",
                    expander()
                        .title("Label")
                        .subtitle("Caption")
                        .icon(String::from("edit-paste"))
                        .on_row_selected(Box::new(Message::RowSelected))
                        .rows(vec![
                            list_row()
                                .title("Label")
                                .subtitle("Caption")
                                .icon(String::from("help-about")),
                            list_row().subtitle("Caption").title("Label"),
                            list_row().title("Label")
                        ])
                ),
                list_view_section!(
                    "List Box",
                    list_box()
                        .style(theme::Container::Custom(list_section_style))
                        .children(vec![
                            cosmic::list_box_row!("Title").into(),
                            cosmic::list_box_row!("Title", "Subtitle").into(),
                            cosmic::list_box_row!("Title", "", "edit-paste").into(),
                            cosmic::list_box_row!("", "Subtitle", "edit-paste").into(),
                            cosmic::list_box_row!("Title", "Subtitle", "edit-paste").into()
                        ])
                        .render()
                ),
            )
            .into();

            let mut widgets = Vec::with_capacity(2);

            widgets.push(if self.debug {
                sidebar.explain(Color::WHITE)
            } else {
                sidebar
            });

            widgets.push(
                scrollable!(row![
                    horizontal_space(Length::Fill),
                    if self.debug {
                        content.explain(Color::WHITE)
                    } else {
                        content
                    },
                    horizontal_space(Length::Fill),
                ])
                .into(),
            );

            container(row(widgets))
                .padding([16, 16])
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        })
        .into();

        column(vec![header, content]).into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
