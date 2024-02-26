use cosmic::iced::widget::{horizontal_space, row};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, icon, segmented_button, tab_bar};
use cosmic::{Apply, Element};
use slotmap::Key;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Activate(segmented_button::Entity),
    AddNew,
    Close(segmented_button::Entity),
}

pub struct State {
    pub pages: segmented_button::SingleSelectModel,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            pages: segmented_button::Model::default(),
        };

        let id = state.tab_add_new();
        state.pages.activate(id);

        state
    }
}

impl State {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Activate(id) => self.pages.activate(id),
            Message::AddNew => {
                self.tab_add_new();
            }
            Message::Close(id) => self.tab_close(id),
        }
    }

    pub fn tab_add_new(&mut self) -> segmented_button::Entity {
        let id = self.pages.insert().closable().id();

        self.pages
            .text_set(id, format!("Tab {}", id.data().as_ffi() & 0xffff_ffff));

        id
    }

    pub fn tab_close(&mut self, id: segmented_button::Entity) {
        if self.pages.is_active(id) {
            if let Some(pos) = self.pages.position(id) {
                let next = if pos == 0 { pos + 1 } else { pos - 1 };
                self.pages.activate_position(next);
            }
        }

        self.pages.remove(id);
    }

    pub(super) fn view<'a>(&'a self, _window: &'a super::Window) -> Element<'a, Message> {
        let tabs = tab_bar::horizontal(&self.pages)
            .show_close_icon_on_hover(true)
            .on_activate(Message::Activate)
            .on_close(Message::Close)
            .width(Length::Shrink);

        let new_tab_button = icon::from_name("tab-new-symbolic")
            .size(20)
            .apply(button::icon)
            .on_press(Message::AddNew);

        let tab_header = row!(tabs, new_tab_button).align_items(Alignment::Center);

        row!(tab_header, horizontal_space(Length::Fill)).into()
    }
}
