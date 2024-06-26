// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MIT

use iced::Size;
use iced_runtime::core::widget::Id;
use iced_runtime::{keyboard, Command};

use iced_core::event::{self, Event};
use iced_core::renderer::{self, Quad, Renderer};
use iced_core::touch;
use iced_core::widget::tree::{self, Tree};
use iced_core::widget::Operation;
use iced_core::Element;
use iced_core::{layout, svg};
use iced_core::{mouse, Border};
use iced_core::{overlay, Shadow};
use iced_core::{
    Background, Clipboard, Color, Layout, Length, Padding, Point, Rectangle, Shell, Vector, Widget,
};
use iced_renderer::core::widget::{operation, OperationOutputWrapper};



use super::Toasts;

pub struct Toaster<'a, Message, Theme, Renderer> {
    toasts: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Toaster<'a, Message, Theme, Renderer> {
    pub fn new(
        toasts: Element<'a, Message, Theme, Renderer>,
        content: Element<'a, Message, Theme, Renderer>,
    ) -> Self {
        Self {
            toasts,
            content,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toaster<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn size(&self) -> Size<Length> {
        todo!()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        todo!()
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        todo!()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message, Theme, Renderer> From<Toaster<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: 'a,
    Message: 'a
{
    fn from(
        toggler: Toaster<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(toggler)
    }
}
