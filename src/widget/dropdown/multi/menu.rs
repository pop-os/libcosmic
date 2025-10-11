use super::Model;
pub use crate::widget::dropdown::menu::{Appearance, StyleSheet};

use crate::widget::Container;
use iced_core::event::{self, Event};
use iced_core::layout::{self, Layout};
use iced_core::text::{self, Text};
use iced_core::widget::Tree;
use iced_core::{
    Border, Clipboard, Element, Length, Padding, Pixels, Point, Rectangle, Renderer, Shadow, Shell,
    Size, Vector, Widget, alignment, mouse, overlay, renderer, svg, touch,
};
use iced_widget::scrollable::Scrollable;

/// A dropdown menu with multiple lists.
#[must_use]
pub struct Menu<'a, S, Item, Message>
where
    S: AsRef<str>,
{
    state: &'a mut State,
    options: &'a Model<S, Item>,
    hovered_option: &'a mut Option<Item>,
    selected_option: Option<&'a Item>,
    on_selected: Box<dyn FnMut(Item) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(Item) -> Message>,
    width: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    style: (),
}

impl<'a, S, Item, Message: 'a> Menu<'a, S, Item, Message>
where
    S: AsRef<str>,
    Item: Clone + PartialEq,
{
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub(super) fn new(
        state: &'a mut State,
        options: &'a Model<S, Item>,
        hovered_option: &'a mut Option<Item>,
        selected_option: Option<&'a Item>,
        on_selected: impl FnMut(Item) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(Item) -> Message>,
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
            text_line_height: text::LineHeight::Absolute(Pixels::from(16.0)),
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
    ) -> overlay::Element<'a, Message, crate::Theme, crate::Renderer> {
        overlay::Element::new(Box::new(Overlay::new(self, target_height, position)))
    }
}

/// The local state of a [`Menu`].
#[must_use]
#[derive(Debug)]
pub(super) struct State {
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
    container: Container<'a, Message, crate::Theme, crate::Renderer>,
    width: f32,
    target_height: f32,
    style: (),
    position: Point,
}

impl<'a, Message: 'a> Overlay<'a, Message> {
    pub fn new<S: AsRef<str>, Item: Clone + PartialEq>(
        menu: Menu<'a, S, Item, Message>,
        target_height: f32,
        position: Point,
    ) -> Self {
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

        let mut container = Container::new(Scrollable::new(
            Container::new(InnerList {
                options,
                hovered_option,
                selected_option,
                on_selected,
                on_option_hovered,
                padding,
                text_size,
                text_line_height,
            })
            .padding(padding),
        ))
        .class(crate::style::Container::Dropdown);

        state.tree.diff(&mut container as &mut dyn Widget<_, _, _>);

        Self {
            state: &mut state.tree,
            container,
            width,
            target_height,
            style,
            position,
        }
    }
}

impl<Message> iced_core::Overlay<Message, crate::Theme, crate::Renderer> for Overlay<'_, Message> {
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> layout::Node {
        let position = self.position;
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

        let node = self.container.layout(self.state, renderer, &limits);

        let node_size = node.size();
        node.move_to(if space_below > space_above {
            position + Vector::new(0.0, self.target_height)
        } else {
            position - Vector::new(0.0, node_size.height)
        })
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
                border: Border {
                    width: appearance.border_width,
                    color: appearance.border_color,
                    radius: appearance.border_radius,
                },
                shadow: Shadow::default(),
            },
            appearance.background,
        );

        self.container
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }
}

struct InnerList<'a, S, Item, Message> {
    options: &'a Model<S, Item>,
    hovered_option: &'a mut Option<Item>,
    selected_option: Option<&'a Item>,
    on_selected: Box<dyn FnMut(Item) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(Item) -> Message>,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
}

impl<S, Item, Message> Widget<Message, crate::Theme, crate::Renderer>
    for InnerList<'_, S, Item, Message>
where
    S: AsRef<str>,
    Item: Clone + PartialEq,
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

        let lists = self.options.lists.len();
        let (descriptions, options) = self.options.lists.iter().fold((0, 0), |acc, l| {
            (
                acc.0 + i32::from(l.description.is_some()),
                acc.1 + l.options.len(),
            )
        });

        let vertical_padding = self.padding.vertical();
        let text_line_height = f32::from(text_line_height);

        let size = {
            #[allow(clippy::cast_precision_loss)]
            let intrinsic = Size::new(0.0, {
                let text = vertical_padding + text_line_height;
                let separators = ((vertical_padding / 2.0) + 1.0) * (lists - 1) as f32;
                let descriptions = (text + 4.0) * descriptions as f32;
                let options = text * options as f32;
                separators + descriptions + options
            });

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
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    if let Some(item) = self.hovered_option.as_ref() {
                        shell.publish((self.on_selected)(item.clone()));
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer).0);

                    let text_line_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)));

                    let heights = self
                        .options
                        .element_heights(self.padding.vertical(), text_line_height);

                    let mut current_offset = 0.0;

                    let previous_hover_option = self.hovered_option.take();

                    for (element, elem_height) in self.options.elements().zip(heights) {
                        let bounds = Rectangle {
                            x: 0.0,
                            y: 0.0 + current_offset,
                            width: bounds.width,
                            height: elem_height,
                        };

                        if bounds.contains(cursor_position) {
                            *self.hovered_option = if let OptionElement::Option((_, item)) = element
                            {
                                if previous_hover_option.as_ref() == Some(item) {
                                    previous_hover_option
                                } else {
                                    if let Some(on_option_hovered) = self.on_option_hovered {
                                        shell.publish(on_option_hovered(item.clone()));
                                    }

                                    Some(item.clone())
                                }
                            } else {
                                None
                            };

                            break;
                        }
                        current_offset += elem_height;
                    }
                }
            }
            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let text_size = self
                        .text_size
                        .unwrap_or_else(|| text::Renderer::default_size(renderer).0);

                    let text_line_height =
                        f32::from(self.text_line_height.to_absolute(Pixels(text_size)));

                    let heights = self
                        .options
                        .element_heights(self.padding.vertical(), text_line_height);

                    let mut current_offset = 0.0;

                    let previous_hover_option = self.hovered_option.take();

                    for (element, elem_height) in self.options.elements().zip(heights) {
                        let bounds = Rectangle {
                            x: 0.0,
                            y: 0.0 + current_offset,
                            width: bounds.width,
                            height: elem_height,
                        };

                        if bounds.contains(cursor_position) {
                            *self.hovered_option = if let OptionElement::Option((_, item)) = element
                            {
                                if previous_hover_option.as_ref() == Some(item) {
                                    previous_hover_option
                                } else {
                                    Some(item.clone())
                                }
                            } else {
                                None
                            };

                            if let Some(item) = self.hovered_option {
                                shell.publish((self.on_selected)(item.clone()));
                            }

                            break;
                        }
                        current_offset += elem_height;
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

    #[allow(clippy::too_many_lines)]
    fn draw(
        &self,
        _state: &Tree,
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

        let offset = viewport.y - bounds.y;

        let text_line_height = f32::from(self.text_line_height.to_absolute(Pixels(text_size)));

        let visible_options = self.options.visible_options(
            self.padding.vertical(),
            text_line_height,
            offset,
            viewport.height,
        );

        let mut current_offset = 0.0;

        for (elem, elem_height) in visible_options {
            let mut bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + current_offset,
                width: bounds.width,
                height: elem_height,
            };

            current_offset += elem_height;

            match elem {
                OptionElement::Option((option, item)) => {
                    let (color, font) = if self.selected_option.as_ref() == Some(&item) {
                        let item_x = bounds.x + appearance.border_width;
                        let item_width = appearance.border_width.mul_add(-2.0, bounds.width);

                        bounds = Rectangle {
                            x: item_x,
                            width: item_width,
                            ..bounds
                        };

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds,
                                border: Border {
                                    radius: appearance.border_radius,
                                    ..Default::default()
                                },
                                shadow: Shadow::default(),
                            },
                            appearance.selected_background,
                        );

                        let svg_handle =
                            svg::Svg::new(crate::widget::common::object_select().clone())
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
                    } else if self.hovered_option.as_ref() == Some(item) {
                        let item_x = bounds.x + appearance.border_width;
                        let item_width = appearance.border_width.mul_add(-2.0, bounds.width);

                        bounds = Rectangle {
                            x: item_x,
                            width: item_width,
                            ..bounds
                        };

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds,
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

                    let bounds = Rectangle {
                        x: bounds.x + self.padding.left,
                        // TODO: Figure out why it's offset by 8 pixels
                        y: bounds.y + self.padding.top + 8.0,
                        width: bounds.width,
                        height: elem_height,
                    };
                    text::Renderer::fill_text(
                        renderer,
                        Text {
                            content: option.as_ref().to_string(),
                            bounds: bounds.size(),
                            size: iced::Pixels(text_size),
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

                OptionElement::Separator => {
                    let divider = crate::widget::divider::horizontal::light().height(1.0);

                    let layout_node = layout::Node::new(Size {
                        width: bounds.width,
                        height: 1.0,
                    })
                    .move_to(Point {
                        x: bounds.x,
                        y: bounds.y + (self.padding.vertical() / 2.0) - 4.0,
                    });

                    Widget::<Message, crate::Theme, crate::Renderer>::draw(
                        crate::Element::<Message>::from(divider).as_widget(),
                        &Tree::empty(),
                        renderer,
                        theme,
                        style,
                        Layout::new(&layout_node),
                        cursor,
                        viewport,
                    );
                }

                OptionElement::Description(description) => {
                    let bounds = Rectangle {
                        x: bounds.center_x(),
                        y: bounds.center_y(),
                        ..bounds
                    };
                    text::Renderer::fill_text(
                        renderer,
                        Text {
                            content: description.as_ref().to_string(),
                            bounds: bounds.size(),
                            size: iced::Pixels(text_size),
                            line_height: text::LineHeight::Absolute(Pixels(text_line_height + 4.0)),
                            font: crate::font::default(),
                            horizontal_alignment: alignment::Horizontal::Center,
                            vertical_alignment: alignment::Vertical::Center,
                            shaping: text::Shaping::Advanced,
                            wrapping: text::Wrapping::default(),
                        },
                        bounds.position(),
                        appearance.description_color,
                        *viewport,
                    );
                }
            }
        }
    }
}

impl<'a, S, Item, Message: 'a> From<InnerList<'a, S, Item, Message>>
    for Element<'a, Message, crate::Theme, crate::Renderer>
where
    S: AsRef<str>,
    Item: Clone + PartialEq,
{
    fn from(list: InnerList<'a, S, Item, Message>) -> Self {
        Element::new(list)
    }
}

pub(super) enum OptionElement<'a, S, Item> {
    Description(&'a S),
    Option(&'a (S, Item)),
    Separator,
}

impl<S, Message> Model<S, Message> {
    pub(super) fn elements(&self) -> impl Iterator<Item = OptionElement<'_, S, Message>> + '_ {
        self.lists.iter().flat_map(|list| {
            let description = list
                .description
                .as_ref()
                .into_iter()
                .map(OptionElement::Description);

            let options = list.options.iter().map(OptionElement::Option);

            description
                .chain(options)
                .chain(std::iter::once(OptionElement::Separator))
        })
    }

    fn element_heights(
        &self,
        vertical_padding: f32,
        text_line_height: f32,
    ) -> impl Iterator<Item = f32> + '_ {
        self.elements().map(move |element| match element {
            OptionElement::Option(_) => vertical_padding + text_line_height,
            OptionElement::Separator => (vertical_padding / 2.0) + 1.0,
            OptionElement::Description(_) => vertical_padding + text_line_height + 4.0,
        })
    }

    fn visible_options(
        &self,
        padding_vertical: f32,
        text_line_height: f32,
        offset: f32,
        height: f32,
    ) -> impl Iterator<Item = (OptionElement<'_, S, Message>, f32)> + '_ {
        let heights = self.element_heights(padding_vertical, text_line_height);

        let mut current = 0.0;
        self.elements()
            .zip(heights)
            .filter(move |(_, element_height)| {
                let end = current + element_height;
                let visible = current >= offset && end <= offset + height;
                current = end;
                visible
            })
    }
}
