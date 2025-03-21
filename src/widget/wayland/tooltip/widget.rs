// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Allow your users to perform actions by pressing a button.
//!
//! A [`Tooltip`] has some local [`State`].

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::Task;
use iced_runtime::core::widget::Id;

use iced_core::event::{self, Event};
use iced_core::renderer;
use iced_core::touch;
use iced_core::widget::Operation;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Background, Clipboard, Color, Layout, Length, Padding, Point, Rectangle, Shell, Vector, Widget,
};
use iced_core::{Border, mouse};
use iced_core::{Shadow, overlay};
use iced_core::{layout, svg};

pub use super::{Catalog, Style};

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
pub struct Tooltip<'a, Message, TopLevelMessage> {
    id: Id,
    #[cfg(feature = "a11y")]
    name: Option<std::borrow::Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    label: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    content: crate::Element<'a, Message>,
    on_leave: Message,
    on_surface_action: Box<dyn Fn(crate::surface::Action) -> Message>,
    width: Length,
    height: Length,
    padding: Padding,
    selected: bool,
    style: crate::theme::Tooltip,
    delay: Option<Duration>,
    settings: Option<
        Arc<
            dyn Fn(Rectangle) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
                + Send
                + Sync
                + 'static,
        >,
    >,
    view: Arc<
        dyn Fn() -> crate::Element<'static, crate::Action<TopLevelMessage>> + Send + Sync + 'static,
    >,
}

impl<'a, Message, TopLevelMessage> Tooltip<'a, Message, TopLevelMessage> {
    /// Creates a new [`Tooltip`] with the given content.
    pub fn new(
        content: impl Into<crate::Element<'a, Message>>,
        settings: Option<
            impl Fn(Rectangle) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
            + Send
            + Sync
            + 'static,
        >,
        view: impl Fn() -> crate::Element<'static, crate::Action<TopLevelMessage>>
        + Send
        + Sync
        + 'static,
        on_leave: Message,
        on_surface_action: impl Fn(crate::surface::Action) -> Message + 'static,
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
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(0.0),
            selected: false,
            style: crate::theme::Tooltip::default(),
            on_leave,
            on_surface_action: Box::new(on_surface_action),
            delay: None,
            settings: if let Some(s) = settings {
                Some(Arc::new(s))
            } else {
                None
            },
            view: Arc::new(view),
        }
    }

    pub fn delay(mut self, dur: Duration) -> Self {
        self.delay = Some(dur);
        self
    }

    /// Sets the [`Id`] of the [`Tooltip`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the width of the [`Tooltip`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tooltip`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Tooltip`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the widget to a selected state.
    ///
    /// Displays a selection indicator on image buttons.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;

        self
    }

    /// Sets the style variant of this [`Tooltip`].
    pub fn class(mut self, style: crate::theme::Tooltip) -> Self {
        self.style = style;
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Tooltip`].
    pub fn name(mut self, name: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Tooltip`].
    pub fn description_widget<T: iced_accessibility::Describes>(mut self, description: &T) -> Self {
        self.description = Some(iced_accessibility::Description::Id(
            description.description(),
        ));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Tooltip`].
    pub fn description(mut self, description: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.description = Some(iced_accessibility::Description::Text(description.into()));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the label of the [`Tooltip`].
    pub fn label(mut self, label: &dyn iced_accessibility::Labels) -> Self {
        self.label = Some(label.label().into_iter().map(|l| l.into()).collect());
        self
    }
}

impl<'a, Message: 'static + Clone, TopLevelMessage: 'static + Clone>
    Widget<Message, crate::Theme, crate::Renderer> for Tooltip<'a, Message, TopLevelMessage>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
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
        let status = update(
            self.id.clone(),
            event.clone(),
            layout,
            cursor,
            shell,
            self.settings.as_ref(),
            &self.view,
            self.delay,
            &self.on_leave,
            &self.on_surface_action,
            || tree.state.downcast_mut::<State>(),
        );
        status.merge(
            self.content.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
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
            ),
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
        let content_layout = layout.children().next().unwrap();

        let state = tree.state.downcast_ref::<State>();

        let styling = theme.style(&self.style);

        let icon_color = styling.icon_color.unwrap_or(renderer_style.icon_color);

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
                        text_color: styling.text_color,
                        scale_factor: renderer_style.scale_factor,
                    },
                    content_layout.with_virtual_offset(layout.virtual_offset()),
                    cursor,
                    &viewport.intersection(&bounds).unwrap_or_default(),
                );
            },
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
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
        let c_layout = layout.children().next().unwrap();

        self.content.as_widget().a11y_nodes(
            c_layout.with_virtual_offset(layout.virtual_offset()),
            state,
            p,
        )
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message: Clone + 'static, TopLevelMessage: Clone + 'static>
    From<Tooltip<'a, Message, TopLevelMessage>> for crate::Element<'a, Message>
{
    fn from(button: Tooltip<'a, Message, TopLevelMessage>) -> Self {
        Self::new(button)
    }
}

/// The local state of a [`Tooltip`].
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_field_names)]
pub struct State {
    is_hovered: Arc<Mutex<bool>>,
}

impl State {
    /// Returns whether the [`Tooltip`] is currently hovered or not.
    pub fn is_hovered(self) -> bool {
        let guard = self.is_hovered.lock().unwrap();
        *guard
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`Tooltip`]
/// accordingly.
#[allow(clippy::needless_pass_by_value)]
pub fn update<'a, Message: Clone + 'static, TopLevelMessage: Clone + 'static>(
    _id: Id,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    settings: Option<
        &Arc<
            dyn Fn(Rectangle) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
                + Send
                + Sync
                + 'static,
        >,
    >,
    view: &Arc<
        dyn Fn() -> crate::Element<'static, crate::Action<TopLevelMessage>> + Send + Sync + 'static,
    >,
    delay: Option<Duration>,
    on_leave: &Message,
    on_surface_action: &dyn Fn(crate::surface::Action) -> Message,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    match event {
        Event::Touch(touch::Event::FingerLifted { .. }) => {
            let state = state();
            let mut guard = state.is_hovered.lock().unwrap();
            if *guard {
                *guard = false;

                shell.publish(on_leave.clone());

                return event::Status::Captured;
            }
        }

        Event::Touch(touch::Event::FingerLost { .. }) | Event::Mouse(mouse::Event::CursorLeft) => {
            let state = state();
            let mut guard = state.is_hovered.lock().unwrap();

            if *guard {
                *guard = false;

                shell.publish(on_leave.clone());
            }
        }

        Event::Mouse(mouse::Event::CursorMoved { .. }) => {
            let state = state();
            let bounds = layout.bounds();
            let is_hovered = state.is_hovered.clone();
            let mut guard = state.is_hovered.lock().unwrap();

            if *guard {
                *guard = cursor.is_over(bounds);
                if !*guard {
                    shell.publish(on_leave.clone());
                }
            } else {
                *guard = cursor.is_over(bounds);
                if *guard {
                    if let Some(settings) = settings {
                        if let Some(delay) = delay {
                            let s = settings.clone();
                            let view = view.clone();
                            let bounds = layout.bounds();

                            let sm = crate::surface::Action::Task(Arc::new(move || {
                                let s = s.clone();
                                let view = view.clone();
                                let is_hovered = is_hovered.clone();
                                Task::future(async move {
                                    #[cfg(feature = "tokio")]
                                    {
                                        _ = tokio::time::sleep(delay).await;
                                    }
                                    #[cfg(feature = "async-std")]
                                    {
                                        _ = async_std::task::sleep(delay).await;
                                    }
                                    let is_hovered = is_hovered.clone();
                                    let g = is_hovered.lock().unwrap();
                                    if !*g {
                                        return crate::surface::Action::Ignore;
                                    }
                                    let boxed: Box<
                                        dyn Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
                                            + Send
                                            + Sync
                                            + 'static,
                                    > = Box::new(move || s(bounds));
                                    let boxed: Box<dyn Any + Send + Sync + 'static> =
                                        Box::new(boxed);
                                    crate::surface::Action::Popup(
                                        Arc::new(boxed),
                                        Some({
                                            let boxed: Box<
                                                dyn Fn() -> crate::Element<
                                                        'static,
                                                        crate::Action<TopLevelMessage>,
                                                    > + Send
                                                    + Sync
                                                    + 'static,
                                            > = Box::new(move || view());
                                            let boxed: Box<dyn Any + Send + Sync + 'static> =
                                                Box::new(boxed);
                                            Arc::new(boxed)
                                        }),
                                    )
                                })
                            }));

                            shell.publish((on_surface_action)(sm));
                        } else {
                            let s = settings.clone();
                            let view = view.clone();
                            let bounds = layout.bounds();

                            let boxed: Box<
                                dyn Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings
                                    + Send
                                    + Sync
                                    + 'static,
                            > = Box::new(move || s(bounds));
                            let boxed: Box<dyn Any + Send + Sync + 'static> = Box::new(boxed);

                            let sm = crate::surface::Action::Popup(
                                Arc::new(boxed),
                                Some({
                                    let boxed: Box<
                                        dyn Fn() -> crate::Element<
                                                'static,
                                                crate::Action<TopLevelMessage>,
                                            > + Send
                                            + Sync
                                            + 'static,
                                    > = Box::new(move || view());
                                    let boxed: Box<dyn Any + Send + Sync + 'static> =
                                        Box::new(boxed);
                                    Arc::new(boxed)
                                }),
                            );
                            shell.publish((on_surface_action)(sm));
                        }
                    }
                }
            }
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
    styling: &super::Style,
    draw_contents: impl FnOnce(&mut Renderer, &Style),
) where
    Theme: super::Catalog,
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

        // Then draw the button contents onto the background.
        draw_contents(renderer, styling);

        let mut clipped_bounds = viewport_bounds.intersection(&bounds).unwrap_or_default();
        clipped_bounds.height += styling.border_width;

        renderer.with_layer(clipped_bounds, |renderer| {
            // Finish by drawing the border above the contents.
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
        });
    } else {
        draw_contents(renderer, styling);
    }
}

/// Computes the layout of a [`Tooltip`].
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
