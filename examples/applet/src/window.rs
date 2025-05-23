use cosmic::app::{Core, Task};

use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
use cosmic::iced_runtime::core::window;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{dropdown::popup_dropdown, list_column, settings, toggler};
use cosmic::Element;

const ID: &str = "com.system76.CosmicAppletExample";

pub struct Window {
    core: Core,
    popup: Option<Id>,
    example_row: bool,
    selected: Option<usize>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            example_row: false,
            selected: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    ToggleExampleRow(bool),
    Selected(usize),
    Surface(cosmic::surface::Action),
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
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::Selected(i) => {
                self.selected = Some(i);
            }
        };
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let have_popup = self.popup.clone();
        let btn = self
            .core
            .applet
            .icon_button("display-symbolic")
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<Window>(
                        move |state: &mut Window| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };

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
                                .add(popup_dropdown(
                                    &["1", "asdf", "hello", "test"],
                                    state.selected,
                                    Message::Selected,
                                    state.popup.unwrap_or(Id::NONE),
                                    Message::Surface,
                                    |m| m,
                                ));
                            Element::from(state.core.applet.popup_container(content_list))
                                .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            "test",
            self.popup.is_some(),
            |a| Message::Surface(a),
            None,
        ))
    }

    fn view_window(&self, _id: Id) -> Element<Message> {
        "oops".into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
