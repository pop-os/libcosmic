use crate::separator;
use derive_setters::Setters;
use iced::mouse::Interaction;
use iced::{overlay, Alignment, Length, Padding, Point, Rectangle};
use iced_native::event::Status;
use iced_native::layout::flex::{resolve, Axis};
use iced_native::layout::{Limits, Node};
use iced_native::overlay::from_children;
use iced_native::renderer::Style;
use iced_native::widget::{column, Operation, Tree};
use iced_native::{
    renderer, row, Background, Clipboard, Color, Element, Event, Layout, Shell, Widget,
};
use iced_style::container::{Appearance, StyleSheet};
use iced_style::theme;
use iced_style::theme::Container;

#[derive(Setters)]
#[allow(dead_code)]
pub struct ListBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet + iced_style::rule::StyleSheet,
    <Renderer as iced_native::Renderer>::Theme: iced_style::rule::StyleSheet,
{
    spacing: u16,
    #[setters(into)]
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    align_items: Alignment,
    style: <Renderer::Theme as StyleSheet>::Style,
    children: Vec<Element<'a, Message, Renderer>>,
    #[setters(strip_option)]
    placeholder: Option<Element<'a, Message, Renderer>>,
    show_separators: bool,
    on_item_selected: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

pub fn list_box<'a, Message: 'a, Renderer>() -> ListBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    <<Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style: From<Container>,
    <Renderer as iced_native::Renderer>::Theme: StyleSheet + iced_style::rule::StyleSheet,
    <<Renderer as iced_native::Renderer>::Theme as iced_style::rule::StyleSheet>::Style:
        From<theme::Rule>,
{
    ListBox::new()
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a> ListBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet + iced_style::rule::StyleSheet,
    <<Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style: From<Container>,
    <<Renderer as iced_native::Renderer>::Theme as iced_style::rule::StyleSheet>::Style:
        From<theme::Rule>,
{
    /// The default padding of a [`ListBox`] drawn by this renderer.
    pub const DEFAULT_PADDING: u16 = 0;

    /// Creates an empty [`ListBox`].
    pub fn new() -> Self {
        Self::with_children(Vec::<Element<Message, Renderer>>::new()).render()
    }

    /// Creates a new [`ListBox`].
    ///
    /// [`ListBox`]: struct.ListBox.html
    pub fn with_children(children: Vec<Element<'a, Message, Renderer>>) -> Self {
        let list_box = Self {
            spacing: 0,
            padding: Padding::from(Self::DEFAULT_PADDING),
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            align_items: Alignment::Center,
            style: Default::default(),
            children,
            placeholder: None,
            show_separators: true,
            on_item_selected: None,
        };
        list_box.render()
    }

    pub fn render(mut self) -> Self {
        let children_size = self.children.len();
        self.children = self
            .children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                let row_items = if self.show_separators && index != children_size - 1 {
                    vec![
                        row![child].align_items(self.align_items).into(),
                        separator!(1).into(),
                    ]
                } else {
                    vec![row![child].align_items(self.align_items).into()]
                };
                column(row_items).spacing(self.spacing).into()
            })
            .collect();
        self
    }

    /// Adds an element to the [`ListBox`].
    pub fn push(mut self, child: impl Into<Element<'a, Message, Renderer>>) -> Self {
        self.children.push(child.into());
        self = self.render();
        self
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a> std::default::Default
    for ListBox<'a, Message, Renderer>
where
    Renderer::Theme: StyleSheet + iced_style::rule::StyleSheet,
    <<Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style: From<Container>,
    <<Renderer as iced_native::Renderer>::Theme as iced_style::rule::StyleSheet>::Style:
        From<theme::Rule>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ListBox<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    <Renderer as iced_native::Renderer>::Theme: StyleSheet + iced_style::rule::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        let limits = limits
            .max_width(self.max_width)
            .width(self.width)
            .height(self.height);

        if !self.children.is_empty() {
            resolve(
                Axis::Vertical,
                renderer,
                &limits,
                self.padding,
                self.spacing as f32,
                self.align_items,
                &self.children,
            )
        } else if self.placeholder.is_some() {
            self.placeholder
                .as_ref()
                .unwrap()
                .as_widget()
                .layout(renderer, &limits)
        } else {
            Node::default()
        }
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let color_scheme = theme.appearance(self.style);

        draw_background(renderer, &color_scheme, layout.bounds());

        if !self.children.is_empty() {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&state.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                );
            }
        } else if let Some(placeholder) = &self.placeholder {
            placeholder.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }
    }

    fn children(&self) -> Vec<Tree> {
        let widgets = if !self.children.is_empty() {
            self.children.iter().map(Tree::new).collect()
        } else if let Some(placeholder) = &self.placeholder {
            vec![Tree::new(placeholder)]
        } else {
            vec![Tree::empty()]
        };
        widgets
    }

    fn diff(&self, tree: &mut Tree) {
        if !self.children.is_empty() {
            tree.diff_children(&self.children);
        } else if let Some(placeholder) = &self.placeholder {
            tree.diff_children(&[placeholder]);
        }
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        if !self.children.is_empty() {
            operation.container(None, &mut |operation| {
                self.children
                    .iter()
                    .zip(&mut state.children)
                    .zip(layout.children())
                    .for_each(|((child, state), layout)| {
                        child.as_widget().operate(state, layout, operation);
                    })
            });
        } else if let Some(placeholder) = &self.placeholder {
            placeholder.as_widget().operate(state, layout, operation);
        }
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> Status {
        if !self.children.is_empty() {
            self.children
                .iter_mut()
                .zip(&mut state.children)
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child.as_widget_mut().on_event(
                        state,
                        event.clone(),
                        layout,
                        cursor_position,
                        renderer,
                        clipboard,
                        shell,
                    )
                })
                .fold(Status::Ignored, Status::merge)
        } else if self.placeholder.is_some() {
            self.placeholder.as_mut().unwrap().as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        } else {
            Status::Ignored
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        if !self.children.is_empty() {
            self.children
                .iter()
                .zip(&state.children)
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child.as_widget().mouse_interaction(
                        state,
                        layout,
                        cursor_position,
                        viewport,
                        renderer,
                    )
                })
                .max()
                .unwrap_or_default()
        } else if let Some(placeholder) = &self.placeholder {
            placeholder.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        } else {
            Interaction::Idle
        }
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        if !self.children.is_empty() {
            from_children(&self.children, tree, layout, renderer)
        } else if let Some(placeholder) = &self.placeholder {
            placeholder
                .as_widget()
                .overlay(&mut tree.children[0], layout, renderer)
        } else {
            None
        }
    }
}

/// Draws the background of a [`Container`] given its [`Style`] and its `bounds`.
pub fn draw_background<Renderer>(
    renderer: &mut Renderer,
    appearance: &Appearance,
    bounds: Rectangle,
) where
    Renderer: iced_native::Renderer,
{
    if appearance.background.is_some() || appearance.border_width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: appearance.border_radius,
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            appearance
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}

impl<'a, Message: 'a, Renderer: iced_native::Renderer + 'a> From<ListBox<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: StyleSheet + iced_style::rule::StyleSheet,
{
    fn from(list_box: ListBox<'a, Message, Renderer>) -> Self {
        Self::new(list_box)
    }
}

#[macro_export]
macro_rules! list_box_item {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::row![
            column(vec![
                $($x),+
            ])
        ]
    );
}
pub use list_box_item;

#[macro_export]
macro_rules! list_box_heading {
    ($title:expr) => {
        $crate::iced::widget::container(
            $crate::iced::widget::row![
                text($title).size(18),
                $crate::iced::widget::vertical_space(Length::Fill),
                $crate::iced::widget::horizontal_space(Length::Fill)
            ]
            .height(Length::Fill)
            .align_items($crate::iced::alignment::Alignment::Center),
        )
        .style($crate::iced::theme::Container::Custom(
            $crate::widget::expander_heading_style,
        ))
        .max_height(60)
        .padding(10)
    };
    ($title:expr, $subtitle:expr) => {
        $crate::iced::widget::container(
            $crate::iced::widget::row![
                column(vec![
                    text($title).size(18).into(),
                    text($subtitle).size(16).into(),
                ]),
                $crate::iced::widget::vertical_space(Length::Fill),
                $crate::iced::widget::horizontal_space(Length::Fill)
            ]
            .height(Length::Fill)
            .align_items($crate::iced::alignment::Alignment::Center),
        )
        .style($crate::iced::theme::Container::Custom(
            $crate::widget::expander_heading_style,
        ))
        .max_height(60)
        .padding(10)
    };
    ($title:expr, $subtitle:expr, $icon:expr) => {
        $crate::iced::widget::container(
            $crate::iced::widget::row![
                container($crate::widget::icon($icon, 20)).padding(10),
                column(vec![
                    text($title).size(18).into(),
                    text($subtitle).size(16).into(),
                ]),
                $crate::iced::widget::vertical_space(Length::Fill),
                $crate::iced::widget::horizontal_space(Length::Fill)
            ]
            .height(Length::Fill)
            .align_items($crate::iced::alignment::Alignment::Center),
        )
        .style($crate::iced::theme::Container::Custom(
            $crate::widget::expander_heading_style,
        ))
        .max_height(60)
        .padding(10)
    };
}
pub use list_box_heading;
