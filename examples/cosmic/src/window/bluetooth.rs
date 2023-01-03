use super::{Page, Window};
use cosmic::{
    iced::widget::{column, text},
    widget::{list_column, settings, toggler},
    Element,
};

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Enable(bool),
}

#[derive(Default)]
pub struct State {
    enabled: bool,
}

impl State {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Enable(value) => self.enabled = value,
        }
    }

    pub(super) fn view<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        settings::view_column(vec![
            window.page_title(Page::Bluetooth),
            column!(
                list_column().add(settings::item(
                    "Bluetooth",
                    toggler(None, self.enabled, Message::Enable)
                )),
                text("Now visible as \"TODO\", just kidding")
            )
            .spacing(8)
            .into(),
            settings::view_section("Devices")
                .add(settings::item("No devices found", text("")))
                .into(),
        ])
        .into()
    }
}
