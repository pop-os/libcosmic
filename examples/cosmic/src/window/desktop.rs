use cosmic::{
    Element,
    iced::Length,
    iced::widget::{column, container, horizontal_space, image, row, text},
    theme,
    widget::{list_column, settings, toggler},
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
    pub(super) fn view_desktop(&self, desktop_page_opt: Option<DesktopPage>) -> Element<Message> {
        match desktop_page_opt {
            None => settings::view_column(vec![
                self.page_title(self.page),
                column!(
                    self.sub_page_button(DesktopPage::DesktopOptions),
                    self.sub_page_button(DesktopPage::Wallpaper),
                    self.sub_page_button(DesktopPage::Appearance),
                    self.sub_page_button(DesktopPage::DockAndTopPanel),
                    self.sub_page_button(DesktopPage::Workspaces),
                    self.sub_page_button(DesktopPage::Notifications),
                ).spacing(16).into()
            ]).into(),
            Some(DesktopPage::DesktopOptions) => self.view_desktop_options(),
            Some(DesktopPage::Wallpaper) => self.view_desktop_wallpaper(),
            Some(DesktopPage::Workspaces) => self.view_desktop_workspaces(),
            Some(sub_page) => self.view_unimplemented_sub_page(sub_page),
        }
    }

    fn view_desktop_options(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(DesktopPage::DesktopOptions),

            settings::view_section("Super Key Action")
                .add(settings::item("Launcher", horizontal_space(Length::Fill)))
                .add(settings::item("Workspaces", horizontal_space(Length::Fill)))
                .add(settings::item("Applications", horizontal_space(Length::Fill)))
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

    fn view_desktop_wallpaper(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(DesktopPage::Wallpaper),

            row!(
                horizontal_space(Length::Fill),
                container(
                    image(
                        "/usr/share/backgrounds/pop/kate-hazen-COSMIC-desktop-wallpaper.png"
                    ).width(Length::Units(300))
                )
                .padding(4)
                .style(theme::Container::Box),
                horizontal_space(Length::Fill),
            ).into(),

            list_column()
                .add(settings::item("Same background on all displays", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Background fit", text("TODO")))
                .add(settings::item("Slideshow", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),
        ]).into()
    }

    fn view_desktop_workspaces(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(DesktopPage::Wallpaper),

            settings::view_section("Workspace Behavior")
                .add(settings::item("Dynamic workspaces", horizontal_space(Length::Fill)))
                .add(settings::item("Fixed Number of Workspaces", horizontal_space(Length::Fill)))
                .into(),

            settings::view_section("Multi-monitor Behavior")
                .add(settings::item("Workspaces Span Displays", horizontal_space(Length::Fill)))
                .add(settings::item("Displays Have Separate Workspaces", horizontal_space(Length::Fill)))
                .into(),
        ]).into()
    }
}
