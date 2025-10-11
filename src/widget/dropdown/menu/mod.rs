// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

mod appearance;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

pub use appearance::{Appearance, StyleSheet};

use crate::surface;
use crate::widget::{Container, RcWrapper, icon};
use iced_core::event::{self, Event};
use iced_core::layout::{self, Layout};
use iced_core::text::{self, Text};
use iced_core::widget::Tree;
use iced_core::{
    Border, Clipboard, Element, Length, Padding, Pixels, Point, Rectangle, Renderer, Shadow, Shell,
    Size, Vector, Widget, alignment, mouse, overlay, renderer, svg, touch,
};
use iced_widget::scrollable::Scrollable;

/// A list of selectable options.
#[must_use]
pub struct Menu<'a, S, Message>
where
    S: AsRef<str>,
    [S]: std::borrow::ToOwned,
{
    state: State,
    options: Cow<'a, [S]>,
    icons: Cow<'a, [icon::Handle]>,
    hovered_option: Arc<Mutex<Option<usize>>>,
    selected_option: Option<usize>,
    on_selected: Box<dyn FnMut(usize) -> Message + 'a>,
    close_on_selected: Option<Message>,
    on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    width: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    style: (),
}

impl<'a, S: AsRef<str>, Message: 'a + std::clone::Clone> Menu<'a, S, Message>
where
    [S]: std::borrow::ToOwned,
{
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub fn new(
        state: State,
        options: Cow<'a, [S]>,
        icons: Cow<'a, [icon::Handle]>,
        hovered_option: Arc<Mutex<Option<usize>>>,
        selected_option: Option<usize>,
        on_selected: impl FnMut(usize) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
        close_on_selected: Option<Message>,
    ) -> Self {
        Menu {
            state,
            options,
            icons,
            hovered_option,
            selected_option,
            on_selected: Box::new(on_selected),
            on_option_hovered,
            width: 0.0,
            padding: Padding::ZERO,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            style: Default::default(),
            close_on_selected,
        }
    }

    /// Sets the width of the [`Menu`].
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`Menu`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`Menu`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Menu`].
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Turns the [`Menu`] into an overlay [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`Menu`].
    #[must_use]
    pub fn overlay(
        self,
        position: Point,
        target_height: f32,
    ) -> overlay::Element<'a, Message, crate::Theme, crate::Renderer> {
        overlay::Element::new(Box::new(Overlay::new(self, target_height, position)))
    }

    /// Turns the [`Menu`] into a popup [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`Menu`].
    #[must_use]
    pub fn popup(self, position: Point, target_height: f32) -> crate::Element<'a, Message> {
        Overlay::new(self, target_height, position).into()
    }
}

/// The local state of a [`Menu`].
#[must_use]
#[derive(Debug, Clone)]
pub struct State {
    pub(crate) tree: RcWrapper<Tree>,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    pub fn new() -> Self {
        Self {
            tree: RcWrapper::new(Tree::empty()),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Overlay<'a, Message> {
    state: RcWrapper<Tree>,
    container: Container<'a, Message, crate::Theme, crate::Renderer>,
    width: f32,
    target_height: f32,
    style: (),
    position: Point,
}

impl<'a, Message: Clone + 'a> Overlay<'a, Message> {
    pub fn new<S: AsRef<str>>(
        menu: Menu<'a, S, Message>,
        target_height: f32,
        position: Point,
    ) -> Self
    where
        [S]: ToOwned,
    {
        let Menu {
            state,
            options,
            icons,
            hovered_option,
            selected_option,
            on_selected,
            on_option_hovered,
            width,
            padding,
            text_size,
            text_line_height,
            style,
            close_on_selected,
        } = menu;

        let mut container = Container::new(Scrollable::new(
            Container::new(List {
                options,
                icons,
                hovered_option,
                selected_option,
                on_selected,
                close_on_selected,
                on_option_hovered,
                text_size,
                text_line_height,
                padding,
            })
            .padding(padding),
        ))
        .class(crate::style::Container::Dropdown);

        state
            .tree
            .with_data_mut(|tree| tree.diff(&mut container as &mut dyn Widget<_, _, _>));

        Self {
            state: state.tree,
            container,
            width,
            target_height,
            style,
            position,
        }
    }

    fn _layout(&self, renderer: &crate::Renderer, bounds: Size) -> layout::Node {
        let space_below = bounds.height - (self.position.y + self.target_height);
        let space_above = self.position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - self.position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(self.width);

        let node = self
            .state
            .with_data_mut(|tree| self.container.layout(tree, renderer, &limits));

        let node_size = node.size();
        node.move_to(if space_below > space_above {
            self.position + Vector::new(0.0, self.target_height)
        } else {
            self.position - Vector::new(0.0, node_size.height)
        })
    }

    fn _on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();

        self.state.with_data_mut(|tree| {
            self.container.on_event(
                tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
            )
        })
    }

    fn _mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self.state.with_data(|tree| {
            self.container
                .mouse_interaction(tree, layout, cursor, viewport, renderer)
        })
    }

    fn _draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let appearance = theme.appearance(&self.style);
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    width: appearance.border_width,
                    color: appearance.border_color,
                    radius: appearance.border_radius,
                },
                shadow: Shadow::default(),
            },
            appearance.background,
        );

        self.state.with_data(|tree| {
            self.container
                .draw(tree, renderer, theme, style, layout, cursor, &bounds)
        })
    }
}

impl<'a, Message: Clone + 'a> iced_core::Overlay<Message, crate::Theme, crate::Renderer>
    for Overlay<'a, Message>
{
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> layout::Node {
        self._layout(renderer, bounds)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self._on_event(event, layout, cursor, renderer, clipboard, shell)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self._mouse_interaction(layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self._draw(renderer, theme, style, layout, cursor);
    }
}

impl<'a, Message: Clone + 'a> crate::widget::Widget<Message, crate::Theme, crate::Renderer>
    for Overlay<'a, Message>
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Shrink)
    }

    fn layout(
        &self,
        _tree: &mut iced_core::widget::Tree,
        renderer: &crate::Renderer,
        limits: &iced::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);

        self.state
            .with_data_mut(|tree| self.container.layout(tree, renderer, &limits))
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self._mouse_interaction(layout, cursor, viewport, renderer)
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        self._on_event(event, layout, cursor, renderer, clipboard, shell)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        self._draw(renderer, theme, style, layout, cursor);
    }
}

impl<'a, Message: Clone + 'a> From<Overlay<'a, Message>> for crate::Element<'a, Message> {
    fn from(widget: Overlay<'a, Message>) -> Self {
        Element::new(widget)
    }
}

struct List<'a, S: AsRef<str>, Message>
where
    [S]: std::borrow::ToOwned,
{
    options: Cow<'a, [S]>,
    icons: Cow<'a, [icon::Handle]>,
    hovered_option: Arc<Mutex<Option<usize>>>,
    selected_option: Option<usize>,
    on_selected: Box<dyn FnMut(usize) -> Message + 'a>,
    close_on_selected: Option<Message>,
    on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
}

impl<S: AsRef<str>, Message> Widget<Message, crate::Theme, crate::Renderer> for List<'_, S, Message>
where
    [S]: std::borrow::ToOwned,
    Message: Clone,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self
            .text_size
            .unwrap_or_else(|| text::Renderer::default_size(renderer).0);

        let text_line_height = self.text_line_height.to_absolute(Pixels(text_size));

        let size = {
            let intrinsic = Size::new(
                0.0,
                (f32::from(text_line_height) + self.padding.vertical()) * self.options.len() as f32,
            );

            limits.resolve(Length::Fill, Length::Shrink, intrinsic)
        };

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let hovered_guard = self.hovered_option.lock().unwrap();
                if cursor.is_over(layout.bounds()) {
                    if let Some(index) = *hovered_guard {
                        shell.publish((self.on_selected)(index));
                        if let Some(close_on_selected) = self.close_on_selected.as_ref() {
                            shell.publish(close_on_selected.clone());
                        }
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer).0);

                    let option_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
                            + self.padding.vertical();

                    let new_hovered_option = (cursor_position.y / option_height) as usize;
                    let mut hovered_guard = self.hovered_option.lock().unwrap();

                    if let Some(on_option_hovered) = self.on_option_hovered {
                        if *hovered_guard != Some(new_hovered_option) {
                            shell.publish(on_option_hovered(new_hovered_option));
                        }
                    }

                    *hovered_guard = Some(new_hovered_option);
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer).0);

                    let option_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
                            + self.padding.vertical();
                    let mut hovered_guard = self.hovered_option.lock().unwrap();

                    *hovered_guard = Some((cursor_position.y / option_height) as usize);

                    if let Some(index) = *hovered_guard {
                        shell.publish((self.on_selected)(index));
                        if let Some(close_on_selected) = self.close_on_selected.as_ref() {
                            shell.publish(close_on_selected.clone());
                        }
                        return event::Status::Captured;
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let appearance = theme.appearance(&());
        let bounds = layout.bounds();

        let text_size = self
            .text_size
            .unwrap_or_else(|| text::Renderer::default_size(renderer).0);
        let option_height = f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
            + self.padding.vertical();

        let offset = viewport.y - bounds.y;
        let start = (offset / option_height) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;

        let visible_options = &self.options[start..end.min(self.options.len())];

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;

            let bounds = Rectangle {
                x: bounds.x,
                y: option_height.mul_add(i as f32, bounds.y),
                width: bounds.width,
                height: option_height,
            };

            let hovered_guard = self.hovered_option.lock().unwrap();

            let (color, font) = if self.selected_option == Some(i) {
                let item_x = bounds.x + appearance.border_width;
                let item_width = appearance.border_width.mul_add(-2.0, bounds.width);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: item_x,
                            width: item_width,
                            ..bounds
                        },
                        border: Border {
                            radius: appearance.border_radius,
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    appearance.selected_background,
                );

                let svg_handle =
                    iced_core::Svg::new(crate::widget::common::object_select().clone())
                        .color(appearance.selected_text_color)
                        .border_radius(appearance.border_radius);

                svg::Renderer::draw_svg(
                    renderer,
                    svg_handle,
                    Rectangle {
                        x: item_x + item_width - 16.0 - 8.0,
                        y: bounds.y + (bounds.height / 2.0 - 8.0),
                        width: 16.0,
                        height: 16.0,
                    },
                );

                (appearance.selected_text_color, crate::font::semibold())
            } else if *hovered_guard == Some(i) {
                let item_x = bounds.x + appearance.border_width;
                let item_width = appearance.border_width.mul_add(-2.0, bounds.width);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: item_x,
                            width: item_width,
                            ..bounds
                        },
                        border: Border {
                            radius: appearance.border_radius,
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    appearance.hovered_background,
                );

                (appearance.hovered_text_color, crate::font::default())
            } else {
                (appearance.text_color, crate::font::default())
            };

            let mut bounds = Rectangle {
                x: bounds.x + self.padding.left,
                y: bounds.center_y(),
                width: f32::INFINITY,
                ..bounds
            };

            if let Some(handle) = self.icons.get(i) {
                let icon_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + 8.0 - (bounds.height / 2.0),
                    width: 20.0,
                    height: 20.0,
                };

                bounds.x += 24.0;
                icon::draw(renderer, handle, icon_bounds);
            }

            text::Renderer::fill_text(
                renderer,
                Text {
                    content: option.as_ref().to_string(),
                    bounds: bounds.size(),
                    size: Pixels(text_size),
                    line_height: self.text_line_height,
                    font,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::default(),
                },
                bounds.position(),
                color,
                *viewport,
            );
        }
    }
}

impl<'a, S: AsRef<str>, Message: 'a> From<List<'a, S, Message>>
    for Element<'a, Message, crate::Theme, crate::Renderer>
where
    [S]: std::borrow::ToOwned,
    Message: Clone,
{
    fn from(list: List<'a, S, Message>) -> Self {
        Element::new(list)
    }
}
