use iced::{
    Element, 
    Padding, 
    Length, 
    Alignment, 
    widget::{
        Container,
        Column, 
        scrollable
    },
     alignment
};
use iced_native::{
    Widget, 
    widget::{
        Tree, 
        container::{
            layout, 
            draw_background
        }
    }, 
    row,
    renderer
};
use iced_style::container::StyleSheet;

pub struct NavBar<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    spacing: u16,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    align_items: Alignment,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    style: <Renderer::Theme as StyleSheet>::Style,
    condensed: bool,
    active: bool,
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message: 'a, Renderer> NavBar<'a, Message, Renderer>
where 
    Renderer: iced_native::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    /// Creates a [`NavBar`] with the given elements.
    pub fn with_children(
        children: Vec<Element<'a, Message, Renderer>>,
    ) -> Self where <Renderer as iced_native::Renderer>::Theme: iced_style::scrollable::StyleSheet {
        let nav = Self::default();
        NavBar {
            content: Container::new(
                scrollable(
                    row![
                        Column::with_children(children)
                            .spacing(nav.spacing)
                            .padding(nav.padding)
                    ]
                )
                .scrollbar_width(6)
                .scroller_width(6)
            ).into(),
            ..Default::default()
        }
    }

    pub fn condensed(mut self, condensed: bool) -> Self {
        self.condensed = condensed;
        self
    }
    
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the [`Padding`] of the [`NavBar`].
    pub fn padding<P: Into<iced::Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`NavBar`].
    pub fn width(mut self, width: iced::Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`NavBar`].
    pub fn height(mut self, height: iced::Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the vertical alignment of the contents of the [`NavBar`] .
    pub fn align_items(mut self, align: iced::Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Sets the maximum width of the [`NavBar`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`NavBar`].
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the style of the [`NavBar`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message: 'a, Renderer> Default for NavBar<'a, Message, Renderer>
where 
    Renderer: iced_native::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn default() -> Self {
        Self { 
            spacing: 12,
            padding: Padding::new(12),
            width: Length::Shrink,
            height: Length::Fill,
            max_width: 300,
            max_height: u32::MAX,
            align_items: Alignment::Start,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style: Default::default(),
            condensed: false, 
            active: true, 
            content: Container::new(row![Column::new()]).into(),
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for NavBar<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<iced_native::widget::Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            if self.condensed {
                100
            } else {
                self.max_width
            },
            self.max_height,
            if self.active {
                self.padding
            } else {
                Padding::ZERO
            },
            self.horizontal_alignment,
            self.vertical_alignment,
            |renderer, limits| {
                if self.active {
                    self.content.as_widget().layout(renderer, limits)
                } else {
                    let content: Element<Message, Renderer> = Container::new(row![Column::new()]).into();
                    content.as_widget().layout(renderer, limits)
                }
            },
        )
    }
    
    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_native::Layout<'_>,
        operation: &mut dyn iced_native::widget::Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut iced_native::widget::Tree,
        event: iced::Event,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> iced::event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &iced_native::widget::Tree,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced_native::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        if self.active {
            let style = theme.appearance(self.style);

            draw_background(renderer, &style, layout.bounds());

            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &renderer::Style {
                    text_color: style
                        .text_color
                        .unwrap_or(renderer_style.text_color),
                },
                layout.children().next().unwrap(),
                cursor_position,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut iced_native::widget::Tree,
        layout: iced_native::Layout<'_>,
        renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
    
}

impl<'a, Message, Renderer> From<NavBar<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(navbar: NavBar<'a, Message, Renderer>) -> Self {
        Self::new(navbar)
    }
}

/// Creates a [NavBar`] with the given children.
///
/// [`NavBar`]: widget::NavBar
#[macro_export]
macro_rules! navbar {
    ($($x:expr),+ $(,)?) => (
        $crate::widget::NavBar::with_children(vec![$($crate::iced::Element::from($x)),+])
    );
}