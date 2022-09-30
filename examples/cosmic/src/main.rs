use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, progress_bar, radio,
    row, slider, svg, text, toggler,
    vertical_space,
};
use iced::{theme, Alignment, Background, Color, Element, Font, Length, Sandbox, Settings, Theme};

const FONT: Font = Font::External {
    name: "Fira Sans Regular",
    bytes: include_bytes!("../res/Fira/Sans/Regular.otf"),
};

const FONT_LIGHT: Font = Font::External {
    name: "Fira Sans Light",
    bytes: include_bytes!("../res/Fira/Sans/Light.otf"),
};

const FONT_SEMIBOLD: Font = Font::External {
    name: "Fira Sans SemiBold",
    bytes: include_bytes!("../res/Fira/Sans/SemiBold.otf"),
};

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.default_font = match FONT {
        Font::Default => None,
        Font::External { bytes, .. } => Some(bytes),
    };
    settings.default_text_size = 18;
    Window::run(settings)
}

fn icon(name: &str, size: u16) -> svg::Svg {
    let handle = match freedesktop_icons::lookup(name)
        .with_size(size)
        .with_theme("Pop")
        .with_cache()
        .force_svg()
        .find()
    {
        Some(path) => svg::Handle::from_path(path),
        None => {
            eprintln!("icon '{}' size {} not found", name, size);
            svg::Handle::from_memory(Vec::new())
        },
    };
    svg::Svg::new(handle)
}

fn sidebar_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
        text_color: None,
        background: Some(Background::Color(
            match theme {
                Theme::Dark => Color::from_rgb8(0x29, 0x29, 0x29),
                Theme::Light => Color::from_rgb8(0xfd, 0xfd, 0xfd),
            }
        )),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}

fn listview_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
        text_color: None,
        background: Some(Background::Color(
            match theme {
                Theme::Dark => Color::from_rgb8(0x27, 0x27, 0x27),
                Theme::Light => Color::from_rgb8(0xf7, 0xf7, 0xf7),
            }
        )),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}

#[derive(Default)]
struct Window {
    page: u8,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Page(u8),
    Debug(bool),
    ThemeChanged(Theme),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
}

impl Sandbox for Window {
    type Message = Message;

    fn new() -> Self {
        let mut window = Window::default();
        window.slider_value = 50.0;
        window
    }

    fn title(&self) -> String {
        String::from("COSMIC Design System - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Page(page) => self.page = page,
            Message::Debug(debug) => self.debug = debug,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
        }
    }

    fn view(&self) -> Element<Message> {
        let sidebar: Element<_> = container(
            column![
                //TODO: Support symbolic icons
                button(
                    row![
                        icon("network-wireless", 16).width(Length::Units(16)),
                        text("Wi-Fi"),
                        horizontal_space(Length::Fill),
                    ]
                    .padding([4, 12])
                    .spacing(8)
                )
                .on_press(Message::Page(0))
                .style(if self.page == 0 { theme::Button::Primary } else { theme::Button::Text })
                ,
                button(
                    row![
                        icon("preferences-desktop", 16).width(Length::Units(16)),
                        text("Desktop"),
                        horizontal_space(Length::Fill),
                    ]
                    .padding([4, 12])
                    .spacing(8)
                )
                .on_press(Message::Page(1))
                .style(if self.page == 1 { theme::Button::Primary } else { theme::Button::Text })
                ,
                button(
                    row![
                        icon("system-software-update", 16).width(Length::Units(16)),
                        text("OS Upgrade & Recovery"),
                        horizontal_space(Length::Fill),
                    ]
                    .padding([4, 12])
                    .spacing(8)
                )
                .on_press(Message::Page(2))
                .style(if self.page == 2 { theme::Button::Primary } else { theme::Button::Text })
                ,
                vertical_space(Length::Fill),
            ]
            .spacing(12)
            .padding(12)
            .max_width(300)
        )
        .style(theme::Container::Custom(sidebar_style))
        .into();

        let choose_theme = [Theme::Light, Theme::Dark].iter().fold(
            row![text("Theme:")].spacing(10).align_items(Alignment::Center),
            |row, theme| {
                row.push(radio(
                    format!("{:?}", theme),
                    *theme,
                    Some(self.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        let content: Element<_> = column![
            choose_theme,
            vertical_space(Length::Units(16)),
            toggler(
                String::from("Debug layout"),
                self.debug,
                Message::Debug,
            )
            .width(Length::Shrink)
            .size(24)
            .spacing(12)
            ,
            vertical_space(Length::Units(16)),
            text("Buttons").font(FONT_SEMIBOLD),
            container(
                column![
                    row![
                        button("Primary")
                        .style(theme::Button::Primary)
                        .padding([8, 16])
                        .on_press(Message::ButtonPressed)
                        ,
                        button("Secondary")
                        .style(theme::Button::Secondary)
                        .padding([8, 16])
                        .on_press(Message::ButtonPressed)
                        ,
                        button("Positive")
                        .style(theme::Button::Positive)
                        .padding([8, 16])
                        .on_press(Message::ButtonPressed)
                        ,
                        button("Destructive")
                        .style(theme::Button::Destructive)
                        .padding([8, 16])
                        .on_press(Message::ButtonPressed)
                        ,
                        button("Text")
                        .style(theme::Button::Text)
                        .padding([8, 16])
                        .on_press(Message::ButtonPressed)
                        ,
                    ].spacing(12),
                    horizontal_rule(12),
                    row![
                        button("Primary")
                        .style(theme::Button::Primary)
                        .padding([8, 16])
                        ,
                        button("Secondary")
                        .style(theme::Button::Secondary)
                        .padding([8, 16])
                        ,
                        button("Positive")
                        .style(theme::Button::Positive)
                        .padding([8, 16])
                        ,
                        button("Destructive")
                        .style(theme::Button::Destructive)
                        .padding([8, 16])
                        ,
                        button("Text")
                        .style(theme::Button::Text)
                        .padding([8, 16])
                        ,
                    ].spacing(12),
                ]
                .padding([12, 16])
                .spacing(12)
            )
            .style(theme::Container::Custom(listview_style))
            ,
            vertical_space(Length::Units(16)),
            text("Controls").font(FONT_SEMIBOLD),
            container(
                column![
                    row![
                        text("Toggler"),
                        horizontal_space(Length::Fill),
                        toggler(None, self.toggler_value, Message::TogglerToggled)
                        .size(24)
                        .width(Length::Shrink),
                    ]
                    .padding([0, 8])
                    ,
                    horizontal_rule(12),
                    row![
                        text("Slider"),
                        horizontal_space(Length::Fill),
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                        .width(Length::Units(250)),
                    ]
                    .padding([0, 8])
                    ,
                    horizontal_rule(12),
                    row![
                        text("Progress"),
                        horizontal_space(Length::Fill),
                        progress_bar(0.0..=100.0, self.slider_value).height(Length::Units(4))
                        .width(Length::Units(250)),
                    ]
                    .padding([0, 8])
                    ,
                    horizontal_rule(12),
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled),
                ]
                .padding([12, 16])
                .spacing(12)
            )
            .style(theme::Container::Custom(listview_style))
        ]
        .spacing(8)
        .padding(24)
        .max_width(600)
        .into();

        container(row![
            if self.debug { sidebar.explain(Color::WHITE) } else { sidebar },
            horizontal_space(Length::Fill),
            if self.debug { content.explain(Color::WHITE) } else { content },
            horizontal_space(Length::Fill),
        ])
        .padding([16, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
