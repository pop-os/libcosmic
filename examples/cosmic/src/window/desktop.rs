use cosmic::{
    Element,
    iced::Length,
    iced::widget::horizontal_space,
    widget::{settings, toggler},
};

use super::{Message, Page, SubPage, Window};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopPage {
    DesktopOptions,
    Wallpaper,
    Appearance,
    DockAndTopPanel,
    Workspaces,
    Notifications,
}

impl SubPage for DesktopPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "Desktop Options",
            Wallpaper => "Wallpaper",
            Appearance => "Appearance",
            DockAndTopPanel => "Dock & Top Panel",
            Workspaces => "Workspaces",
            Notifications => "Notifications",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "Super Key action, hot corners, window control options.",
            Wallpaper => "Background images, colors, and slideshow options.",
            Appearance => "Accent colors and COSMIC theming",
            DockAndTopPanel => "Customize size, positions, and more for Dock and Top Panel.",
            Workspaces => "Set workspace number, behavior, and placement.",
            Notifications => "Do Not Disturb, lockscreen notifications, and per-application settings.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "video-display-symbolic",
            Wallpaper => "preferences-desktop-wallpaper-symbolic",
            Appearance => "preferences-pop-desktop-appearance-symbolic",
            DockAndTopPanel => "preferences-pop-desktop-dock-symbolic",
            Workspaces => "preferences-pop-desktop-workspaces-symbolic",
            Notifications => "preferences-system-notifications-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::Desktop(None)
    }

    fn into_page(self) -> Page {
        Page::Desktop(Some(self))
    }
}

impl Window {
    pub(super) fn view_desktop_options(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(DesktopPage::DesktopOptions),

            settings::view_section("Super Key Action")
                .add(settings::item("TODO", horizontal_space(Length::Fill)))
                .into(),

            settings::view_section("Hot Corner")
                .add(settings::item("Enable top-left hot corner for Workspaces", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Top Panel")
                .add(settings::item("Show Workspaces Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Show Applications Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Window Controls")
                .add(settings::item("Show Minimize Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Show Maximize Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),
        ]).into()
    }
}
