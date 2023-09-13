use cosmic::{
    iced::widget::{horizontal_space, row, text},
    iced::Length,
    widget::{icon, list_column, settings},
    Element,
};

use super::{Message, Page, SubPage, Window};

#[derive(Default)]
pub struct State {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemAndAccountsPage {
    Users,
    About,
    Firmware,
}

impl SubPage for SystemAndAccountsPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "Users",
            About => "About",
            Firmware => "Firmware",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "Authentication and login, lock screen.",
            About => "Device name, hardware information, operating system defaults.",
            Firmware => "Firmware details.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "system-users-symbolic",
            About => "help-about-symbolic",
            Firmware => "firmware-manager-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::SystemAndAccounts(None)
    }

    fn into_page(self) -> Page {
        Page::SystemAndAccounts(Some(self))
    }
}

impl State {
    pub(super) fn view<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        settings::view_column(vec![
            window.parent_page_button(SystemAndAccountsPage::About),
            row!(
                horizontal_space(Length::Fill),
                icon::from_name("distributor-logo").size(78).icon(),
                horizontal_space(Length::Fill),
            )
            .into(),
            list_column()
                .add(settings::item("Device name", text("TODO")))
                .into(),
            settings::view_section("Hardware")
                .add(settings::item("Hardware model", text("TODO")))
                .add(settings::item("Memory", text("TODO")))
                .add(settings::item("Processor", text("TODO")))
                .add(settings::item("Graphics", text("TODO")))
                .add(settings::item("Disk Capacity", text("TODO")))
                .into(),
            settings::view_section("Operating System")
                .add(settings::item("Operating system", text("TODO")))
                .add(settings::item(
                    "Operating system architecture",
                    text("TODO"),
                ))
                .add(settings::item("Desktop environment", text("TODO")))
                .add(settings::item("Windowing system", text("TODO")))
                .into(),
            settings::view_section("Related settings")
                .add(settings::item("Get support", text("TODO")))
                .into(),
        ])
        .into()
    }
}
