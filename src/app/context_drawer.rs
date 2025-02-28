use std::borrow::Cow;

use crate::Element;

pub struct ContextDrawer<'a, Message: Clone + 'static> {
    pub title: Option<Cow<'a, str>>,
    pub header_actions: Vec<Element<'a, Message>>,
    pub header: Option<Element<'a, Message>>,
    pub content: Element<'a, Message>,
    pub footer: Option<Element<'a, Message>>,
    pub on_close: Message,
}

#[cfg(feature = "about")]
pub fn about<Message: Clone + 'static>(
    about: &crate::widget::about::About,
    on_url_press: impl Fn(String) -> Message,
    on_close: Message,
) -> ContextDrawer<'_, Message> {
    context_drawer(crate::widget::about(about, on_url_press), on_close)
}

pub fn context_drawer<'a, Message: Clone + 'static>(
    content: impl Into<Element<'a, Message>>,
    on_close: Message,
) -> ContextDrawer<'a, Message> {
    ContextDrawer {
        title: None,
        content: content.into(),
        header_actions: vec![],
        footer: None,
        on_close,
        header: None,
    }
}

impl<'a, Message: Clone + 'static> ContextDrawer<'a, Message> {
    /// Set a context drawer header title
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// App-specific actions at the start of the context drawer header
    pub fn header_actions(
        mut self,
        header_actions: impl IntoIterator<Item = Element<'a, Message>>,
    ) -> Self {
        self.header_actions = header_actions.into_iter().collect();
        self
    }
    /// Non-scrolling elements placed below the context drawer title row
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Elements placed below the context drawer scrollable
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }
}
