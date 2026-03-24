// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::cosmic_theme::{Density, Spacing};
use crate::{Element, theme, widget};
use apply::Apply;
use derive_setters::Setters;
use iced_core::{Length, Size, Vector, Widget, layout, text, widget::tree};
use std::borrow::Cow;

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
        transparent: false,
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

    /// Whether the headerbar should be transparent
    transparent: bool,
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

pub struct HeaderBarWidget<'a, Message> {
    start: Element<'a, Message>,
    center: Option<Element<'a, Message>>,
    end: Element<'a, Message>,
}

impl<'a, Message> HeaderBarWidget<'a, Message> {
    pub fn new(
        start: Element<'a, Message>,
        center: Option<Element<'a, Message>>,
        end: Element<'a, Message>,
    ) -> Self {
        Self { start, center, end }
    }

    fn elems(&self) -> impl Iterator<Item = &Element<'a, Message>> {
        std::iter::once(&self.start)
            .chain(std::iter::once(&self.end))
            .chain(self.center.as_ref())
    }

    fn elems_mut(&mut self) -> impl Iterator<Item = &mut Element<'a, Message>> {
        std::iter::once(&mut self.start)
            .chain(std::iter::once(&mut self.end))
            .chain(self.center.as_mut())
    }
}

impl<'a, Message: Clone + 'static> Widget<Message, crate::Theme, crate::Renderer>
    for HeaderBarWidget<'a, Message>
{
    fn diff(&mut self, tree: &mut tree::Tree) {
        if let Some(center) = &mut self.center {
            tree.diff_children(&mut [&mut self.start, &mut self.end, center]);
        } else {
            tree.diff_children(&mut [&mut self.start, &mut self.end]);
        }
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.elems().map(tree::Tree::new).collect()
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let height = limits.max().height;
        let gap = 8.0;

        let end_node =
            self.end
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits.loose());
        let end_width = end_node.size().width;

        let start_available = (width - end_width - gap).max(0.0);
        let start_node = self.start.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(start_available, height)),
        );
        let start_width = start_node.size().width;

        let vcenter = |node: layout::Node, x: f32| -> layout::Node {
            let dy = ((height - node.size().height) / 2.0).max(0.0);
            node.translate(Vector::new(x, dy))
        };

        let mut child_nodes = Vec::with_capacity(3);
        child_nodes.push(vcenter(start_node, 0.0));
        child_nodes.push(vcenter(end_node, width - end_width));

        if let Some(center) = &mut self.center {
            let slot_start = start_width + gap;
            let slot_end = (width - end_width - gap).max(slot_start);
            let slot_width = slot_end - slot_start;
            // this instead of `node.size().width` prevents center jitter as text ellipsizes
            let natural_width = center
                .as_widget_mut()
                .layout(&mut tree.children[2], renderer, &limits.loose())
                .size()
                .width;

            let node = center.as_widget_mut().layout(
                &mut tree.children[2],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(slot_width, height)),
            );

            let ideal_x = (width - natural_width) / 2.0;
            let max_x = (width - end_width - gap - natural_width).max(slot_start);
            let center_x = ideal_x.clamp(slot_start, max_x);

            child_nodes.push(vcenter(node, center_x))
        }

        layout::Node::with_children(Size::new(width, height), child_nodes)
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
        for ((e, s), l) in self.elems().zip(&tree.children).zip(layout.children()) {
            e.as_widget()
                .draw(s, renderer, theme, style, l, cursor, viewport);
        }
    }

    fn update(
        &mut self,
        state: &mut tree::Tree,
        event: &iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) {
        for ((e, s), l) in self
            .elems_mut()
            .zip(&mut state.children)
            .zip(layout.children())
        {
            e.as_widget_mut()
                .update(s, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &crate::Renderer,
    ) -> iced_core::mouse::Interaction {
        self.elems()
            .zip(&state.children)
            .zip(layout.children())
            .map(|((e, s), l)| {
                e.as_widget()
                    .mouse_interaction(s, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or(iced_core::mouse::Interaction::None)
    }

    fn operate(
        &mut self,
        state: &mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        for ((e, s), l) in self
            .elems_mut()
            .zip(&mut state.children)
            .zip(layout.children())
        {
            e.as_widget_mut().operate(s, l, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: iced_core::Layout<'b>,
        renderer: &crate::Renderer,
        viewport: &iced_core::Rectangle,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let mut layouts = layout.children();
        let mut try_overlay = |elem: &'b mut Element<'a, Message>,
                               state: &'b mut tree::Tree|
         -> Option<
            iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>,
        > {
            elem.as_widget_mut()
                .overlay(state, layouts.next()?, renderer, viewport, translation)
        };

        if let Some(center) = &mut self.center {
            let (start_slice, end_center) = state.children.split_at_mut(1);
            let (end_slice, center_slice) = end_center.split_at_mut(1);
            try_overlay(&mut self.start, &mut start_slice[0])
                .or_else(|| try_overlay(&mut self.end, &mut end_slice[0]))
                .or_else(|| try_overlay(center, &mut center_slice[0]))
        } else {
            let (start_slice, end_slice) = state.children.split_at_mut(1);
            try_overlay(&mut self.start, &mut start_slice[0])
                .or_else(|| try_overlay(&mut self.end, &mut end_slice[0]))
        }
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        for ((e, s), l) in self.elems().zip(&state.children).zip(layout.children()) {
            e.as_widget()
                .drag_destinations(s, l, renderer, dnd_rectangles);
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
        iced_accessibility::A11yTree::join(
            self.elems()
                .zip(&state.children)
                .zip(layout.children())
                .map(|((e, s), l)| e.as_widget().a11y_nodes(l, s, p)),
        )
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBarWidget<'a, Message>> for Element<'a, Message> {
    fn from(w: HeaderBarWidget<'a, Message>) -> Self {
        Element::new(w)
    }
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
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

        // Also packs the window controls at the very end.
        end.push(self.window_controls(space_xxs));

        let padding = if self.is_ssd {
            [2, 8, 2, 8]
        } else {
            match (
                self.density.unwrap_or_else(crate::config::header_size),
                self.maximized, // window border handling
            ) {
                (Density::Compact, true) => [4, 8, 4, 8],
                (Density::Compact, false) => [3, 7, 4, 7],
                (_, true) => [8, 8, 8, 8],
                (_, false) => [7, 7, 8, 7],
            }
        };

        let start = widget::row::with_children(start)
            .spacing(space_xxxs)
            .align_y(iced::Alignment::Center)
            .into();
        let center = if !center.is_empty() {
            Some(
                widget::row::with_children(center)
                    .spacing(space_xxxs)
                    .align_y(iced::Alignment::Center)
                    .into(),
            )
        } else if !self.title.is_empty() {
            Some(
                widget::text::heading(self.title)
                    .wrapping(text::Wrapping::None)
                    .ellipsize(text::Ellipsize::End(text::EllipsizeHeightLimit::Lines(1)))
                    .into(),
            )
        } else {
            None
        };
        let end = widget::row::with_children(end)
            .spacing(space_xxs)
            .align_y(iced::Alignment::Center)
            .into();

        let mut widget = HeaderBarWidget::new(start, center, end)
            .apply(widget::container)
            .class(crate::theme::Container::HeaderBar {
                focused: self.focused,
                sharp_corners: self.sharp_corners,
                transparent: self.transparent,
            })
            .height(Length::Fixed(32.0 + padding[0] as f32 + padding[2] as f32))
            .padding(padding)
            .apply(widget::mouse_area);

        if let Some(message) = self.on_drag {
            widget = widget.on_drag(message);
        }
        if let Some(message) = self.on_maximize {
            widget = widget.on_release(message);
        }
        if let Some(message) = self.on_double_click {
            widget = widget.on_double_press(message);
        }
        if let Some(message) = self.on_right_click {
            widget = widget.on_right_press(message);
        }

        widget.into()
    }

    /// Creates the widget for window controls.
    fn window_controls(&mut self, spacing: u16) -> Element<'a, Message> {
        macro_rules! icon {
            ($name:expr, $size:expr, $on_press:expr) => {{
                widget::icon::from_name($name)
                    .apply(widget::button::icon)
                    .padding(8)
                    .class(crate::theme::Button::HeaderBar)
                    .selected(self.focused)
                    .icon_size($size)
                    .on_press($on_press)
            }};
        }

        widget::row::with_capacity(3)
            .push_maybe(
                self.on_minimize
                    .take()
                    .map(|m| icon!("window-minimize-symbolic", 16, m)),
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
            .spacing(spacing)
            .align_y(iced::Alignment::Center)
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBar<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBar<'a, Message>) -> Self {
        headerbar.view()
    }
}
