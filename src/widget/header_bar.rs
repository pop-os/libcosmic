// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::cosmic_theme::{Density, Spacing};
use crate::{Element, theme, widget};
use apply::Apply;
use derive_setters::Setters;
use iced::Length;
use iced_core::{Vector, Widget, widget::tree};
use std::{borrow::Cow, cmp};

#[must_use]
pub fn header_bar<'a, Message>() -> HeaderBar<'a, Message> {
    HeaderBar {
        title: Cow::Borrowed(""),
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
        on_right_click: None,
        start: Vec::new(),
        center: Vec::new(),
        end: Vec::new(),
        density: None,
        focused: false,
        maximized: false,
        sharp_corners: false,
        is_ssd: false,
        on_double_click: None,
        is_condensed: false,
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

    /// A message emitted when the header is double clicked,
    /// usually used to maximize the window.
    #[setters(strip_option)]
    on_double_click: Option<Message>,

    /// A message emitted when the header is right clicked.
    #[setters(strip_option)]
    on_right_click: Option<Message>,

    /// Elements packed at the start of the headerbar.
    #[setters(skip)]
    start: Vec<Element<'a, Message>>,

    /// Elements packed in the center of the headerbar.
    #[setters(skip)]
    center: Vec<Element<'a, Message>>,

    /// Elements packed at the end of the headerbar.
    #[setters(skip)]
    end: Vec<Element<'a, Message>>,

    /// Controls the density of the headerbar.
    #[setters(strip_option)]
    density: Option<Density>,

    /// Focused state of the window
    focused: bool,

    /// Maximized state of the window
    maximized: bool,

    /// Whether the corners of the window should be sharp
    sharp_corners: bool,

    /// HeaderBar used for server-side decorations
    is_ssd: bool,

    /// Whether the headerbar should be compact
    is_condensed: bool,
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
    #[inline]
    pub fn build(self) -> HeaderBarWidget<'a, Message> {
        HeaderBarWidget {
            header_bar_inner: self.view(),
        }
    }
}

pub struct HeaderBarWidget<'a, Message> {
    header_bar_inner: Element<'a, Message>,
}

impl<Message: Clone + 'static> Widget<Message, crate::Theme, crate::Renderer>
    for HeaderBarWidget<'_, Message>
{
    fn diff(&mut self, tree: &mut tree::Tree) {
        tree.diff_children(&mut [&mut self.header_bar_inner]);
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.header_bar_inner)]
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.header_bar_inner.as_widget().size()
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
        theme: &crate::Theme,
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
        operation: &mut dyn iced_core::widget::Operation<()>,
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
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let child_tree = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget_mut().overlay(
            child_tree,
            child_layout,
            renderer,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        if let Some((child_tree, child_layout)) =
            state.children.iter().zip(layout.children()).next()
        {
            self.header_bar_inner.as_widget().drag_destinations(
                child_tree,
                child_layout,
                renderer,
                dnd_rectangles,
            );
        }
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &tree::Tree,
        p: iced::mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let c_layout = layout.children().next().unwrap();
        let c_state = &state.children[0];
        self.header_bar_inner
            .as_widget()
            .a11y_nodes(c_layout, c_state, p)
    }
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    #[allow(clippy::too_many_lines)]
    /// Converts the headerbar builder into an Iced element.
    pub fn view(mut self) -> Element<'a, Message> {
        let Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::spacing();

        // Take ownership of the regions to be packed.
        let start = std::mem::take(&mut self.start);
        let center = std::mem::take(&mut self.center);
        let mut end = std::mem::take(&mut self.end);

        let window_control_cnt = self.on_close.is_some() as usize
            + self.on_maximize.is_some() as usize
            + self.on_minimize.is_some() as usize;
        // Also packs the window controls at the very end.
        end.push(self.window_controls());

        // Center content depending on window border
        let padding = match self.density.unwrap_or_else(crate::config::header_size) {
            Density::Compact => {
                if self.maximized {
                    [4, 8, 4, 8]
                } else {
                    [3, 7, 4, 7]
                }
            }
            _ => {
                if self.maximized {
                    [8, 8, 8, 8]
                } else {
                    [7, 7, 8, 7]
                }
            }
        };

        let acc_count = |v: &[Element<'a, Message>]| {
            v.iter().fold(0, |acc, e| {
                acc + match e.as_widget().size().width {
                    Length::Fixed(w) if w > 30. => (w / 30.0).ceil() as usize,
                    _ => 1,
                }
            })
        };

        let left_len = acc_count(&start);
        let right_len = acc_count(&end);

        let portion = ((left_len.max(right_len + window_control_cnt) as f32
            / center.len().max(1) as f32)
            .round() as u16)
            .max(1);
        let (left_portion, right_portion) =
            if center.is_empty() && (self.title.is_empty() || self.is_condensed) {
                let left_to_right_ratio = left_len as f32 / right_len.max(1) as f32;
                let right_to_left_ratio = right_len as f32 / left_len.max(1) as f32;
                if right_to_left_ratio > 2. || left_len < 1 {
                    (1, 2)
                } else if left_to_right_ratio > 2. || right_len < 1 {
                    (2, 1)
                } else {
                    (left_len as u16, (right_len + window_control_cnt) as u16)
                }
            } else {
                (portion, portion)
            };
        let title_portion = cmp::max(left_portion, right_portion) * 2;
        // Creates the headerbar widget.
        let mut widget = widget::row::with_capacity(3)
            // If elements exist in the start region, append them here.
            .push(
                widget::row::with_children(start)
                    .spacing(space_xxxs)
                    .align_y(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::Alignment::Start)
                    .width(Length::FillPortion(left_portion)),
            )
            // If elements exist in the center region, use them here.
            // This will otherwise use the title as a widget if a title was defined.
            .push_maybe(if !center.is_empty() {
                Some(
                    widget::row::with_children(center)
                        .spacing(space_xxxs)
                        .align_y(iced::Alignment::Center)
                        .apply(widget::container)
                        .center_x(Length::Fill)
                        .into(),
                )
            } else if !self.title.is_empty() && !self.is_condensed {
                Some(self.title_widget(title_portion))
            } else {
                None
            })
            .push(
                widget::row::with_children(end)
                    .spacing(space_xxs)
                    .align_y(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::Alignment::End)
                    .width(Length::FillPortion(right_portion)),
            )
            .align_y(iced::Alignment::Center)
            .height(Length::Fixed(32.0 + padding[0] as f32 + padding[2] as f32))
            .padding(if self.is_ssd { [0, 8, 0, 8] } else { padding })
            .spacing(8)
            .apply(widget::container)
            .class(crate::theme::Container::HeaderBar {
                focused: self.focused,
                sharp_corners: self.sharp_corners,
            })
            .center_y(Length::Shrink)
            .apply(widget::mouse_area);

        // Assigns a message to emit when the headerbar is dragged.
        if let Some(message) = self.on_drag.clone() {
            widget = widget.on_drag(message);
        }

        // Assigns a message to emit when the headerbar is double-clicked.
        if let Some(message) = self.on_maximize.clone() {
            widget = widget.on_release(message);
        }
        if let Some(message) = self.on_double_click.clone() {
            widget = widget.on_double_press(message);
        }
        if let Some(message) = self.on_right_click.clone() {
            widget = widget.on_right_press(message);
        }

        widget.into()
    }

    fn title_widget(&mut self, title_portion: u16) -> Element<'a, Message> {
        let mut title = Cow::default();
        std::mem::swap(&mut title, &mut self.title);

        widget::text::heading(title)
            .apply(widget::container)
            .center(Length::FillPortion(title_portion))
            .into()
    }

    /// Creates the widget for window controls.
    fn window_controls(&mut self) -> Element<'a, Message> {
        macro_rules! icon {
            ($name:expr, $size:expr, $on_press:expr) => {{
                let icon = {
                    widget::icon::from_name($name)
                        .apply(widget::button::icon)
                        .padding(8)
                };

                icon.class(crate::theme::Button::HeaderBar)
                    .selected(self.focused)
                    .icon_size($size)
                    .on_press($on_press)
            }};
        }

        widget::row::with_capacity(3)
            .push_maybe(
                self.on_minimize
                    .take()
                    .map(|m: Message| icon!("window-minimize-symbolic", 16, m)),
            )
            .push_maybe(self.on_maximize.take().map(|m| {
                if self.maximized {
                    icon!("window-restore-symbolic", 16, m)
                } else {
                    icon!("window-maximize-symbolic", 16, m)
                }
            }))
            .push_maybe(
                self.on_close
                    .take()
                    .map(|m| icon!("window-close-symbolic", 16, m)),
            )
            .spacing(theme::spacing().space_xxs)
            .apply(widget::container)
            .center_y(Length::Fill)
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
