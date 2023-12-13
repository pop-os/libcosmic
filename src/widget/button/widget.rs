// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].

use iced_runtime::core::widget::Id;
use iced_runtime::{keyboard, Command};

use iced_core::event::{self, Event};
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer::{self, Quad};
use iced_core::touch;
use iced_core::widget::tree::{self, Tree};
use iced_core::widget::Operation;
use iced_core::{layout, svg};
use iced_core::{
    Background, Clipboard, Color, Element, Layout, Length, Padding, Point, Rectangle, Shell,
    Vector, Widget,
};
use iced_renderer::core::widget::{operation, OperationOutputWrapper};

use crate::theme::THEME;

pub use super::style::{Appearance, StyleSheet};

/// Internally defines different button widget variants.
enum Variant<Message> {
    Normal,
    Image {
        close_icon: svg::Handle,
        selection_icon: svg::Handle,
        on_remove: Option<Message>,
    },
}

/// A generic button which emits a message when pressed.
#[allow(missing_debug_implementations)]
#[must_use]
pub struct Button<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Id,
    #[cfg(feature = "a11y")]
    name: Option<Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    label: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    selected: bool,
    style: <Renderer::Theme as StyleSheet>::Style,
    variant: Variant<Message>,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            id: Id::unique(),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            #[cfg(feature = "a11y")]
            label: None,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            selected: false,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            variant: Variant::Normal,
        }
    }

    pub fn new_image(
        content: impl Into<Element<'a, Message, Renderer>>,
        on_remove: Option<Message>,
    ) -> Self {
        Self {
            id: Id::unique(),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            #[cfg(feature = "a11y")]
            label: None,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            selected: false,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            variant: Variant::Image {
                on_remove,
                close_icon: crate::widget::icon::from_name("window-close-symbolic")
                    .size(8)
                    .icon()
                    .into_svg_handle()
                    .unwrap_or_else(|| {
                        let bytes: &'static [u8] = &[];
                        iced_core::svg::Handle::from_memory(bytes)
                    }),
                selection_icon: crate::widget::icon::from_name("object-select-symbolic")
                    .size(16)
                    .icon()
                    .into_svg_handle()
                    .unwrap_or_else(|| {
                        let bytes: &'static [u8] = &[];
                        iced_core::svg::Handle::from_memory(bytes)
                    }),
            },
        }
    }

    /// Sets the [`Id`] of the [`Button`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    /// Sets the widget to a selected state.
    ///
    /// Displays a selection indicator on image buttons.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;

        self
    }

    /// Sets the style variant of this [`Button`].
    pub fn style(mut self, style: <Renderer::Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Button`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description_widget<T: iced_accessibility::Describes>(mut self, description: &T) -> Self {
        self.description = Some(iced_accessibility::Description::Id(
            description.description(),
        ));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(iced_accessibility::Description::Text(description.into()));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the label of the [`Button`].
    pub fn label(mut self, label: &dyn iced_accessibility::Labels) -> Self {
        self.label = Some(label.label().into_iter().map(|l| l.into()).collect());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Button<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer + svg::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            |renderer, limits| {
                self.content
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
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(state, Some(&self.id));
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
        if let Variant::Image {
            on_remove: Some(on_remove),
            ..
        } = &self.variant
        {
            // Capture mouse/touch events on the removal button
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(position) = cursor.position() {
                        if removal_bounds(layout.bounds(), 4.0).contains(position) {
                            shell.publish(on_remove.clone());
                            return event::Status::Captured;
                        }
                    }
                }

                _ => (),
            }
        }

        if self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) == event::Status::Captured
        {
            return event::Status::Captured;
        }

        update(
            self.id.clone(),
            event,
            layout,
            cursor,
            shell,
            &self.on_press,
            || tree.state.downcast_mut::<State>(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let styling = draw(
            renderer,
            bounds,
            cursor,
            self.on_press.is_some(),
            self.selected,
            theme,
            &self.style,
            || tree.state.downcast_ref::<State>(),
            |renderer, styling| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    &renderer::Style {
                        icon_color: styling.icon_color.unwrap_or(renderer_style.icon_color),
                        text_color: styling.text_color.unwrap_or(renderer_style.icon_color),
                        scale_factor: renderer_style.scale_factor,
                    },
                    content_layout,
                    cursor,
                    &bounds,
                );
            },
        );

        if let Variant::Image {
            close_icon,
            selection_icon,
            on_remove,
        } = &self.variant
        {
            let selection_background = theme.selection_background();

            let c_rad = THEME.with(|t| t.borrow().cosmic().corner_radii);

            if self.selected {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            width: 24.0,
                            height: 20.0,
                            x: bounds.x + styling.border_width,
                            y: bounds.y + (bounds.height - 20.0 - styling.border_width),
                        },
                        border_radius: [
                            c_rad.radius_0[0],
                            c_rad.radius_s[1],
                            c_rad.radius_0[2],
                            c_rad.radius_s[3],
                        ]
                        .into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    selection_background,
                );

                iced_core::svg::Renderer::draw(
                    renderer,
                    selection_icon.clone(),
                    styling.icon_color,
                    Rectangle {
                        width: 16.0,
                        height: 16.0,
                        x: bounds.x + 5.0 + styling.border_width,
                        y: bounds.y + (bounds.height - 18.0 - styling.border_width),
                    },
                );
            }

            if on_remove.is_some() {
                if let Some(position) = cursor.position() {
                    if bounds.contains(position) {
                        let bounds = removal_bounds(layout.bounds(), 4.0);
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds,
                                border_radius: c_rad.radius_m.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            selection_background,
                        );

                        iced_core::svg::Renderer::draw(
                            renderer,
                            close_icon.clone(),
                            styling.icon_color,
                            Rectangle {
                                width: 16.0,
                                height: 16.0,
                                x: bounds.x + 4.0,
                                y: bounds.y + 4.0,
                            },
                        );
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor, self.on_press.is_some())
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        use iced_accessibility::{
            accesskit::{Action, DefaultActionVerb, NodeBuilder, NodeId, Rect, Role},
            A11yNode, A11yTree,
        };

        let child_layout = layout.children().next().unwrap();
        let child_tree = &state.children[0];
        let child_tree = self
            .content
            .as_widget()
            .a11y_nodes(child_layout, &child_tree, p);

        let Rectangle {
            x,
            y,
            width,
            height,
        } = layout.bounds();
        let bounds = Rect::new(x as f64, y as f64, (x + width) as f64, (y + height) as f64);
        let is_hovered = state.state.downcast_ref::<State>().is_hovered;

        let mut node = NodeBuilder::new(Role::Button);
        node.add_action(Action::Focus);
        node.add_action(Action::Default);
        node.set_bounds(bounds);
        if let Some(name) = self.name.as_ref() {
            node.set_name(name.clone());
        }
        match self.description.as_ref() {
            Some(iced_accessibility::Description::Id(id)) => {
                node.set_described_by(
                    id.iter()
                        .cloned()
                        .map(|id| NodeId::from(id))
                        .collect::<Vec<_>>(),
                );
            }
            Some(iced_accessibility::Description::Text(text)) => {
                node.set_description(text.clone());
            }
            None => {}
        }

        if let Some(label) = self.label.as_ref() {
            node.set_labelled_by(label.clone());
        }

        if self.on_press.is_none() {
            node.set_disabled()
        }
        if is_hovered {
            node.set_hovered()
        }
        node.set_default_action_verb(DefaultActionVerb::Click);

        A11yTree::node_with_child_tree(A11yNode::new(node, self.id.clone()), child_tree)
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + svg::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(button: Button<'a, Message, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_field_names)]
pub struct State {
    is_hovered: bool,
    is_pressed: bool,
    is_focused: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the [`Button`] is currently focused or not.
    pub fn is_focused(self) -> bool {
        self.is_focused
    }

    /// Returns whether the [`Button`] is currently hovered or not.
    pub fn is_hovered(self) -> bool {
        self.is_hovered
    }

    /// Focuses the [`Button`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`Button`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`Button`]
/// accordingly.
#[allow(clippy::needless_pass_by_value)]
pub fn update<'a, Message: Clone>(
    _id: Id,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_press: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if on_press.is_some() {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    let state = state();

                    state.is_pressed = true;

                    return event::Status::Captured;
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if let Some(on_press) = on_press.clone() {
                let state = state();

                if state.is_pressed {
                    state.is_pressed = false;

                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        shell.publish(on_press);
                    }

                    return event::Status::Captured;
                }
            }
        }
        #[cfg(feature = "a11y")]
        Event::A11y(event_id, iced_accessibility::accesskit::ActionRequest { action, .. }) => {
            let state = state();
            if let Some(Some(on_press)) = (id == event_id
                && matches!(action, iced_accessibility::accesskit::Action::Default))
            .then(|| on_press.clone())
            {
                state.is_pressed = false;
                shell.publish(on_press);
            }
            return event::Status::Captured;
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
            if let Some(on_press) = on_press.clone() {
                let state = state();
                if state.is_focused && key_code == keyboard::KeyCode::Enter {
                    state.is_pressed = true;
                    shell.publish(on_press);
                    return event::Status::Captured;
                }
            }
        }
        Event::Touch(touch::Event::FingerLost { .. }) | Event::Mouse(mouse::Event::CursorLeft) => {
            let state = state();
            state.is_hovered = false;
            state.is_pressed = false;
        }
        _ => {}
    }

    event::Status::Ignored
}

#[allow(clippy::too_many_arguments)]
pub fn draw<'a, Renderer: iced_core::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    is_enabled: bool,
    is_selected: bool,
    style_sheet: &dyn StyleSheet<Style = <Renderer::Theme as StyleSheet>::Style>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    state: impl FnOnce() -> &'a State,
    draw_contents: impl FnOnce(&mut Renderer, Appearance),
) -> Appearance
where
    Renderer::Theme: StyleSheet,
{
    let is_mouse_over = cursor.position().is_some_and(|p| bounds.contains(p));

    let state: &State = state();

    let styling = if !is_enabled {
        style_sheet.disabled(style)
    } else if is_mouse_over {
        if state.is_pressed {
            style_sheet.pressed(state.is_focused || is_selected, style)
        } else {
            style_sheet.hovered(state.is_focused || is_selected, style)
        }
    } else {
        style_sheet.active(state.is_focused || is_selected, style)
    };

    let doubled_border_width = styling.border_width * 2.0;
    let doubled_outline_width = styling.outline_width * 2.0;

    if styling.outline_width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x - styling.border_width - styling.outline_width,
                    y: bounds.y - styling.border_width - styling.outline_width,
                    width: bounds.width + doubled_border_width + doubled_outline_width,
                    height: bounds.height + doubled_border_width + doubled_outline_width,
                },
                border_radius: styling.border_radius,
                border_width: styling.outline_width,
                border_color: styling.outline_color,
            },
            Color::TRANSPARENT,
        );
    }

    if styling.background.is_some() || styling.border_width > 0.0 {
        if styling.shadow_offset != Vector::default() {
            // TODO: Implement proper shadow support
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + styling.shadow_offset.x,
                        y: bounds.y + styling.shadow_offset.y,
                        width: bounds.width,
                        height: bounds.height,
                    },
                    border_radius: styling.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color([0.0, 0.0, 0.0, 0.5].into()),
            );
        }

        // Draw the button background first.
        if let Some(background) = styling.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: styling.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                background,
            );
        }

        // Then draw the button contents onto the background.
        draw_contents(renderer, styling);

        // Finish by drawing the border above the contents.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: styling.border_radius,
                border_width: styling.border_width,
                border_color: styling.border_color,
            },
            Color::TRANSPARENT,
        );
    } else {
        draw_contents(renderer, styling);
    }

    styling
}

/// Computes the layout of a [`Button`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    padding: Padding,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);

    let mut content = layout_content(renderer, &limits.pad(padding));
    let padding = padding.fit(content.size(), limits.max());
    let size = limits.pad(padding).resolve(content.size()).pad(padding);

    content.move_to(Point::new(padding.left, padding.top));

    layout::Node::with_children(size, vec![content])
}

/// Returns the [`mouse::Interaction`] of a [`Button`].
#[must_use]
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    is_enabled: bool,
) -> mouse::Interaction {
    let is_mouse_over = cursor.is_over(layout.bounds());

    if is_mouse_over && is_enabled {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

/// Produces a [`Command`] that focuses the [`Button`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id))
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        Self::is_focused(*self)
    }

    fn focus(&mut self) {
        Self::focus(self);
    }

    fn unfocus(&mut self) {
        Self::unfocus(self);
    }
}

fn removal_bounds(bounds: Rectangle, offset: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - 12.0 - offset,
        y: bounds.y - 12.0 + offset,
        width: 24.0,
        height: 24.0,
    }
}
