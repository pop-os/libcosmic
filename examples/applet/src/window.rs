use std::sync::{Arc, Mutex};

use cosmic::app::Core;
use cosmic::iced::application;
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Limits, Task};
use cosmic::iced_runtime::core::window;
use cosmic::iced_runtime::platform_specific::wayland::subsurface;
use cosmic::theme::iced;
use cosmic::widget::{layer_container, list_column, settings, toggler};
use cosmic::{applet, iced_core, Element, Theme};

const ID: &str = "com.system76.CosmicAppletExample";

#[derive(Default)]
pub struct Window {
    core: Core,
    popup: Option<Id>,
    example_row: bool,
    subsurface_id: Option<Id>,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    ToggleExampleRow(bool),
    Hover,
    Leave,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::app::Message<Self::Message>>) {
        let window = Window {
            core,
            ..Default::default()
        };
        (window, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::app::Message<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    cosmic::task::message(cosmic::app::message::destroy_popup(p))
                } else {
                    cosmic::task::message(
                        cosmic::app::message::get_popup(
                            |state: &mut Window| {
                                let new_id = Id::unique();
                                state.popup = Some(new_id);
                                let mut popup_settings = state.core.applet.get_popup_settings(
                                    state.core.main_window_id().unwrap(),
                                    new_id,
                                    None,
                                    None,
                                    None,
                                );
                                popup_settings.positioner.size_limits = Limits::NONE
                                    .max_width(372.0)
                                    .min_width(300.0)
                                    .min_height(200.0)
                                    .max_height(1080.0)
                                    .height(500)
                                    .width(500);
                                popup_settings.positioner.size = Some((500, 500));
                                popup_settings
                            },
                            Some(
                                move |state: &Window| -> cosmic::Element<
                                    'static,
                                    cosmic::app::Message<Message>,
                                > {
                                    {
                                        let content_list = list_column().padding(5).spacing(0).add(
                                            settings::item(
                                                "Example row",
                                                cosmic::widget::container(
                                                    toggler(state.example_row).on_toggle(|value| {
                                                        Message::ToggleExampleRow(value)
                                                    }),
                                                )
                                                .height(Length::Fixed(50.)),
                                            ),
                                        );

                                        Element::from(
                                            state.core.applet.popup_container(content_list),
                                        )
                                        .map(cosmic::app::Message::App)
                                    }
                                },
                            ),
                        ),
                    )
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => {
                self.example_row = toggled;
            }
            Message::Hover => {
                return cosmic::task::message(cosmic::app::message::get_subsurface(
                    |app: &mut Window| {
                        let id = window::Id::unique();
                        app.subsurface_id = Some(id);

                        subsurface::SctkSubsurfaceSettings {
                            parent: window::Id::RESERVED,
                            id,
                            loc: iced_core::Point { x: -100., y: 0. },
                            size: Some(iced_core::Size::new(100., 18.)),
                            z: 1,
                        }
                    },
                    Some(|app: &Window| layer_container(cosmic::widget::text("hello")).into()),
                ));
            }
            Message::Leave => {
                return cosmic::task::message(cosmic::app::message::destroy_subsurface(
                    self.subsurface_id.unwrap_or(window::Id::NONE),
                ));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        cosmic::widget::wayland::tooltip::widget::Tooltip::new(
            self.core
                .applet
                .icon_button("display-symbolic")
                .on_press(Message::TogglePopup),
            |layout| Message::Hover,
            Message::Leave,
        )
        .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        "oops".into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
