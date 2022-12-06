// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use iced::futures::SinkExt;
use iced::{
    futures::StreamExt,
    widget::{container, Container},
    Rectangle,
};
use iced_native::alignment::{self, Alignment};
use iced_native::command::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner};
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::{Operation, Tree};
use iced_native::{
    window, Background, Clipboard, Color, Element, Layout, Length, Padding, Point, Shell, Widget,
};
use std::u32;

pub use iced_style::container::{Appearance, StyleSheet};
pub struct SizeTrackingContainer<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    container: Container<'a, Message, Renderer>,
    tx: UnboundedSender<Rectangle<i32>>,
}

impl<'a, Message, Renderer> SizeTrackingContainer<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [`Container`].
    pub fn new<T>(content: T, tx: UnboundedSender<Rectangle<i32>>) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        SizeTrackingContainer {
            container: container(content),
            tx,
        }
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.container = self.container.padding(padding);
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: Length) -> Self {
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: Length) -> Self {
        self.container = self.container.height(height);
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.container = self.container.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.container = self.container.max_height(max_height);
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.container = self.container.align_x(alignment);
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.container = self.container.align_y(alignment);
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.container = self.container.center_x();
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.container = self.container.center_y();
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.container = self.container.style(style);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for SizeTrackingContainer<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        self.container.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.container.diff(tree)
    }

    fn width(&self) -> Length {
        Widget::width(&self.container)
    }

    fn height(&self) -> Length {
        Widget::height(&self.container)
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.container.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.container.on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.container.mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = layout.bounds();
        let _ = self.tx.unbounded_send(Rectangle {
            x: x as i32,
            y: y as i32,
            width: width as i32,
            height: height as i32,
        });
        self.container.draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.container
            .overlay(&mut tree.children[0], layout, renderer)
    }
}

pub struct PopupParentSubscription {
    id: window::Id,
    settings: SctkPopupSettings,
}

impl PopupParentSubscription {
    pub fn new(id: window::Id, settings: SctkPopupSettings) -> Self {
        Self { id, settings }
    }

    pub fn get_popup_container<'a, T, Message, Renderer>(
        &self,
        content: T,
        tx: UnboundedSender<Rectangle<i32>>,
    ) -> SizeTrackingContainer<'a, Message, Renderer>
    where
        T: Into<Element<'a, Message, Renderer>>,
        Renderer: iced_native::Renderer,
        Renderer::Theme: StyleSheet,
    {
        SizeTrackingContainer::new(content, tx.clone())
    }

    pub fn subscription(&self) -> iced::Subscription<(window::Id, PositionerUpdate)> {
        popup_resize(self.id, self.settings.clone())
    }
}

pub fn popup_resize(
    id: window::Id,
    settings: SctkPopupSettings,
) -> iced::Subscription<(window::Id, PositionerUpdate)> {
    iced_native::subscription::unfold(
        id,
        State::Init(settings.positioner.anchor_rect.clone()),
        move |state| rectangle_size(id, state),
    )
    .with(settings)
    .map(|(settings, (id, update))| match update {
        RectangleUpdate::Update(rect) => {
            let mut new_pos = settings.positioner.clone();
            new_pos.anchor_rect = rect;
            (id, PositionerUpdate::Update(new_pos))
        }
        RectangleUpdate::Finished => (id, PositionerUpdate::Finished),
        RectangleUpdate::Sender(sender) => (id, PositionerUpdate::Sender(sender)),
    })
}

#[derive(Debug, Clone)]
pub enum PositionerUpdate {
    Sender(UnboundedSender<Rectangle<i32>>),
    Update(SctkPositioner),
    Finished,
}

#[derive(Debug, Clone)]
pub enum RectangleUpdate {
    Sender(UnboundedSender<Rectangle<i32>>),
    Update(Rectangle<i32>),
    Finished,
}

pub enum State {
    Init(Rectangle<i32>),
    WaitForUpdate(Rectangle<i32>, UnboundedReceiver<Rectangle<i32>>),
    Finished,
}

async fn rectangle_size<I: Copy>(id: I, state: State) -> (Option<(I, RectangleUpdate)>, State) {
    match state {
        State::Init(rectangle) => {
            let (tx, rx) = unbounded();
            (
                Some((id, RectangleUpdate::Sender(tx))),
                State::WaitForUpdate(rectangle, rx),
            )
        }
        State::WaitForUpdate(old_rectangle, mut rx) => {
            let response = rx.next().await;

            match response {
                Some(new_rectangle) => {
                    let new_update = if new_rectangle == old_rectangle {
                        None
                    } else {
                        Some((id, RectangleUpdate::Update(new_rectangle)))
                    };
                    (new_update, State::WaitForUpdate(new_rectangle, rx))
                }
                None => (Some((id, RectangleUpdate::Finished)), State::Finished),
            }
        }
        State::Finished => iced::futures::future::pending().await,
    }
}
