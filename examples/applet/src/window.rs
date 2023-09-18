use cosmic::app::Core;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Command, Limits};
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::theme::Button;
use cosmic::widget::{list_column, settings, toggler};
use cosmic::{Element, Theme};

const ID: &str = "com.system76.CosmicAppletExample";

#[derive(Default)]
pub struct Window {
    core: Core,
    popup: Option<Id>,
    id_ctr: u128,
    example_row: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    ToggleExampleRow(bool),
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

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let window = Window {
            core,
            ..Default::default()
        };
        (window, Command::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    self.id_ctr += 1;
                    let new_id = Id(self.id_ctr);
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet_helper
                            .get_popup_settings(Id(0), new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet_helper
            .icon_button(ID)
            .on_press(Message::TogglePopup)
            .style(Button::Text)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let content_list = list_column().padding(5).spacing(0).add(settings::item(
            "Example row",
            toggler(None, self.example_row, |value| {
                Message::ToggleExampleRow(value)
            }),
        ));

        self.core.applet_helper.popup_container(content_list).into()
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::app::applet::style())
    }
}
