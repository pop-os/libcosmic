// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use apply::Apply;
use derive_setters::Setters;
use iced::{self, widget, Length};
use crate::{theme, Element};

#[must_use]
pub fn header_bar<'a, Message>() -> HeaderBar<'a, Message> {
    HeaderBar {
        title: "",
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
        start: None,
        center: None,
        end: None,
    }
}

#[derive(Setters)]
pub struct HeaderBar<'a, Message> {
    title: &'a str,
    #[setters(strip_option)]
    on_close: Option<Message>,
    #[setters(strip_option)]
    on_drag: Option<Message>,
    #[setters(strip_option)]
    on_maximize: Option<Message>,
    #[setters(strip_option)]
    on_minimize: Option<Message>,
    #[setters(strip_option)]
    start: Option<Element<'a, Message>>,
    #[setters(strip_option)]
    center: Option<Element<'a, Message>>,
    #[setters(strip_option)]
    end: Option<Element<'a, Message>>
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Converts the headerbar builder into an Iced element.
    pub fn into_element(mut self) -> Element<'a, Message> {
        let mut packed: Vec<Element<Message>> = Vec::with_capacity(4);

        if let Some(start) = self.start.take() {
            packed.push(widget::container(start).align_x(iced::alignment::Horizontal::Left).into());
        }

        packed.push(if let Some(center) = self.center.take() {
            widget::container(center).align_x(iced::alignment::Horizontal::Center).into()
        } else {
            self.title_widget()
        });

        packed.push(if let Some(end) = self.end.take() {
            widget::row(vec![end, self.window_controls()])
                .apply(widget::container)
                .align_x(iced::alignment::Horizontal::Right)
                .into()
        } else {
            self.window_controls()
        });

        let mut widget = widget::row(packed)
            .height(Length::Units(50))
            .padding(10)
            .apply(widget::event_container)
            .center_y();

        if let Some(message) = self.on_drag.clone() {
            widget = widget.on_press(message);
        }

        if let Some(message) = self.on_maximize.clone() {
            widget = widget.on_release(message);
        }

        widget.into()
    }

    fn title_widget(&self) -> Element<'a, Message> {
        widget::container(widget::text(self.title))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Creates the widget for window controls.
    fn window_controls(&mut self) -> Element<'a, Message> {
        let mut widgets: Vec<Element<_>> = Vec::with_capacity(3);

        let icon = |name, size, on_press| {
            super::icon(name, size)
                .style(crate::theme::Svg::SymbolicActive)
                .apply(iced::widget::button)
                .style(theme::Button::Text)
                .on_press(on_press)
        };

        if let Some(message) = self.on_minimize.take() {
            widgets.push(icon("window-minimize-symbolic", 16, message).into());
        }

        if let Some(message) = self.on_maximize.take() {
            widgets.push(icon("window-maximize-symbolic", 16, message).into());
        }

        if let Some(message) = self.on_close.take() {
            widgets.push(icon("window-close-symbolic", 16, message).into());
        }

        widget::row(widgets)
            .spacing(8)
            .apply(widget::container)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBar<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBar<'a, Message>) -> Self {
        headerbar.into_element()
    }
}
