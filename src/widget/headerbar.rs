use apply::Apply;
use iced::{self, alignment::Vertical, Element, Length, theme, widget};
use crate::WindowMsg;

#[derive(Default)]
pub struct HeaderBar {
    pub title: String,
    pub nav_title: String,
    pub sidebar_active: bool,
    pub show_minimize: bool,
    pub show_maximize: bool,
}

impl HeaderBar {
    pub fn render<T>(&self) -> Element<'_, T>
        where T: Clone + From<WindowMsg> + 'static
    {
        let navbutton = {
            let text = widget::text(&self.nav_title)
                .vertical_alignment(Vertical::Center)
                .width(Length::Shrink)
                .height(Length::Fill);

            let icon = super::icon(
                if self.sidebar_active {
                    "go-previous-symbolic"
                } else {
                    "go-next-symbolic"
                },
                24,
            )
            .width(Length::Units(24))
            .height(Length::Fill);

            widget::row!(text, icon)
                .padding(4)
                .spacing(4)
                .apply(widget::button)
                .on_press(T::from(WindowMsg::ToggleSidebar))
                .apply(widget::container)
                .center_y()
                .height(Length::Fill)
                .into()
        };

        let content = widget::container(widget::text(&self.title))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        let window_controls = {
            let mut widgets: Vec<Element<'_, T>> = Vec::with_capacity(3);

            if self.show_minimize {
                widgets.push(
                    super::icon("window-minimize-symbolic", 16)
                        .apply(widget::button)
                        .on_press(T::from(WindowMsg::Minimize))
                        .style(theme::Button::Primary)
                        .into(),
                );
            }

            if self.show_maximize {
                widgets.push(
                    super::icon("window-maximize-symbolic", 16)
                        .apply(widget::button)
                        .on_press(T::from(WindowMsg::Maximize))
                        .style(crate::iced::theme::Button::Primary)
                        .into(),
                );
            }

            widgets.push(
                super::icon("window-close-symbolic", 16)
                    .apply(widget::button)
                    .on_press(T::from(WindowMsg::Close))
                    .style(crate::iced::theme::Button::Primary)
                    .into(),
            );

            widget::row(widgets)
                .spacing(8)
                .apply(widget::container)
                .height(Length::Fill)
                .center_y()
                .into()
        };

        widget::row(vec![navbutton, content, window_controls])
            .height(Length::Units(50))
            .padding(8)
            .apply(widget::event_container)
            .center_y()
            .on_press(T::from(WindowMsg::Drag))
            .into()
    }
}
