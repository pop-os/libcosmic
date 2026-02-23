//! Show toggle controls using togglers.

use std::time::{Duration, Instant};

use crate::{Element, anim, iced_core::Border, iced_widget::toggler::Status};
use iced_core::{
    Clipboard, Event, Layout, Length, Pixels, Rectangle, Shell, Size, Widget, alignment, event,
    layout, mouse,
    renderer::{self, Renderer},
    text,
    widget::{self, Tree, tree},
    window,
};
use iced_widget::Id;

pub use crate::iced_widget::toggler::{Catalog, Style};

pub fn toggler<'a, Message>(is_checked: bool) -> Toggler<'a, Message> {
    Toggler::new(is_checked)
}
/// A toggler widget.
#[allow(missing_debug_implementations)]
pub struct Toggler<'a, Message> {
    id: Id,
    is_toggled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    label: Option<String>,
    width: Length,
    size: f32,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_alignment: text::Alignment,
    text_shaping: text::Shaping,
    spacing: f32,
    font: Option<crate::font::Font>,
    duration: Duration,
    ellipsize: text::Ellipsize,
}

impl<'a, Message> Toggler<'a, Message> {
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: f32 = 24.0;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new(is_toggled: bool) -> Self {
        Toggler {
            id: Id::unique(),
            is_toggled,
            on_toggle: None,
            label: None,
            width: Length::Fill,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_alignment: text::Alignment::Left,
            text_shaping: text::Shaping::Advanced,
            spacing: 0.0,
            font: None,
            duration: Duration::from_millis(200),
            ellipsize: text::Ellipsize::None,
        }
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Toggler`].
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: text::Alignment) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Toggler`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the [`text::Ellipsize`] strategy of the [`Toggler`].
    pub fn ellipsize(mut self, ellipsize: text::Ellipsize) -> Self {
        self.ellipsize = ellipsize;
        self
    }

    /// Sets the [`Font`] of the text of the [`Toggler`]
    ///
    /// [`Font`]: cosmic::iced::text::Renderer::Font
    pub fn font(mut self, font: impl Into<crate::font::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn duration(mut self, dur: Duration) -> Self {
        self.duration = dur;
        self
    }

    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|t| Box::new(t) as _);
        self
    }

    /// Sets the label of the [`Button`].
    pub fn label(mut self, label: impl Into<Option<String>>) -> Self {
        self.label = label.into();
        self
    }
}

impl<'a, Message> Widget<Message, crate::Theme, crate::Renderer> for Toggler<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);

        let res = next_to_each_other(
            &limits,
            self.spacing,
            |limits| {
                if let Some(label) = self.label.as_deref() {
                    let state = tree.state.downcast_mut::<State>();
                    let node = iced_core::widget::text::layout(
                        &mut state.text,
                        renderer,
                        limits,
                        label,
                        widget::text::Format {
                            width: self.width,
                            height: Length::Shrink,
                            line_height: self.text_line_height,
                            size: self.text_size.map(iced::Pixels),
                            font: self.font,
                            align_x: self.text_alignment,
                            align_y: alignment::Vertical::Top,
                            shaping: self.text_shaping,
                            wrapping: crate::iced_core::text::Wrapping::default(),
                            ellipsize: self.ellipsize,
                        },
                    );
                    match self.width {
                        Length::Fill => {
                            let size = node.size();
                            layout::Node::with_children(
                                Size::new(limits.width(Length::Fill).max().width, size.height),
                                vec![node],
                            )
                        }
                        _ => node,
                    }
                } else {
                    layout::Node::new(iced_core::Size::ZERO)
                }
            },
            |_| layout::Node::new(Size::new(48., 24.)),
        );
        res
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _renderer: &crate::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(on_toggle) = self.on_toggle.as_ref() else {
            return;
        };
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let mouse_over = cursor_position.is_over(layout.bounds());

                if mouse_over {
                    shell.publish((on_toggle)(!self.is_toggled));
                    state.anim.changed(self.duration);
                    shell.capture_event();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                state.anim.anim_done(self.duration);
                if state.anim.last_change.is_some() {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        if cursor_position.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let mut children = layout.children();
        let label_layout = children.next().unwrap();

        if let Some(_label) = &self.label {
            let state: &State = tree.state.downcast_ref();
            iced_widget::text::draw(
                renderer,
                style,
                label_layout.bounds(),
                state.text.raw(),
                iced_widget::text::Style::default(),
                viewport,
            );
        }

        let toggler_layout = children.next().unwrap();
        let bounds = toggler_layout.bounds();

        let is_mouse_over = cursor_position.is_over(bounds);

        // let style = blend_appearances(
        //     theme.style(
        //         &(),
        //         if is_mouse_over {
        //             Status::Hovered { is_toggled: false }
        //         } else {
        //             Status::Active { is_toggled: false }
        //         },
        //     ),
        //     theme.style(
        //         &(),
        //         if is_mouse_over {
        //             Status::Hovered { is_toggled: true }
        //         } else {
        //             Status::Active { is_toggled: true }
        //         },
        //     ),
        //     percent,
        // );

        let style = theme.style(
            &(),
            if is_mouse_over {
                Status::Hovered {
                    is_toggled: self.is_toggled,
                }
            } else {
                Status::Active {
                    is_toggled: self.is_toggled,
                }
            },
        );

        let space = style.handle_margin;

        let toggler_background_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_background_bounds,
                border: Border {
                    radius: style.border_radius,
                    ..Default::default()
                },
                ..renderer::Quad::default()
            },
            style.background,
        );
        let mut t = state.anim.t(self.duration, self.is_toggled);

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + anim::slerp(
                    space,
                    bounds.width - space - (bounds.height - (2.0 * space)),
                    t,
                ),

            y: bounds.y + space,
            width: bounds.height - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border: Border {
                    radius: style.handle_radius,
                    ..Default::default()
                },
                ..renderer::Quad::default()
            },
            style.foreground,
        );
    }
}

impl<'a, Message: 'static> From<Toggler<'a, Message>> for Element<'a, Message> {
    fn from(toggler: Toggler<'a, Message>) -> Element<'a, Message> {
        Element::new(toggler)
    }
}

/// Produces a [`Node`] with two children nodes one right next to each other.
pub fn next_to_each_other(
    limits: &iced::Limits,
    spacing: f32,
    left: impl FnOnce(&iced::Limits) -> iced_core::layout::Node,
    right: impl FnOnce(&iced::Limits) -> iced_core::layout::Node,
) -> iced_core::layout::Node {
    let mut right_node = right(limits);
    let right_size = right_node.size();

    let left_limits = limits.shrink(Size::new(right_size.width + spacing, 0.0));
    let mut left_node = left(&left_limits);
    let left_size = left_node.size();

    let (left_y, right_y) = if left_size.height > right_size.height {
        (0.0, (left_size.height - right_size.height) / 2.0)
    } else {
        ((right_size.height - left_size.height) / 2.0, 0.0)
    };

    left_node = left_node.move_to(iced::Point::new(0.0, left_y));
    right_node = right_node.move_to(iced::Point::new(left_size.width + spacing, right_y));

    iced_core::layout::Node::with_children(
        Size::new(
            left_size.width + spacing + right_size.width,
            left_size.height.max(right_size.height),
        ),
        vec![left_node, right_node],
    )
}

#[derive(Debug, Default)]
pub struct State {
    text: widget::text::State<<crate::Renderer as iced_core::text::Renderer>::Paragraph>,
    anim: anim::State,
}
