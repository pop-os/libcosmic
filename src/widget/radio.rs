//! Create choices using radio buttons.
use crate::Theme;
use iced::border;
use iced_core::event::{self, Event};
use iced_core::layout;
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::touch;
use iced_core::widget::tree::Tree;
use iced_core::{
    Border, Clipboard, Element, Layout, Length, Pixels, Rectangle, Shell, Size, Vector, Widget,
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
    label: Element<'a, Message, Theme, Renderer>,
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

    /// The default spacing of a [`Radio`] button.
    pub const DEFAULT_SPACING: f32 = 8.0;

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
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING,
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
        vec![Tree::new(&self.label)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.children[0].diff(&mut self.label);
    }
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::next_to_each_other(
            &limits.width(self.width),
            self.spacing,
            |_| layout::Node::new(Size::new(self.size, self.size)),
            |limits| {
                self.label
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        self.label.as_widget().operate(
            &mut tree.children[0],
            layout.children().nth(1).unwrap(),
            renderer,
            operation,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let status = self.label.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().nth(1).unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if status == event::Status::Ignored {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        shell.publish(self.on_click.clone());

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }

            event::Status::Ignored
        } else {
            status
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
        let interaction = self.label.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().nth(1).unwrap(),
            cursor,
            viewport,
            renderer,
        );

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

        let mut children = layout.children();

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

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            let size = bounds.width;
            let dot_size = 6.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
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
                            x: bounds.x + (size - dot_size) / 2.0,
                            y: bounds.y + (size - dot_size) / 2.0,
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

        {
            let label_layout = children.next().unwrap();
            self.label.as_widget().draw(
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
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.label.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().nth(1).unwrap(),
            renderer,
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
        self.label.as_widget().drag_destinations(
            &state.children[0],
            layout.children().nth(1).unwrap(),
            renderer,
            dnd_rectangles,
        );
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
