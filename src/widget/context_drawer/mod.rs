// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An overlayed widget that attaches a toggleable context drawer to the view.

mod overlay;

mod widget;
pub use widget::ContextDrawer;

use crate::Element;

/// An overlayed widget that attaches a toggleable context drawer to the view.
pub fn context_drawer<'a, Message: Clone + 'static, Content, Drawer>(
    header: &'a str,
    on_close: Message,
    content: Content,
    drawer: Drawer,
) -> ContextDrawer<'a, Message>
where
    Content: Into<Element<'a, Message>>,
    Drawer: Into<Element<'a, Message>>,
{
    ContextDrawer::new(header, content, drawer, on_close)
}
