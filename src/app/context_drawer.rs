// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0
//
use std::borrow::Cow;

use crate::Element;

pub struct ContextDrawer<'a, Message: Clone + 'static> {
    pub title: Option<Cow<'a, str>>,
    pub actions: Option<Element<'a, Message>>,
    pub header: Option<Element<'a, Message>>,
    pub content: Element<'a, Message>,
    pub footer: Option<Element<'a, Message>>,
    pub on_close: Message,
}

#[cfg(feature = "about")]
pub fn about<'a, Message: Clone + 'static>(
    about: &'a crate::widget::about::About,
    on_url_press: impl Fn(&'a str) -> Message + 'a,
    on_close: Message,
) -> ContextDrawer<'a, Message> {
    context_drawer(crate::widget::about(about, on_url_press), on_close)
}

pub fn context_drawer<'a, Message: Clone + 'static>(
    content: impl Into<Element<'a, Message>>,
    on_close: Message,
) -> ContextDrawer<'a, Message> {
    ContextDrawer {
        title: None,
        actions: None,
        header: None,
        content: content.into(),
        footer: None,
        on_close,
    }
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    /// Set a context drawer title
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// App-specific actions at the top-left corner of the context drawer
    pub fn actions(mut self, actions: impl Into<Element<'a, Message>>) -> Self {
        self.actions = Some(actions.into());
        self
    }

    /// Elements placed above the context drawer scrollable
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Elements placed below the context drawer scrollable
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn map<Out: Clone + 'static>(
        self,
        on_message: fn(Message) -> Out,
    ) -> ContextDrawer<'a, Out> {
        ContextDrawer {
            title: self.title,
            actions: self.actions.map(|el| el.map(on_message)),
            header: self.header.map(|el| el.map(on_message)),
            content: self.content.map(on_message),
            footer: self.footer.map(|el| el.map(on_message)),
            on_close: on_message(self.on_close),
        }
    }
}
