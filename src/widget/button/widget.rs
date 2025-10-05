// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].

use iced_runtime::core::widget::Id;
use iced_runtime::{Action, Task, keyboard, task};

use iced_core::event::{self, Event};
use iced_core::renderer::{self, Quad, Renderer};
use iced_core::touch;
use iced_core::widget::Operation;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Background, Clipboard, Color, Layout, Length, Padding, Point, Rectangle, Shell, Vector, Widget,
};
use iced_core::{Border, mouse};
use iced_core::{Shadow, overlay};
use iced_core::{layout, svg};
use iced_renderer::core::widget::operation;

use crate::theme::THEME;

pub use super::style::{Catalog, Style};

/// Internally defines different button widget variants.
enum Variant<Message> {
    Normal,
    Image {
        close_icon: svg::Handle,
        on_remove: Option<Message>,
    },
}

/// A generic button which emits a message when pressed.
#[allow(missing_debug_implementations)]
#[must_use]
pub struct Button<'a, Message> {
    id: Id,
    #[cfg(feature = "a11y")]
    name: Option<std::borrow::Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    label: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    content: crate::Element<'a, Message>,
    on_press: Option<Box<dyn Fn(Vector, Rectangle) -> Message + 'a>>,
    on_press_down: Option<Box<dyn Fn(Vector, Rectangle) -> Message + 'a>>,
    width: Length,
    height: Length,
    padding: Padding,
    selected: bool,
    style: crate::theme::Button,
    variant: Variant<Message>,
    force_enabled: bool,
}

impl<'a, Message: Clone + 'a> Button<'a, Message> {
    /// Creates a new [`Button`] with the given content.
    pub(super) fn new(content: impl Into<crate::Element<'a, Message>>) -> Self {
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
            on_press_down: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            selected: false,
            style: crate::theme::Button::default(),
            variant: Variant::Normal,
            force_enabled: false,
        }
    }

    pub fn new_image(
        content: impl Into<crate::Element<'a, Message>>,
        on_remove: Option<Message>,
    ) -> Self {
        Self {
            id: Id::unique(),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            force_enabled: false,
            #[cfg(feature = "a11y")]
            label: None,
            content: content.into(),
            on_press: None,
            on_press_down: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            selected: false,
            style: crate::theme::Button::default(),
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
            },
        }
    }

    /// Sets the [`Id`] of the [`Button`].
    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the width of the [`Button`].
    #[inline]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    #[inline]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    #[inline]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed and released.
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(Box::new(move |_, _| on_press.clone()));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed and released.
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_with_rectangle(
        mut self,
        on_press: impl Fn(Vector, Rectangle) -> Message + 'a,
    ) -> Self {
        self.on_press = Some(Box::new(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_down(mut self, on_press: Message) -> Self {
        self.on_press_down = Some(Box::new(move |_, _| on_press.clone()));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_down_with_rectange(
        mut self,
        on_press: impl Fn(Vector, Rectangle) -> Message + 'a,
    ) -> Self {
        self.on_press_down = Some(Box::new(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        if let Some(m) = on_press {
            self.on_press(m)
        } else {
            self.on_press = None;
            self
        }
    }

    /// Sets the message that will be produced when the [`Button`] is pressed and released.
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_maybe_with_rectangle(
        mut self,
        on_press: impl Fn(Vector, Rectangle) -> Message + 'a,
    ) -> Self {
        self.on_press = Some(Box::new(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_down_maybe(mut self, on_press: Option<Message>) -> Self {
        if let Some(m) = on_press {
            self.on_press(m)
        } else {
            self.on_press_down = None;
            self
        }
    }

    /// Sets the message that will be produced when the [`Button`] is pressed and released.
    ///
    /// Unless `on_press` or `on_press_down` is called, the [`Button`] will be disabled.
    #[inline]
    pub fn on_press_down_maybe_with_rectangle(
        mut self,
        on_press: impl Fn(Vector, Rectangle) -> Message + 'a,
    ) -> Self {
        self.on_press_down = Some(Box::new(on_press));
        self
    }

    /// Sets the the [`Button`] to enabled whether or not it has handlers for on press.
    #[inline]
    pub fn force_enabled(mut self, enabled: bool) -> Self {
        self.force_enabled = enabled;
        self
    }

    /// Sets the widget to a selected state.
    ///
    /// Displays a selection indicator on image buttons.
    #[inline]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;

        self
    }

    /// Sets the style variant of this [`Button`].
    #[inline]
    pub fn class(mut self, style: crate::theme::Button) -> Self {
        self.style = style;
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Button`].
    pub fn name(mut self, name: impl Into<std::borrow::Cow<'a, str>>) -> Self {
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
    pub fn description(mut self, description: impl Into<std::borrow::Cow<'a, str>>) -> Self {
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

impl<'a, Message: 'a + Clone> Widget<Message, crate::Theme, crate::Renderer>
    for Button<'a, Message>
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

    fn size(&self) -> iced_core::Size<Length> {
        iced_core::Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
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
        renderer: &crate::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout
                    .children()
                    .next()
                    .unwrap()
                    .with_virtual_offset(layout.virtual_offset()),
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
        renderer: &crate::Renderer,
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
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
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
            self.on_press.as_deref(),
            self.on_press_down.as_deref(),
            || tree.state.downcast_mut::<State>(),
        )
    }

    #[allow(clippy::too_many_lines)]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if !viewport.intersects(&bounds) {
            return;
        }

        // FIXME: Why is there no content layout
        let Some(content_layout) = layout.children().next() else {
            return;
        };

        let mut headerbar_alpha = None;

        let is_enabled =
            self.on_press.is_some() || self.on_press_down.is_some() || self.force_enabled;
        let is_mouse_over = cursor.position().is_some_and(|p| bounds.contains(p));

        let state = tree.state.downcast_ref::<State>();

        let styling = if !is_enabled {
            theme.disabled(&self.style)
        } else if is_mouse_over {
            if state.is_pressed {
                if !self.selected && matches!(self.style, crate::theme::Button::HeaderBar) {
                    headerbar_alpha = Some(0.8);
                }

                theme.pressed(state.is_focused, self.selected, &self.style)
            } else {
                if !self.selected && matches!(self.style, crate::theme::Button::HeaderBar) {
                    headerbar_alpha = Some(0.8);
                }
                theme.hovered(state.is_focused, self.selected, &self.style)
            }
        } else {
            if !self.selected && matches!(self.style, crate::theme::Button::HeaderBar) {
                headerbar_alpha = Some(0.75);
            }

            theme.active(state.is_focused, self.selected, &self.style)
        };

        let mut icon_color = styling.icon_color.unwrap_or(renderer_style.icon_color);

        // Menu roots should share the accent color that icons get in the header.
        let mut text_color = if matches!(self.style, crate::theme::Button::MenuRoot) {
            icon_color
        } else {
            styling.text_color.unwrap_or(renderer_style.text_color)
        };

        if let Some(alpha) = headerbar_alpha {
            icon_color.a = alpha;
            text_color.a = alpha;
        }

        draw::<_, crate::Theme>(
            renderer,
            bounds,
            *viewport,
            &styling,
            |renderer, _styling| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    &renderer::Style {
                        icon_color,
                        text_color,
                        scale_factor: renderer_style.scale_factor,
                    },
                    content_layout.with_virtual_offset(layout.virtual_offset()),
                    cursor,
                    &viewport.intersection(&bounds).unwrap_or_default(),
                );
            },
            matches!(self.variant, Variant::Image { .. }),
        );

        if let Variant::Image {
            close_icon,
            on_remove,
        } = &self.variant
        {
            renderer.with_layer(*viewport, |renderer| {
                let selection_background = theme.selection_background();

                let c_rad = THEME.lock().unwrap().cosmic().corner_radii;

                if self.selected {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                width: 24.0,
                                height: 20.0,
                                x: bounds.x + styling.border_width,
                                y: bounds.y + (bounds.height - 20.0 - styling.border_width),
                            },
                            border: Border {
                                radius: [
                                    c_rad.radius_0[0],
                                    c_rad.radius_s[1],
                                    c_rad.radius_0[2],
                                    c_rad.radius_s[3],
                                ]
                                .into(),
                                ..Default::default()
                            },
                            shadow: Shadow::default(),
                        },
                        selection_background,
                    );

                    let svg_handle = svg::Svg::new(crate::widget::common::object_select().clone())
                        .color(icon_color);
                    let bounds = Rectangle {
                        width: 16.0,
                        height: 16.0,
                        x: bounds.x + 5.0 + styling.border_width,
                        y: bounds.y + (bounds.height - 18.0 - styling.border_width),
                    };
                    if bounds.intersects(viewport) {
                        iced_core::svg::Renderer::draw_svg(renderer, svg_handle, bounds);
                    }
                }

                if on_remove.is_some() {
                    if let Some(position) = cursor.position() {
                        if bounds.contains(position) {
                            let bounds = removal_bounds(layout.bounds(), 4.0);
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds,
                                    shadow: Shadow::default(),
                                    border: Border {
                                        radius: c_rad.radius_m.into(),
                                        ..Default::default()
                                    },
                                },
                                selection_background,
                            );
                            let svg_handle = svg::Svg::new(close_icon.clone()).color(icon_color);
                            iced_core::svg::Renderer::draw_svg(
                                renderer,
                                svg_handle,
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
            });
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            layout.with_virtual_offset(layout.virtual_offset()),
            cursor,
            self.on_press.is_some(),
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        mut translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let position = layout.bounds().position();
        translation.x += position.x;
        translation.y += position.y;
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
            renderer,
            translation,
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
            A11yNode, A11yTree,
            accesskit::{Action, DefaultActionVerb, NodeBuilder, NodeId, Rect, Role},
        };
        // TODO why is state None sometimes?
        if matches!(state.state, iced_core::widget::tree::State::None) {
            tracing::info!("Button state is missing.");
            return A11yTree::default();
        }

        let child_layout = layout.children().next().unwrap();
        let child_tree = state.children.first();

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
                node.set_described_by(id.iter().cloned().map(NodeId::from).collect::<Vec<_>>());
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
            node.set_disabled();
        }
        if is_hovered {
            node.set_hovered();
        }
        node.set_default_action_verb(DefaultActionVerb::Click);

        if let Some(child_tree) = child_tree.map(|child_tree| {
            self.content.as_widget().a11y_nodes(
                child_layout.with_virtual_offset(layout.virtual_offset()),
                child_tree,
                p,
            )
        }) {
            A11yTree::node_with_child_tree(A11yNode::new(node, self.id.clone()), child_tree)
        } else {
            A11yTree::leaf(node, self.id.clone())
        }
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message: Clone + 'a> From<Button<'a, Message>> for crate::Element<'a, Message> {
    fn from(button: Button<'a, Message>) -> Self {
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
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the [`Button`] is currently focused or not.
    #[inline]
    pub fn is_focused(self) -> bool {
        self.is_focused
    }

    /// Returns whether the [`Button`] is currently hovered or not.
    #[inline]
    pub fn is_hovered(self) -> bool {
        self.is_hovered
    }

    /// Focuses the [`Button`].
    #[inline]
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`Button`].
    #[inline]
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`Button`]
/// accordingly.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn update<'a, Message: Clone>(
    _id: Id,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_press: Option<&dyn Fn(Vector, Rectangle) -> Message>,
    on_press_down: Option<&dyn Fn(Vector, Rectangle) -> Message>,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            // Unfocus the button on clicks in case another widget was clicked.
            let state = state();
            state.unfocus();

            if on_press.is_some() || on_press_down.is_some() {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    state.is_pressed = true;

                    if let Some(on_press_down) = on_press_down {
                        let msg = (on_press_down)(layout.virtual_offset(), layout.bounds());
                        shell.publish(msg);
                    }

                    return event::Status::Captured;
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if let Some(on_press) = on_press {
                let state = state();

                if state.is_pressed {
                    state.is_pressed = false;

                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let msg = (on_press)(layout.virtual_offset(), layout.bounds());
                        shell.publish(msg);
                    }

                    return event::Status::Captured;
                }
            } else if on_press_down.is_some() {
                let state = state();
                state.is_pressed = false;
            }
        }
        #[cfg(feature = "a11y")]
        Event::A11y(event_id, iced_accessibility::accesskit::ActionRequest { action, .. }) => {
            let state = state();
            if let Some(on_press) = matches!(action, iced_accessibility::accesskit::Action::Default)
                .then_some(on_press)
                .flatten()
            {
                state.is_pressed = false;
                let msg = (on_press)(layout.virtual_offset(), layout.bounds());

                shell.publish(msg);
            }
            return event::Status::Captured;
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
            if let Some(on_press) = on_press {
                let state = state();
                if state.is_focused && key == keyboard::Key::Named(keyboard::key::Named::Enter) {
                    state.is_pressed = true;
                    let msg = (on_press)(layout.virtual_offset(), layout.bounds());

                    shell.publish(msg);
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
pub fn draw<Renderer: iced_core::Renderer, Theme>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    viewport_bounds: Rectangle,
    styling: &super::style::Style,
    draw_contents: impl FnOnce(&mut Renderer, &Style),
    is_image: bool,
) where
    Theme: super::style::Catalog,
{
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
                border: Border {
                    width: styling.outline_width,
                    color: styling.outline_color,
                    radius: styling.border_radius,
                },
                shadow: Shadow::default(),
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
                    border: Border {
                        radius: styling.border_radius,
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                },
                Background::Color([0.0, 0.0, 0.0, 0.5].into()),
            );
        }

        // Draw the button background first.
        if let Some(background) = styling.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: styling.border_radius,
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                },
                background,
            );
        }

        // Then button overlay if any.
        if let Some(overlay) = styling.overlay {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: styling.border_radius,
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                },
                overlay,
            );
        }

        // Then draw the button contents onto the background.
        draw_contents(renderer, styling);

        let mut clipped_bounds = viewport_bounds.intersection(&bounds).unwrap_or_default();
        clipped_bounds.height += styling.border_width;
        clipped_bounds.width += 1.0;

        // Finish by drawing the border above the contents.
        renderer.with_layer(clipped_bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        width: styling.border_width,
                        color: styling.border_color,
                        radius: styling.border_radius,
                    },
                    shadow: Shadow::default(),
                },
                Color::TRANSPARENT,
            );
        })
    } else {
        draw_contents(renderer, styling);
    }
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

    let mut content = layout_content(renderer, &limits.shrink(padding));
    let padding = padding.fit(content.size(), limits.max());
    let size = limits
        .shrink(padding)
        .resolve(width, height, content.size())
        .expand(padding);

    content = content.move_to(Point::new(padding.left, padding.top));

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

/// Produces a [`Task`] that focuses the [`Button`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::Widget(Box::new(operation::focusable::focus(id))))
}

impl operation::Focusable for State {
    #[inline]
    fn is_focused(&self) -> bool {
        Self::is_focused(*self)
    }

    #[inline]
    fn focus(&mut self) {
        Self::focus(self);
    }

    #[inline]
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
