// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

mod appearance;
pub use appearance::{Appearance, StyleSheet};

use std::ffi::OsStr;

use crate::widget::{icon, Container};
use iced_core::event::{self, Event};
use iced_core::layout::{self, Layout};
use iced_core::text::{self, Text};
use iced_core::widget::Tree;
use iced_core::{
    alignment, mouse, overlay, renderer, svg, touch, Clipboard, Color, Element, Length, Padding,
    Pixels, Point, Rectangle, Renderer, Shell, Size, Vector, Widget,
};
use iced_widget::scrollable::Scrollable;

/// A list of selectable options.
#[must_use]
pub struct Menu<'a, S, Message>
where
    S: AsRef<str>,
{
    state: &'a mut State,
    options: &'a [S],
    hovered_option: &'a mut Option<usize>,
    selected_option: Option<usize>,
    on_selected: Box<dyn FnMut(usize) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    width: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    style: (),
}

impl<'a, S: AsRef<str>, Message: 'a> Menu<'a, S, Message> {
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub fn new(
        state: &'a mut State,
        options: &'a [S],
        hovered_option: &'a mut Option<usize>,
        selected_option: Option<usize>,
        on_selected: impl FnMut(usize) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    ) -> Self {
        Menu {
            state,
            options,
            hovered_option,
            selected_option,
            on_selected: Box::new(on_selected),
            on_option_hovered,
            width: 0.0,
            padding: Padding::ZERO,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            style: Default::default(),
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
    ) -> overlay::Element<'a, Message, crate::Renderer> {
        overlay::Element::new(position, Box::new(Overlay::new(self, target_height)))
    }
}

/// The local state of a [`Menu`].
#[must_use]
#[derive(Debug)]
pub struct State {
    tree: Tree,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    pub fn new() -> Self {
        Self {
            tree: Tree::empty(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Overlay<'a, Message> {
    state: &'a mut Tree,
    container: Container<'a, Message, crate::Renderer>,
    width: f32,
    target_height: f32,
    style: (),
}

impl<'a, Message: 'a> Overlay<'a, Message> {
    pub fn new<S: AsRef<str>>(menu: Menu<'a, S, Message>, target_height: f32) -> Self {
        let Menu {
            state,
            options,
            hovered_option,
            selected_option,
            on_selected,
            on_option_hovered,
            width,
            padding,
            text_size,
            text_line_height,
            style,
        } = menu;

        let selected_icon = icon::from_name("object-select-symbolic").size(16).handle();

        let mut container = Container::new(Scrollable::new(List {
            options,
            hovered_option,
            selected_option,
            on_selected,
            on_option_hovered,
            text_size,
            text_line_height,
            padding,
            selected_icon: match selected_icon.data {
                icon::Data::Name(named) => named
                    .path()
                    .filter(|path| path.extension().is_some_and(|ext| ext == OsStr::new("svg")))
                    .map(iced_core::svg::Handle::from_path),
                icon::Data::Svg(handle) => Some(handle),
                icon::Data::Image(_) => None,
            },
        }));

        container = container
            .padding(padding)
            .style(crate::style::Container::Dropdown);

        state.tree.diff(&mut container as &mut dyn Widget<_, _>);

        Self {
            state: &mut state.tree,
            container,
            width,
            target_height,
            style,
        }
    }
}

impl<'a, Message> iced_core::Overlay<Message, crate::Renderer> for Overlay<'a, Message> {
    fn layout(&self, renderer: &crate::Renderer, bounds: Size, position: Point) -> layout::Node {
        let space_below = bounds.height - (position.y + self.target_height);
        let space_above = position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(self.width);

        let mut node = self.container.layout(renderer, &limits);

        node.move_to(if space_below > space_above {
            position + Vector::new(0.0, self.target_height)
        } else {
            position - Vector::new(0.0, node.size().height)
        });

        node
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
        let bounds = layout.bounds();

        self.container.on_event(
            self.state, event, layout, cursor, renderer, clipboard, shell, &bounds,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self.container
            .mouse_interaction(self.state, layout, cursor, viewport, renderer)
    }

    fn draw(
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
                border_color: appearance.border_color,
                border_width: appearance.border_width,
                border_radius: appearance.border_radius,
            },
            appearance.background,
        );

        self.container
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }
}

struct List<'a, S: AsRef<str>, Message> {
    options: &'a [S],
    hovered_option: &'a mut Option<usize>,
    selected_option: Option<usize>,
    on_selected: Box<dyn FnMut(usize) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    selected_icon: Option<svg::Handle>,
}

impl<'a, S: AsRef<str>, Message> Widget<Message, crate::Renderer> for List<'a, S, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &crate::Renderer, limits: &layout::Limits) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self
            .text_size
            .unwrap_or_else(|| text::Renderer::default_size(renderer));

        let text_line_height = self.text_line_height.to_absolute(Pixels(text_size));

        let size = {
            let intrinsic = Size::new(
                0.0,
                (f32::from(text_line_height) + self.padding.vertical()) * self.options.len() as f32,
            );

            limits.resolve(intrinsic)
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
                if cursor.is_over(layout.bounds()) {
                    if let Some(index) = *self.hovered_option {
                        shell.publish((self.on_selected)(index));
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer));

                    let option_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
                            + self.padding.vertical();

                    let new_hovered_option = (cursor_position.y / option_height) as usize;

                    if let Some(on_option_hovered) = self.on_option_hovered {
                        if *self.hovered_option != Some(new_hovered_option) {
                            shell.publish(on_option_hovered(new_hovered_option));
                        }
                    }

                    *self.hovered_option = Some(new_hovered_option);
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer));

                    let option_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)))
                            + self.padding.vertical();

                    *self.hovered_option = Some((cursor_position.y / option_height) as usize);

                    if let Some(index) = *self.hovered_option {
                        shell.publish((self.on_selected)(index));
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
        _state: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let appearance = theme.appearance(&());
        let bounds = layout.bounds();

        let text_size = self
            .text_size
            .unwrap_or_else(|| text::Renderer::default_size(renderer));
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
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                        border_radius: appearance.border_radius,
                    },
                    appearance.selected_background,
                );

                if let Some(handle) = self.selected_icon.clone() {
                    svg::Renderer::draw(
                        renderer,
                        handle,
                        Some(appearance.selected_text_color),
                        Rectangle {
                            x: item_x + item_width - 16.0 - 8.0,
                            y: bounds.y + (bounds.height / 2.0 - 8.0),
                            width: 16.0,
                            height: 16.0,
                        },
                    );
                }

                (appearance.selected_text_color, crate::font::FONT_SEMIBOLD)
            } else if *self.hovered_option == Some(i) {
                let item_x = bounds.x + appearance.border_width;
                let item_width = appearance.border_width.mul_add(-2.0, bounds.width);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: item_x,
                            width: item_width,
                            ..bounds
                        },
                        border_color: Color::TRANSPARENT,
                        border_width: 0.0,
                        border_radius: appearance.border_radius,
                    },
                    appearance.hovered_background,
                );

                (appearance.hovered_text_color, crate::font::FONT)
            } else {
                (appearance.text_color, crate::font::FONT)
            };

            text::Renderer::fill_text(
                renderer,
                Text {
                    content: option.as_ref(),
                    bounds: Rectangle {
                        x: bounds.x + self.padding.left,
                        y: bounds.center_y(),
                        width: f32::INFINITY,
                        ..bounds
                    },
                    size: text_size,
                    line_height: self.text_line_height,
                    font,
                    color,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                },
            );
        }
    }
}

impl<'a, S: AsRef<str>, Message: 'a> From<List<'a, S, Message>>
    for Element<'a, Message, crate::Renderer>
{
    fn from(list: List<'a, S, Message>) -> Self {
        Element::new(list)
    }
}
