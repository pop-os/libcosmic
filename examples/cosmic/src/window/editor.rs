use cosmic::iced::widget::row;
use cosmic::iced::Length;
use cosmic::iced_winit::Alignment;
use cosmic::widget::{button, segmented_button, view_switcher};
use cosmic::{theme, Element};
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

    pub(super) fn view<'a>(&'a self, window: &'a super::Window) -> Element<'a, Message> {
        let tabs = view_switcher::horizontal(&self.pages)
            .show_close_icon_on_hover(true)
            .on_activate(Message::Activate)
            .on_close(Message::Close)
            .width(Length::Fill);

        let new_tab_button = button(theme::Button::Text)
            .icon(theme::Svg::Symbolic, "tab-new-symbolic", 20)
            .on_press(Message::AddNew);

        row!(tabs, new_tab_button)
            .align_items(Alignment::Center)
            .into()
    }
}
