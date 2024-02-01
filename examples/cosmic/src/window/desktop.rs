use cosmic::{
    iced::widget::{column, container, horizontal_space, image, row, svg, text},
    iced::Length,
    theme,
    widget::{list_column, settings, toggler},
    Element,
};

use super::{Page, SubPage, Window};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopPage {
    DesktopOptions,
    Wallpaper,
    Appearance,
    DockAndTopPanel,
    Workspaces,
    Notifications,
}

#[derive(Debug, Default)]
pub struct State {
    pub top_left_hot_corner: bool,
    pub show_workspaces_button: bool,
    pub show_applications_button: bool,
    pub show_minimize_button: bool,
    pub show_maximize_button: bool,
    pub slideshow: bool,
    pub same_background: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Page(Page),
    Slideshow(bool),
    SameBackground(bool),
    ShowWorkspacesButton(bool),
    ShowApplicationsButton(bool),
    ShowMinimizeButton(bool),
    ShowMaximizeButton(bool),
    TopLeftHotCorner(bool),
}

impl From<Page> for Message {
    fn from(page: Page) -> Message {
        Message::Page(page)
    }
}

pub enum Output {
    Page(Page),
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
            Notifications => {
                "Do Not Disturb, lockscreen notifications, and per-application settings."
            }
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

impl State {
    pub(super) fn update(&mut self, message: Message) -> Option<Output> {
        match message {
            Message::Page(page) => return Some(Output::Page(page)),
            Message::SameBackground(value) => self.same_background = value,
            Message::ShowApplicationsButton(value) => self.show_applications_button = value,
            Message::ShowMaximizeButton(value) => self.show_maximize_button = value,
            Message::ShowMinimizeButton(value) => self.show_maximize_button = value,
            Message::ShowWorkspacesButton(value) => self.show_workspaces_button = value,
            Message::Slideshow(value) => self.slideshow = value,
            Message::TopLeftHotCorner(value) => self.top_left_hot_corner = value,
        }
        None
    }

    pub(super) fn view<'a>(
        &'a self,
        window: &'a Window,
        desktop_page_opt: Option<DesktopPage>,
    ) -> Element<'a, Message> {
        match desktop_page_opt {
            None => settings::view_column(vec![
                window.page_title(window.page),
                column!(
                    window.sub_page_button(DesktopPage::DesktopOptions),
                    window.sub_page_button(DesktopPage::Wallpaper),
                    window.sub_page_button(DesktopPage::Appearance),
                    window.sub_page_button(DesktopPage::DockAndTopPanel),
                    window.sub_page_button(DesktopPage::Workspaces),
                    window.sub_page_button(DesktopPage::Notifications),
                )
                .spacing(16)
                .into(),
            ])
            .width(Length::Fill)
            .into(),
            Some(DesktopPage::DesktopOptions) => self.view_desktop_options(window),
            Some(DesktopPage::Wallpaper) => self.view_desktop_wallpaper(window),
            Some(DesktopPage::Workspaces) => self.view_desktop_workspaces(window),
            Some(sub_page) => window.view_unimplemented_sub_page(sub_page),
        }
    }

    fn view_desktop_options<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        settings::view_column(vec![
            window.parent_page_button(DesktopPage::DesktopOptions),
            settings::view_section("Super Key Action")
                .add(settings::item("Launcher", horizontal_space(Length::Fill)))
                .add(settings::item("Workspaces", horizontal_space(Length::Fill)))
                .add(settings::item(
                    "Applications",
                    horizontal_space(Length::Fill),
                ))
                .into(),
            settings::view_section("Hot Corner")
                .add(settings::item(
                    "Enable top-left hot corner for Workspaces",
                    toggler(None, self.top_left_hot_corner, Message::TopLeftHotCorner),
                ))
                .into(),
            settings::view_section("Top Panel")
                .add(settings::item(
                    "Show Workspaces Button",
                    toggler(
                        None,
                        self.show_workspaces_button,
                        Message::ShowWorkspacesButton,
                    ),
                ))
                .add(settings::item(
                    "Show Applications Button",
                    toggler(
                        None,
                        self.show_applications_button,
                        Message::ShowApplicationsButton,
                    ),
                ))
                .into(),
            settings::view_section("Window Controls")
                .add(settings::item(
                    "Show Minimize Button",
                    toggler(None, self.show_minimize_button, Message::ShowMinimizeButton),
                ))
                .add(settings::item(
                    "Show Maximize Button",
                    toggler(None, self.show_maximize_button, Message::ShowMaximizeButton),
                ))
                .into(),
        ])
        .into()
    }

    fn view_desktop_wallpaper<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        let image_paths: Vec<std::path::PathBuf> = Vec::new();
        /*
        //TODO: load image paths, do this asynchronously somehow
        if let Ok(entries) = std::fs::read_dir("/usr/share/backgrounds") {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(ok) => ok,
                    Err(_) => continue,
                };

                let path = entry.path();
                if path.is_dir() {
                    //TODO: recursive
                } else {
                    image_paths.push(path);
                }
            }
        }
        */

        let mut image_column = Vec::with_capacity(image_paths.len() / 4);
        for chunk in image_paths.chunks(4) {
            let mut image_row = Vec::with_capacity(chunk.len());
            for image_path in chunk.iter() {
                image_row.push(if image_path.ends_with(".svg") {
                    svg(svg::Handle::from_path(image_path))
                        .width(Length::Fixed(150.0))
                        .into()
                } else {
                    image(image_path).width(Length::Fixed(150.0)).into()
                });
            }
            image_column.push(row(image_row).spacing(16).into());
        }

        settings::view_column(vec![
            window.parent_page_button(DesktopPage::Wallpaper),
            row!(
                horizontal_space(Length::Fill),
                container(
                    image("/usr/share/backgrounds/pop/kate-hazen-COSMIC-desktop-wallpaper.png")
                        .width(Length::Fixed(300.0))
                )
                .padding(4)
                .style(theme::Container::Background),
                horizontal_space(Length::Fill),
            )
            .into(),
            list_column()
                .add(settings::item(
                    "Same background on all displays",
                    toggler(None, self.same_background, Message::SameBackground),
                ))
                .add(settings::item("Background fit", text("TODO")))
                .add(settings::item(
                    "Slideshow",
                    toggler(None, self.slideshow, Message::Slideshow),
                ))
                .into(),
            column(image_column).spacing(16).into(),
        ])
        .into()
    }

    fn view_desktop_workspaces<'a>(&'a self, window: &'a Window) -> Element<'a, Message> {
        settings::view_column(vec![
            window.parent_page_button(DesktopPage::Wallpaper),
            settings::view_section("Workspace Behavior")
                .add(settings::item(
                    "Dynamic workspaces",
                    horizontal_space(Length::Fill),
                ))
                .add(settings::item(
                    "Fixed Number of Workspaces",
                    horizontal_space(Length::Fill),
                ))
                .into(),
            settings::view_section("Multi-monitor Behavior")
                .add(settings::item(
                    "Workspaces Span Displays",
                    horizontal_space(Length::Fill),
                ))
                .add(settings::item(
                    "Displays Have Separate Workspaces",
                    horizontal_space(Length::Fill),
                ))
                .into(),
        ])
        .into()
    }
}
