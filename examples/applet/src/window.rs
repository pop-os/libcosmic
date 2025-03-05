use cosmic::app::{Core, Task};
use cosmic::cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity;
use cosmic::iced::event::listen_with;
use cosmic::iced::window::Id;
use cosmic::iced::{self, Length, Limits};
use cosmic::iced_runtime::core::window;
use cosmic::iced_runtime::platform_specific::wayland::popup::SctkPopupSettings;
use cosmic::iced_runtime::platform_specific::wayland::subsurface;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::dropdown::DropdownView;
use cosmic::widget::{autosize, dropdown, layer_container, list_column, settings, toggler};
use cosmic::{iced_core, Element};
use once_cell::sync::Lazy;

const ID: &str = "com.system76.CosmicAppletExample";

static SUBSURFACE_ID: Lazy<cosmic::widget::Id> =
    Lazy::new(|| cosmic::widget::Id::new("subsurface"));

pub struct Window {
    core: Core,
    popup: Option<Id>,
    example_row: bool,
    selected: Option<usize>,
    subsurface_id: Id,
    dropdown_id: Id,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            example_row: false,
            selected: None,
            subsurface_id: Id::unique(),
            dropdown_id: Id::unique(),
        }
    }
}

#[derive(Clone)]
pub enum Message {
    PopupClosed(Id),
    PopupCloseRequested(Id),
    ToggleExampleRow(bool),
    Selected(usize),
    Surface(cosmic::surface::Action),
    OpenDropdown(SctkPopupSettings, DropdownView<Message>),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PopupClosed(arg0) => f.debug_tuple("PopupClosed").field(arg0).finish(),
            Self::PopupCloseRequested(arg0) => {
                f.debug_tuple("PopupCloseRequested").field(arg0).finish()
            }
            Self::ToggleExampleRow(arg0) => f.debug_tuple("ToggleExampleRow").field(arg0).finish(),
            Self::Selected(arg0) => f.debug_tuple("Selected").field(arg0).finish(),
            Self::Surface(arg0) => f.debug_tuple("Surface").field(arg0).finish(),
            Self::OpenDropdown(arg0, _) => f.debug_tuple("OpenDropdown").field(arg0).finish(),
        }
    }
}

// impl From<Message> for MessageWrapper<Message> {
//     fn from(value: Message) -> Self {
//         match value {
//             Message::Surface(s) => MessageWrapper::Surface(s),
//             m => MessageWrapper::Message(m),
//         }
//     }
// }

// impl From<SurfaceMessage> for Message {
//     fn from(value: SurfaceMessage) -> Self {
//         Message::Surface(value)
//     }
// }

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

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let window = Window {
            core,
            ..Default::default()
        };
        (window, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => {
                self.example_row = toggled;
            }

            Message::Surface(a) => {
                dbg!("hi i got an action...");
                return cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
                    a,
                )));
            }
            Message::Selected(i) => {
                self.selected = Some(i);
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(cosmic::surface::action::destroy_popup(
                        self.dropdown_id,
                    )),
                ));
            }
            Message::OpenDropdown(sctk_popup_settings, view) => {
                self.dropdown_id = sctk_popup_settings.id;
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(cosmic::surface::action::app_popup(
                        move |_: &mut Window| sctk_popup_settings.clone(),
                        Some(Box::new(move |_: &Window| view().map(cosmic::Action::App))),
                    )),
                ));
            }
            Message::PopupCloseRequested(id) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(cosmic::surface::action::destroy_popup(id)),
                ));
            }
        };
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let btn = self.core.applet.icon_button("display-symbolic").on_press(
            if let Some(id) = self.popup {
                Message::Surface(destroy_popup(id))
            } else {
                Message::Surface(app_popup::<Window>(
                    |state: &mut Window| {
                        let new_id = Id::unique();
                        state.popup = Some(new_id);
                        let popup_settings = state.core.applet.get_popup_settings(
                            state.core.main_window_id().unwrap(),
                            new_id,
                            None,
                            None,
                            None,
                        );

                        popup_settings
                    },
                    Some(Box::new(move |state: &Window| {
                        let content_list = list_column()
                            .padding(5)
                            .spacing(0)
                            .add(settings::item(
                                "Example row",
                                cosmic::widget::container(
                                    toggler(state.example_row)
                                        .on_toggle(|value| Message::ToggleExampleRow(value)),
                                )
                                .height(Length::Fixed(50.)),
                            ))
                            .add(
                                dropdown(
                                    &["1", "asdf", "hello", "test"],
                                    state.selected,
                                    Message::Selected,
                                )
                                .with_popup(state.popup.unwrap_or(Id::NONE), Message::OpenDropdown)
                                .on_close_popup(Message::PopupCloseRequested),
                            );
                        Element::from(state.core.applet.popup_container(content_list))
                            .map(cosmic::Action::App)
                    })),
                ))
            },
        );

        Element::from(
            self.core
                .applet
                .applet_tooltip(btn, "test", self.popup.is_some(), |a| Message::Surface(a)),
        )
    }

    fn view_window(&self, _id: Id) -> Element<Message> {
        "oops".into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        listen_with(|e, status, id| {
            if matches!(e, iced::event::Event::Keyboard(_)) {
                dbg!(e, id);
            }
            None
        })
    }
}
