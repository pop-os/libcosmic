use cosmic::{
    Element,
    iced::widget::{column, text},
    widget::{list_column, settings, toggler},
};
use super::{Message, Page, Window};

impl Window {
    pub(super) fn view_bluetooth(&self) -> Element<Message> {
        settings::view_column(vec![
            self.page_title(Page::Bluetooth),

            column!(
                list_column()
                    .add(settings::item("Bluetooth", toggler(None, self.toggler_value, Message::TogglerToggled))),
                text("Now visible as \"TODO\", just kidding")
            ).spacing(8).into(),

            settings::view_section("Devices")
                .add(settings::item("No devices found", text("")))
                .into()
        ]).into()
    }
}
