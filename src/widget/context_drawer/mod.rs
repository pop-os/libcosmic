// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An overlayed widget that attaches a toggleable context drawer to the view.

mod overlay;

mod widget;
use std::borrow::Cow;

pub use widget::ContextDrawer;

use crate::Element;

/// An overlayed widget that attaches a toggleable context drawer to the view.
pub fn context_drawer<'a, Message: Clone + 'static, Content, Drawer>(
    title: Option<Cow<'a, str>>,
    actions: Option<Element<'a, Message>>,
    header: Option<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    on_close: Message,
    content: Content,
    drawer: Drawer,
    max_width: f32,
) -> ContextDrawer<'a, Message>
where
    Content: Into<Element<'a, Message>>,
    Drawer: Into<Element<'a, Message>>,
{
    ContextDrawer::new(
        title, actions, header, footer, content, drawer, on_close, max_width,
    )
}
