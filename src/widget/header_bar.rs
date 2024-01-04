// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{ext::CollectionWidget, widget, Element};
use apply::Apply;
use derive_setters::Setters;
use iced::Length;
use std::borrow::Cow;

#[must_use]
pub fn header_bar<'a, Message>() -> HeaderBar<'a, Message> {
    HeaderBar {
        title: Cow::Borrowed(""),
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
        start: Vec::new(),
        center: Vec::new(),
        end: Vec::new(),
    }
}

#[derive(Setters)]
pub struct HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[setters(skip)]
    title: Cow<'a, str>,

    /// A message emitted when the close button is pressed.
    #[setters(strip_option)]
    on_close: Option<Message>,

    /// A message emitted when dragged.
    #[setters(strip_option)]
    on_drag: Option<Message>,

    /// A message emitted when the maximize button is pressed.
    #[setters(strip_option)]
    on_maximize: Option<Message>,

    /// A message emitted when the minimize button is pressed.
    #[setters(strip_option)]
    on_minimize: Option<Message>,

    /// Elements packed at the start of the headerbar.
    #[setters(skip)]
    start: Vec<Element<'a, Message>>,

    /// Elements packed in the center of the headerbar.
    #[setters(skip)]
    center: Vec<Element<'a, Message>>,

    /// Elements packed at the end of the headerbar.
    #[setters(skip)]
    end: Vec<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[must_use]
    pub fn title(mut self, title: impl Into<Cow<'a, str>> + 'a) -> Self {
        self.title = title.into();
        self
    }

    /// Pushes an element to the start region.
    #[must_use]
    pub fn start(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.start.push(widget.into());
        self
    }

    /// Pushes an element to the center region.
    #[must_use]
    pub fn center(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.center.push(widget.into());
        self
    }

    /// Pushes an element to the end region.
    #[must_use]
    pub fn end(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.end.push(widget.into());
        self
    }
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Converts the headerbar builder into an Iced element.
    pub fn into_element(mut self) -> Element<'a, Message> {
        // Take ownership of the regions to be packed.
        let start = std::mem::take(&mut self.start);
        let center = std::mem::take(&mut self.center);
        let mut end = std::mem::take(&mut self.end);

        // Also packs the window controls at the very end.
        end.push(widget::horizontal_space(Length::Fixed(12.0)).into());
        end.push(self.window_controls());

        // Creates the headerbar widget.
        let mut widget = widget::row::with_capacity(4)
            // If elements exist in the start region, append them here.
            .push(
                widget::row::with_children(start)
                    .align_items(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::alignment::Horizontal::Left)
                    .width(Length::Fill),
            )
            // If elements exist in the center region, use them here.
            // This will otherwise use the title as a widget if a title was defined.
            .push(if !center.is_empty() {
                widget::row::with_children(center)
                    .align_items(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(Length::Fill)
                    .into()
            } else if self.title.is_empty() {
                widget::horizontal_space(Length::Fill).into()
            } else {
                self.title_widget()
            })
            .push(
                widget::row::with_children(end)
                    .align_items(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill),
            )
            .height(Length::Fixed(50.0))
            .padding(8)
            .spacing(8)
            .apply(widget::container)
            .style(crate::theme::Container::HeaderBar)
            .center_y()
            .apply(widget::mouse_area);

        // Assigns a message to emit when the headerbar is dragged.
        if let Some(message) = self.on_drag.clone() {
            widget = widget.on_drag(message);
        }

        // Assigns a message to emit when the headerbar is double-clicked.
        if let Some(message) = self.on_maximize.clone() {
            widget = widget.on_release(message);
        }

        widget.into()
    }

    fn title_widget(&mut self) -> Element<'a, Message> {
        let mut title = Cow::default();
        std::mem::swap(&mut title, &mut self.title);

        widget::text(title)
            .size(16)
            .font(crate::font::FONT_SEMIBOLD)
            .apply(widget::container)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Creates the widget for window controls.
    fn window_controls(&mut self) -> Element<'a, Message> {
        let icon = |icon_bytes, size, on_press| {
            
            widget::icon::from_svg_bytes(
                icon_bytes,
            )
                .symbolic(true)
                .apply(widget::button::icon)
                .icon_size(size)
                .on_press(on_press)
        };

        widget::row::with_capacity(3)
            .push_maybe(
                self.on_minimize
                    .take()
                    .map(|m| icon(&include_bytes!("../../res/icons/window-minimize-symbolic.svg")[..], 16, m)),
            )
            .push_maybe(
                self.on_maximize
                    .take()
                    .map(|m| icon(&include_bytes!("../../res/icons/window-maximize-symbolic.svg")[..], 16, m)),
            )
            .push_maybe(
                self.on_close
                    .take()
                    .map(|m| icon(&include_bytes!("../../res/icons/window-close-symbolic.svg")[..], 16, m)),
            )
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
