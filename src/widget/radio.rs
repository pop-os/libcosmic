//! Create choices using radio buttons.
use crate::{Theme, theme};
use iced::border;
use iced_core::event::{self, Event};
use iced_core::widget::tree::Tree;
use iced_core::{
    Border, Clipboard, Element, Layout, Length, Pixels, Rectangle, Shell, Size, Vector, Widget,
    layout, mouse, overlay, renderer, touch,
};

use iced_widget::radio as iced_radio;
pub use iced_widget::radio::Catalog;

pub fn radio<'a, Message: Clone, V, F>(
    label: impl Into<Element<'a, Message, Theme, crate::Renderer>>,
    value: V,
    selected: Option<V>,
    f: F,
) -> Radio<'a, Message, crate::Renderer>
where
    V: Eq + Copy,
    F: FnOnce(V) -> Message,
{
    Radio::new(label, value, selected, f)
}

/// A circular button representing a choice.
///
/// # Example
/// ```no_run
/// # type Radio<'a, Message> =
/// #     cosmic::widget::Radio<'a, Message, cosmic::Renderer>;
/// #
/// # use cosmic::widget::text;
/// # use cosmic::iced::widget::column;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum Choice {
///     A,
///     B,
///     C,
///     All,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Message {
///     RadioSelected(Choice),
/// }
///
/// let selected_choice = Some(Choice::A);
///
/// let a = Radio::new(
///     text::heading("A"),
///     Choice::A,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let b = Radio::new(
///     text::heading("B"),
///     Choice::B,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let c = Radio::new(
///     text::heading("C"),
///     Choice::C,
///     selected_choice,
///     Message::RadioSelected,
/// );
///
/// let all = Radio::new(
///     column![
///         text::heading("All"),
///         text::body("A, B and C"),
///     ],
///     Choice::All,
///     selected_choice,
///     Message::RadioSelected
/// );
///
/// let content = column![a, b, c, all];
/// ```
#[allow(missing_debug_implementations)]
pub struct Radio<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced_core::Renderer,
{
    is_selected: bool,
    on_click: Message,
    label: Option<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    size: f32,
    spacing: f32,
}

impl<'a, Message, Renderer> Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    /// The default size of a [`Radio`] button.
    pub const DEFAULT_SIZE: f32 = 16.0;

    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///     receives the value of the radio and must produce a `Message`.
    pub fn new<T, F, V>(label: T, value: V, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: Some(label.into()),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: theme::spacing().space_xs as f32,
        }
    }

    /// Creates a new [`Radio`] button without a label.
    ///
    /// This is intended for internal use with the settings item builder,
    /// where the label comes from the settings item title instead.
    pub(crate) fn new_no_label<V, F>(value: V, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: None,
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: theme::spacing().space_xs as f32,
        }
    }

    #[must_use]
    /// Sets the size of the [`Radio`] button.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    #[must_use]
    /// Sets the width of the [`Radio`] button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    /// Sets the spacing between the [`Radio`] button and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Radio<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        if let Some(label) = &self.label {
            vec![Tree::new(label)]
        } else {
            vec![]
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        if let Some(label) = &mut self.label {
            tree.diff_children(std::slice::from_mut(label));
        }
    }
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(label) = &mut self.label {
            layout::next_to_each_other(
                &limits.width(self.width),
                self.spacing,
                |_| layout::Node::new(Size::new(self.size, self.size)),
                |limits| {
                    label
                        .as_widget_mut()
                        .layout(&mut tree.children[0], renderer, limits)
                },
            )
        } else {
            layout::Node::new(Size::new(self.size, self.size))
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        if let Some(label) = &mut self.label {
            label.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().nth(1).unwrap(),
                renderer,
                operation,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(label) = &mut self.label {
            label.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().nth(1).unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if !shell.is_event_captured() {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        shell.publish(self.on_click.clone());
                        shell.capture_event();
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = if let Some(label) = &self.label {
            label.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().nth(1).unwrap(),
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::default()
        };

        if interaction == mouse::Interaction::default() {
            if cursor.is_over(layout.bounds()) {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        } else {
            interaction
        }
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
        let is_mouse_over = cursor.is_over(layout.bounds());

        let custom_style = if is_mouse_over {
            theme.style(
                &(),
                iced_radio::Status::Hovered {
                    is_selected: self.is_selected,
                },
            )
        } else {
            theme.style(
                &(),
                iced_radio::Status::Active {
                    is_selected: self.is_selected,
                },
            )
        };

        let (dot_bounds, label_layout) = if self.label.is_some() {
            let mut children = layout.children();
            let dot_bounds = children.next().unwrap().bounds();
            (dot_bounds, children.next())
        } else {
            (layout.bounds(), None)
        };

        {
            let size = dot_bounds.width;
            let dot_size = 6.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: dot_bounds,
                    border: Border {
                        radius: (size / 2.0).into(),
                        width: custom_style.border_width,
                        color: custom_style.border_color,
                    },
                    ..renderer::Quad::default()
                },
                custom_style.background,
            );

            if self.is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: dot_bounds.x + (size - dot_size) / 2.0,
                            y: dot_bounds.y + (size - dot_size) / 2.0,
                            width: dot_size,
                            height: dot_size,
                        },
                        border: border::rounded(dot_size / 2.0),
                        ..renderer::Quad::default()
                    },
                    custom_style.dot_color,
                );
            }
        }

        if let (Some(label), Some(label_layout)) = (&self.label, label_layout) {
            label.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                label_layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.label.as_mut()?.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().nth(1).unwrap(),
            renderer,
            viewport,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        if let Some(label) = &self.label {
            label.as_widget().drag_destinations(
                &state.children[0],
                layout.children().nth(1).unwrap(),
                renderer,
                dnd_rectangles,
            );
        }
    }
}

impl<'a, Message, Renderer> From<Radio<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer,
{
    fn from(radio: Radio<'a, Message, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(radio)
    }
}
