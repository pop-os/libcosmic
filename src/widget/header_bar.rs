// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{ext::CollectionWidget, widget, Element};
use apply::Apply;
use derive_setters::Setters;
use iced::{window, Length};
use iced_core::{renderer::Quad, widget::tree, Background, Color, Renderer, Widget};
use std::{borrow::Cow, process::Child};

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
        window_id: None,
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

    /// The window id for the headerbar.
    #[setters(strip_option)]
    window_id: Option<iced::window::Id>,

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

    /// Build the widget
    #[must_use]
    pub fn build(self) -> HeaderBarWidget<'a, Message> {
        HeaderBarWidget {
            window_id: self.window_id,
            header_bar_inner: self.into_element(),
        }
    }
}

pub struct HeaderBarWidget<'a, Message> {
    header_bar_inner: Element<'a, Message>,
    window_id: Option<iced::window::Id>,
}

impl<'a, Message: Clone + 'static> Widget<Message, crate::Renderer>
    for HeaderBarWidget<'a, Message>
{
    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.header_bar_inner)]
    }

    fn width(&self) -> Length {
        self.header_bar_inner.width()
    }

    fn height(&self) -> Length {
        self.header_bar_inner.height()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MyState>()
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        tree.diff_children(&mut [&mut self.header_bar_inner]);
        let prev = tree.state.downcast_mut::<MyState>();
        if prev.window_id != self.window_id {
            *prev = MyState::new(self.window_id);
        }
    }

    fn state(&self) -> tree::State {
        let state = MyState::new(self.window_id);
        tree::State::new(state)
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let child_tree = &mut tree.children[0];
        let child = self
            .header_bar_inner
            .as_widget()
            .layout(child_tree, renderer, limits);
        iced_core::layout::Node::with_children(child.size(), vec![child])
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut crate::Renderer,
        theme: &<crate::Renderer as iced_core::Renderer>::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        let layout_children = layout.children().next().unwrap();
        let state_children = &tree.children[0];
        self.header_bar_inner.as_widget().draw(
            state_children,
            renderer,
            theme,
            style,
            layout_children,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<MyState>();
        if !state.window_has_focus {
            let header_bar_appearance =
                <crate::Theme as crate::iced_style::container::StyleSheet>::appearance(
                    theme,
                    &crate::theme::Container::HeaderBar,
                );
            let cosmic = theme.cosmic();
            let mut neutral_0 = cosmic.palette.neutral_0;
            neutral_0.alpha = 0.3;

            // draw overlay rectangle
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    border_radius: header_bar_appearance.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color(neutral_0.into()),
            );
        }
    }

    fn on_event(
        &mut self,
        state: &mut tree::Tree,
        event: iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) -> iced_core::event::Status {
        if let iced_core::Event::Window(id, iced_core::window::Event::Focused) = event {
            let state = state.state.downcast_mut::<MyState>();
            state.focus_window(id);
        } else if let iced_core::Event::Window(id, iced_core::window::Event::Unfocused) = event {
            let state = state.state.downcast_mut::<MyState>();
            state.unfocus_window(id);
        }

        let child_state = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget_mut().on_event(
            child_state,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &crate::Renderer,
    ) -> iced_core::mouse::Interaction {
        let child_tree = &state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget().mouse_interaction(
            child_tree,
            child_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<
            iced_core::widget::OperationOutputWrapper<Message>,
        >,
    ) {
        let child_tree = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner
            .as_widget()
            .operate(child_tree, child_layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Renderer>> {
        let child_tree = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner
            .as_widget_mut()
            .overlay(child_tree, child_layout, renderer)
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
                    .width(Length::Shrink),
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
                    .width(Length::Shrink),
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
            widget::icon::from_svg_bytes(icon_bytes)
                .symbolic(true)
                .apply(widget::button::icon)
                .icon_size(size)
                .on_press(on_press)
        };

        widget::row::with_capacity(3)
            .push_maybe(self.on_minimize.take().map(|m| {
                icon(
                    &include_bytes!("../../res/icons/window-minimize-symbolic.svg")[..],
                    16,
                    m,
                )
            }))
            .push_maybe(self.on_maximize.take().map(|m| {
                icon(
                    &include_bytes!("../../res/icons/window-maximize-symbolic.svg")[..],
                    16,
                    m,
                )
            }))
            .push_maybe(self.on_close.take().map(|m| {
                icon(
                    &include_bytes!("../../res/icons/window-close-symbolic.svg")[..],
                    16,
                    m,
                )
            }))
            .spacing(8)
            .apply(widget::container)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBar<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBar<'a, Message>) -> Self {
        Element::new(headerbar.build())
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBarWidget<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBarWidget<'a, Message>) -> Self {
        Element::new(headerbar)
    }
}

pub struct MyState {
    pub window_id: Option<window::Id>,
    pub window_has_focus: bool,
}

impl MyState {
    pub fn new(id: Option<window::Id>) -> Self {
        Self {
            window_id: id,
            window_has_focus: id.is_none(),
        }
    }

    pub fn focus_window(&mut self, id: window::Id) {
        if self.window_id != Some(id) {
            return;
        }
        self.window_has_focus = true;
    }

    pub fn unfocus_window(&mut self, id: window::Id) {
        if self.window_id != Some(id) {
            return;
        }
        self.window_has_focus = false;
    }
}
