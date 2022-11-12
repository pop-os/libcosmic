use crate::dbus::{self, PowerDaemonProxy};
use crate::graphics::{get_current_graphics, set_graphics, Graphics};
use cosmic::applet::{get_popup_settings, icon_button};
use cosmic::separator;
use cosmic::{
    iced::widget::{column, radio, text},
    iced::{self, Application, Command, Length},
    iced_native::window,
    theme::{self, Theme},
    Element,
};
use cosmic_panel_config::{PanelAnchor, PanelSize};
use iced_sctk::alignment::Horizontal;
use iced_sctk::commands::popup::{destroy_popup, get_popup};
use zbus::Connection;

#[derive(Default, Clone, Copy)]
enum State {
    #[default]
    SelectGraphicsMode,
    SettingGraphicsMode(Graphics),
}

#[derive(Default)]
pub struct Window {
    popup: Option<window::Id>,
    graphics_mode: Option<Graphics>,
    id_ctr: u32,
    icon_size: u16,
    anchor: PanelAnchor,
    theme: Theme,
    dbus: Option<(Connection, PowerDaemonProxy<'static>)>,
    state: State,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    CurrentGraphics(Option<Graphics>),
    AppliedGraphicsMode(Option<Graphics>),
    DBusInit(Option<(Connection, PowerDaemonProxy<'static>)>),
    SelectGraphicsMode(Graphics),
    TogglePopup,
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut window = Window::default();
        let pixels = std::env::var("COSMIC_PANEL_SIZE")
            .ok()
            .and_then(|size| match size.parse::<PanelSize>() {
                Ok(PanelSize::XL) => Some(64),
                Ok(PanelSize::L) => Some(36),
                Ok(PanelSize::M) => Some(24),
                Ok(PanelSize::S) => Some(16),
                Ok(PanelSize::XS) => Some(12),
                Err(_) => Some(12),
            })
            .unwrap_or(16);
        window.icon_size = pixels;
        window.anchor = std::env::var("COSMIC_PANEL_ANCHOR")
            .ok()
            .map(|size| match size.parse::<PanelAnchor>() {
                Ok(p) => p,
                Err(_) => PanelAnchor::Top,
            })
            .unwrap_or(PanelAnchor::Top);
        (
            window,
            Command::perform(dbus::init(), |dbus_init| Message::DBusInit(dbus_init)),
        )
    }

    fn title(&self) -> String {
        String::from("Cosmic Graphics Applet")
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::SelectGraphicsMode(new_graphics_mode) => {
                dbg!(new_graphics_mode);
                if let Some((_, proxy)) = self.dbus.as_ref() {
                    self.state = State::SettingGraphicsMode(new_graphics_mode);
                    return Command::perform(
                        set_graphics(proxy.clone(), new_graphics_mode),
                        move |success| {
                            Message::AppliedGraphicsMode(success.ok().map(|_| new_graphics_mode))
                        },
                    );
                }
            }
            Message::AppliedGraphicsMode(g) => {
                if let Some(g) = g {
                    dbg!(g);
                    self.graphics_mode.replace(g);
                    self.state = State::SelectGraphicsMode;
                }
            }
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                } else {
                    self.id_ctr += 1;
                    let new_id = window::Id::new(self.id_ctr);
                    self.popup.replace(new_id);
                    let mut commands = Vec::new();
                    if let Some((_, proxy)) = self.dbus.as_ref() {
                        commands.push(Command::perform(
                            get_current_graphics(proxy.clone()),
                            |cur_graphics| Message::CurrentGraphics(cur_graphics.ok()),
                        ));
                    }
                    let popup_settings = get_popup_settings(window::Id::new(0), new_id, None, None);
                    commands.push(get_popup(popup_settings));
                    return Command::batch(commands);
                }
            }
            Message::DBusInit(dbus) => {
                self.dbus = dbus;
                return Command::perform(
                    get_current_graphics(self.dbus.as_ref().unwrap().1.clone()),
                    |cur_graphics| {
                        Message::CurrentGraphics(match cur_graphics {
                            Ok(g) => Some(g),
                            Err(err) => {
                                dbg!(err);
                                None
                            }
                        })
                    },
                );
            }
            Message::CurrentGraphics(g) => {
                dbg!(g);
                if let Some(g) = g {
                    self.graphics_mode.replace(g);
                }
            }
        }
        Command::none()
    }

    fn view_popup(&self, _: window::Id) -> Element<Message> {
        let content = match self.state {
            State::SelectGraphicsMode => column(vec![
                radio(
                    "Integrated Graphics",
                    Graphics::Integrated,
                    self.graphics_mode,
                    |g| Message::SelectGraphicsMode(g),
                )
                .into(),
                radio(
                    "Nvidia Graphics",
                    Graphics::Nvidia,
                    self.graphics_mode,
                    |g| Message::SelectGraphicsMode(g),
                )
                .into(),
                radio(
                    "Hybrid Graphics",
                    Graphics::Hybrid,
                    self.graphics_mode,
                    |g| Message::SelectGraphicsMode(g),
                )
                .into(),
                radio(
                    "Compute Graphics",
                    Graphics::Compute,
                    self.graphics_mode,
                    |g| Message::SelectGraphicsMode(g),
                )
                .into(),
            ])
            .padding([8, 0])
            .spacing(8)
            .into(),
            State::SettingGraphicsMode(graphics) => {
                let graphics_str = match graphics {
                    Graphics::Integrated => "integrated",
                    Graphics::Hybrid => "hybrid",
                    Graphics::Nvidia => "nvidia",
                    Graphics::Compute => "compute",
                };
                column(vec![text(format!(
                    "Setting graphics mode to {graphics_str}..."
                ))
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .into()])
                .into()
            }
        };
        column(vec![
            text("Graphics Mode")
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .size(24)
                .into(),
            separator!(1).into(),
            content,
        ])
        .padding(4)
        .spacing(4)
        .into()
    }

    fn view_layer_surface(
        &self,
        _: cosmic::iced_native::window::Id,
    ) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        unimplemented!()
    }
    fn close_window_requested(&self, _: cosmic::iced_native::window::Id) -> Self::Message {
        unimplemented!()
    }
    fn popup_done(&self, _: cosmic::iced_native::window::Id) -> Self::Message {
        unimplemented!()
    }
    fn layer_surface_done(&self, _: cosmic::iced_native::window::Id) -> Self::Message {
        unimplemented!()
    }

    fn view_window(&self, _: window::Id) -> Element<Message> {
        // TODO use panel config crate after resolving version mismatch

        let btn = icon_button().on_press(Message::TogglePopup);
        btn.into()
    }

    fn should_exit(&self) -> bool {
        false
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
