// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Right-click context menu for widgets with selectable text.
//!
//! Use [`context_menu_overlay`] from your widget's `overlay()` method
//! with any widget that implements
//! [`HasSelectableText`](iced_core::widget::text::HasSelectableText)
//! to get a context menu with Copy, Select All, and optionally Cut/Paste.
//!
//! Internally uses libcosmic's [`Menu`](crate::widget::menu) system for
//! proper rendering, hover effects, and positioning.
//!
//! On Wayland, [`create_text_context_popup`] can be used instead to show
//! the context menu as a native popup surface.

pub use iced_core::widget::text::HasSelectableText;

use crate::widget::RcElementWrapper;
use crate::widget::menu::{
    self, CloseCondition, ItemHeight, ItemWidth, Menu, MenuBarState, PathHighlight, menu_roots_diff,
};
use crate::{theme, widget};

use iced_core::layout::Limits;
use iced_core::widget::Tree;
use iced_core::{
    Clipboard, Layout, Point, Rectangle, Shell, Size, Vector, clipboard, mouse, overlay, renderer,
};
use iced_widget::core::event;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

/// Shared state for communicating deferred context menu actions
/// from a Wayland popup back to the owning text widget.
pub(crate) type PendingAction = Arc<Mutex<Option<TextCtxAction>>>;

/// Creates a new [`PendingAction`] for use with popup-based context menus.
pub(crate) fn pending_action() -> PendingAction {
    Arc::new(Mutex::new(None))
}

/// Takes a pending action if one was set by a popup menu, and returns it.
pub(crate) fn take_pending_action(pending: &PendingAction) -> Option<TextCtxAction> {
    pending.lock().ok().and_then(|mut guard| guard.take())
}

use std::cell::Cell;

thread_local! {
    static CURRENT_WINDOW_ID: Cell<iced_core::window::Id> = const { Cell::new(iced_core::window::Id::NONE) };
}

#[cfg(feature = "wayland")]
use iced_runtime::platform_specific::wayland::popup::SctkPopupSettings;

/// A request to create a text context-menu popup surface, queued by a widget
/// during `update()` and drained by `Cosmic::update()` so the popup goes
/// through the normal `get_popup()` Task + `surface_views` pipeline.
#[cfg(feature = "wayland")]
pub(crate) struct PopupRequest {
    settings: SctkPopupSettings,
    menu: Menu<'static, TextCtxAction>,
    selected_text: Option<String>,
    pending_action: PendingAction,
}

#[cfg(feature = "wayland")]
thread_local! {
    static PENDING_POPUP_REQUESTS: std::cell::RefCell<Vec<PopupRequest>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn set_current_window_id(id: iced_core::window::Id) {
    CURRENT_WINDOW_ID.set(id);
}

pub(crate) fn current_window_id() -> iced_core::window::Id {
    CURRENT_WINDOW_ID.get()
}

/// Drains all popup requests queued by widgets this frame.
#[cfg(feature = "wayland")]
pub(crate) fn take_popup_requests() -> Vec<PopupRequest> {
    PENDING_POPUP_REQUESTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Consumes a [`PopupRequest`], returning the popup settings plus a view
/// builder. The builder rebuilds the menu element each frame from the
/// captured content, independent of app state.
#[cfg(feature = "wayland")]
#[allow(clippy::type_complexity)]
pub(crate) fn into_popup_view<Message: Clone + 'static>(
    req: PopupRequest,
) -> (
    SctkPopupSettings,
    Box<dyn Fn() -> crate::Element<'static, crate::Action<Message>> + Send + Sync>,
) {
    let PopupRequest {
        settings,
        menu,
        selected_text,
        pending_action,
    } = req;

    let view = Box::new(move || {
        let popup_widget: TextContextMenuPopup<Message> = TextContextMenuPopup {
            menu: menu.clone(),
            selected_text: selected_text.clone(),
            pending_action: pending_action.clone(),
            _phantom: std::marker::PhantomData,
        };
        crate::Element::from(crate::widget::container(popup_widget).center(iced_core::Length::Fill))
            .map(crate::action::app)
    });

    (settings, view)
}

#[cfg(feature = "wayland")]
thread_local! {
    static PENDING_POPUP_DESTROYS: std::cell::RefCell<Vec<iced_core::window::Id>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Queues a context-menu popup for teardown.
///
/// Mirrors [`create_text_context_popup`]'s request queue: widgets running
/// inside `update()` can't reach `Cosmic` to issue a Task, so they push the
/// id here and `Cosmic::update()` drains it into a `destroy_popup` Task that
/// flows through the normal surface pipeline.
#[cfg(feature = "wayland")]
fn queue_destroy_popup(id: iced_core::window::Id) {
    PENDING_POPUP_DESTROYS.with(|q| q.borrow_mut().push(id));
    wake_runtime();
}

#[cfg(feature = "wayland")]
static WAKE_TX: std::sync::OnceLock<iced_futures::futures::channel::mpsc::Sender<()>> =
    std::sync::OnceLock::new();

/// Stable identity for [`wake_subscription`].
#[cfg(feature = "wayland")]
struct PopupWake;

/// Nudges the runtime so `Cosmic::update()` runs and drains the popup queues.
#[cfg(feature = "wayland")]
fn wake_runtime() {
    if let Some(tx) = WAKE_TX.get() {
        let _ = tx.clone().try_send(());
    }
}

/// Subscription that backs [`wake_runtime`]: it owns the receiving end of the
/// wake channel and re-emits each ping as [`crate::Action::None`]. Add it to
/// the app's subscriptions (done by `Cosmic::subscription`) so popup creation
/// and teardown queued from widget `update()` get drained promptly.
#[cfg(feature = "wayland")]
pub(crate) fn wake_subscription<Message: Send + 'static>()
-> iced_futures::Subscription<crate::Action<Message>> {
    use iced_futures::futures::{SinkExt, StreamExt};
    iced_futures::Subscription::run_with(std::any::TypeId::of::<PopupWake>(), |_| {
        iced::stream::channel(
            16,
            |mut output: iced_futures::futures::channel::mpsc::Sender<crate::Action<Message>>| async move {
                let (tx, mut rx) = iced_futures::futures::channel::mpsc::channel(16);
                let _ = WAKE_TX.set(tx);
                while rx.next().await.is_some() {
                    let _ = output.send(crate::Action::None).await;
                }
            },
        )
    })
}

/// Drains all popup teardown requests queued by widgets this frame.
///
/// Called by `Cosmic::update()`, which turns each id into a `destroy_popup()`
/// Task. Drained before the creation queue so a destroy-then-recreate (a
/// second right-click reusing the same id) keeps its order.
#[cfg(wayland_platform)]
pub(crate) fn take_popup_destroys() -> Vec<iced_core::window::Id> {
    PENDING_POPUP_DESTROYS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Creates a context menu overlay for any widget implementing
/// [`HasSelectableText`].
///
/// Call this from your widget's `overlay()` method. Pass `on_input` for
/// editable widgets so Cut and Paste can publish text-change messages.
///
/// The `menu_bar_state` parameter must be a persistent [`MenuBarState`]
/// stored in the widget's tree state.
pub(crate) fn context_menu_overlay<'a, W, Message>(
    widget: &'a W,
    tree: &'a mut Tree,
    on_input: Option<&'a dyn Fn(String) -> Message>,
    translation: Vector,
    menu_bar_state: MenuBarState,
) -> Option<overlay::Element<'a, Message, crate::Theme, crate::Renderer>>
where
    W: HasSelectableText + 'a,
    Message: Clone + 'static,
{
    let click_position = widget.context_menu_position(tree)?;
    let selected_text = widget.selected_text(tree);
    let is_editable = widget.is_editable();

    let mut menu_roots = build_menu_roots(is_editable, selected_text.is_some());
    menu_roots.iter_mut().for_each(menu::Tree::set_index);

    let bounds = Rectangle {
        x: click_position.x,
        y: click_position.y,
        width: 240.0,
        height: 240.0,
    };

    let item_count = menu_roots[0].children.len();
    menu_bar_state.inner.with_data_mut(|state| {
        let stale = state.menu_states.first().is_some_and(|ms| {
            ms.menu_bounds.child_positions.len() != item_count
                || (ms.menu_bounds.parent_bounds.x - bounds.x).abs() > 0.5
                || (ms.menu_bounds.parent_bounds.y - bounds.y).abs() > 0.5
        });
        if !state.open || stale {
            state.menu_states.clear();
            state.active_root.clear();
            state.open = true;
        }
        menu_roots_diff(&mut menu_roots, &mut state.tree);
    });

    let menu = Menu {
        tree: menu_bar_state.clone(),
        menu_roots: Cow::Owned(menu_roots),
        bounds_expand: 16,
        menu_overlays_parent: true,
        close_condition: CloseCondition {
            leave: false,
            click_outside: true,
            click_inside: true,
        },
        item_width: ItemWidth::Uniform(240),
        item_height: ItemHeight::Dynamic(40),
        bar_bounds: bounds,
        main_offset: -(bounds.height as i32),
        cross_offset: 0,
        root_bounds_list: vec![bounds],
        path_highlight: Some(PathHighlight::MenuActive),
        style: Cow::Owned(theme::menu_bar::MenuBarStyle::Default),
        position: Point::new(translation.x, translation.y),
        is_overlay: true,
        window_id: iced::window::Id::NONE,
        depth: 0,
        on_surface_action: None,
    };

    Some(overlay::Element::new(Box::new(TextMenuOverlay {
        menu,
        widget,
        tree,
        on_input,
    })))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextCtxAction {
    Copy,
    Cut,
    Paste,
    SelectAll,
}

fn build_menu_roots(is_editable: bool, has_selection: bool) -> Vec<menu::Tree<TextCtxAction>> {
    let mut items = Vec::with_capacity(4);

    if is_editable && has_selection {
        items.push(menu::Tree::from(crate::Element::from(
            menu::menu_button(vec![widget::text("Cut").into()]).on_press(TextCtxAction::Cut),
        )));
    }
    if has_selection {
        items.push(menu::Tree::from(crate::Element::from(
            menu::menu_button(vec![widget::text("Copy").into()]).on_press(TextCtxAction::Copy),
        )));
    }
    if is_editable {
        items.push(menu::Tree::from(crate::Element::from(
            menu::menu_button(vec![widget::text("Paste").into()]).on_press(TextCtxAction::Paste),
        )));
    }
    items.push(menu::Tree::from(crate::Element::from(
        menu::menu_button(vec![widget::text("Select All").into()])
            .on_press(TextCtxAction::SelectAll),
    )));

    vec![menu::Tree::with_children(
        RcElementWrapper::new(crate::Element::from(widget::Row::new())),
        items,
    )]
}

struct TextMenuOverlay<'a, W, Message: Clone + 'static> {
    menu: Menu<'a, TextCtxAction>,
    widget: &'a W,
    tree: &'a mut Tree,
    on_input: Option<&'a dyn Fn(String) -> Message>,
}

impl<W, Message> overlay::Overlay<Message, crate::Theme, crate::Renderer>
    for TextMenuOverlay<'_, W, Message>
where
    W: HasSelectableText,
    Message: Clone + 'static,
{
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> iced_core::layout::Node {
        // Initialise the menu before the first draw so it appears at the click
        // position immediately
        let needs_init = self
            .menu
            .tree
            .inner
            .with_data(|state| state.open && state.menu_states.is_empty());

        if needs_init {
            let overlay_offset = Point::ORIGIN - self.menu.position;
            let bar_bounds = self.menu.bar_bounds;
            let main_offset = self.menu.main_offset as f32;
            let overlay_cursor = bar_bounds.center();

            let mut init_messages: Vec<TextCtxAction> = Vec::new();
            let mut init_shell = Shell::new(&mut init_messages);
            menu::init_root_menu(
                &mut self.menu,
                renderer,
                &mut init_shell,
                overlay_cursor,
                bounds,
                overlay_offset,
                bar_bounds,
                main_offset,
            );
        }

        self.menu.layout(
            renderer,
            Limits::NONE
                .min_width(bounds.width)
                .max_width(bounds.width)
                .min_height(bounds.height)
                .max_height(bounds.height),
        )
    }

    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.menu.draw(renderer, theme, style, layout, cursor);
    }

    fn update(
        &mut self,
        event: &event::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        // Right-clicks are not menu interactions. A right-click *on* the menu
        // (notably the press/release that opened it) is swallowed so it does
        // not close the menu. A right-click *outside* closes it — clearing the
        // menu position lets the widget reopen it at the new point on the same
        // event. Initialization happens in `layout()`.
        if matches!(
            event,
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
                | event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
        ) {
            let over_menu = cursor
                .position()
                .is_some_and(|p| layout.bounds().contains(p));
            if !over_menu {
                self.menu.tree.inner.with_data_mut(|state| {
                    state.menu_states.clear();
                    state.active_root.clear();
                    state.open = false;
                });
                self.widget.set_context_menu_position(self.tree, None);
            }
            return;
        }

        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        self.menu
            .update(event, layout, cursor, renderer, clipboard, &mut local_shell);

        if local_shell.is_event_captured() {
            shell.capture_event();
        }
        shell.request_redraw_at(local_shell.redraw_request());
        if local_shell.is_layout_invalid() {
            shell.invalidate_layout();
        }

        for action in local_messages {
            match action {
                TextCtxAction::Copy => {
                    self.widget.copy_to_clipboard(self.tree, clipboard);
                }
                TextCtxAction::Cut => {
                    self.widget.copy_to_clipboard(self.tree, clipboard);
                    if let Some(contents) = self.widget.delete_selection(self.tree) {
                        if let Some(on_input) = self.on_input {
                            shell.publish((on_input)(contents));
                        }
                    }
                }
                TextCtxAction::Paste => {
                    let content: String = clipboard
                        .read(clipboard::Kind::Standard)
                        .unwrap_or_default();
                    if let Some(contents) = self.widget.paste_text(self.tree, &content) {
                        if let Some(on_input) = self.on_input {
                            shell.publish((on_input)(contents));
                        }
                    }
                }
                TextCtxAction::SelectAll => {
                    self.widget.select_all(self.tree);
                }
            }
            self.widget.set_context_menu_position(self.tree, None);
        }

        let is_open = self.menu.tree.inner.with_data(|state| state.open);
        if !is_open {
            self.widget.set_context_menu_position(self.tree, None);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            mouse::Interaction::None
        }
    }
}

/// Queues a Wayland popup surface containing the text context menu.
///
/// Pushes a [`PopupRequest`] onto the request queue; `Cosmic::update()`
/// drains it and creates the popup through `get_popup()`, so it flows
/// through the normal Task + `surface_views` pipeline.
#[cfg(feature = "wayland")]
pub(crate) fn create_text_context_popup(
    click_position: Point,
    selected_text: Option<String>,
    is_editable: bool,
    has_selection: bool,
    menu_bar_state: &MenuBarState,
    pending_action: &PendingAction,
    renderer: &crate::Renderer,
    viewport: &Rectangle,
    cursor: mouse::Cursor,
) {
    use iced_runtime::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner};

    let window_id = current_window_id();
    if window_id == iced_core::window::Id::NONE {
        return;
    }

    let mut menu_roots = build_menu_roots(is_editable, has_selection);
    menu_roots.iter_mut().for_each(menu::Tree::set_index);

    let id = menu_bar_state.inner.with_data_mut(|state| {
        state.menu_states.clear();
        state.active_root.clear();
        menu_roots_diff(&mut menu_roots, &mut state.tree);
        if let Some(id) = state.popup_id.get(&window_id).copied() {
            queue_destroy_popup(id);
            state.view_cursor = cursor;
            id
        } else {
            state.open = true;
            state.view_cursor = cursor;
            iced::window::Id::unique()
        }
    });

    let bounds = Rectangle {
        x: click_position.x,
        y: click_position.y,
        width: 240.0,
        height: 240.0,
    };

    let mut popup_menu: Menu<'static, TextCtxAction> = Menu {
        tree: menu_bar_state.clone(),
        menu_roots: Cow::Owned(menu_roots),
        bounds_expand: 16,
        menu_overlays_parent: true,
        close_condition: CloseCondition {
            leave: false,
            click_outside: true,
            click_inside: true,
        },
        item_width: ItemWidth::Uniform(240),
        item_height: ItemHeight::Dynamic(40),
        bar_bounds: bounds,
        main_offset: -(bounds.height as i32),
        cross_offset: 0,
        root_bounds_list: vec![bounds],
        path_highlight: Some(PathHighlight::MenuActive),
        style: Cow::Owned(theme::menu_bar::MenuBarStyle::Default),
        position: Point::new(0., 0.),
        is_overlay: false,
        window_id: id,
        depth: 0,
        on_surface_action: None,
    };

    {
        let mut init_messages: Vec<TextCtxAction> = Vec::new();
        let mut init_shell = Shell::new(&mut init_messages);
        menu::init_root_menu(
            &mut popup_menu,
            renderer,
            &mut init_shell,
            cursor.position().unwrap_or_default(),
            viewport.size(),
            Vector::new(0., 0.),
            bounds,
            -(bounds.height),
        );
    }

    let anchor_rect = menu_bar_state.inner.with_data_mut(|state| {
        state.popup_id.insert(window_id, id);
        let pos = cursor.position().unwrap_or_default();
        iced::Rectangle {
            x: pos.x as i32,
            y: pos.y as i32,
            width: 1,
            height: 1,
        }
    });

    let menu_node = popup_menu.layout(renderer, iced::Limits::NONE.min_width(1.).min_height(1.));
    let popup_size = menu_node.size();

    let positioner = SctkPositioner {
        size: Some((
            popup_size.width.ceil() as u32 + 2,
            popup_size.height.ceil() as u32 + 2,
        )),
        anchor_rect,
        anchor: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Anchor::None,
        gravity: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::BottomRight,
        reactive: true,
        ..Default::default()
    };

    // Queue the request. `Cosmic::update()` drains it and creates the popup
    // through `get_popup()`, so it flows through the normal Task +
    // `surface_views` pipeline (rendering and teardown included).
    let settings = SctkPopupSettings {
        parent: window_id,
        id,
        positioner,
        parent_size: None,
        grab: true,
        close_with_children: false,
        input_zone: None,
    };

    PENDING_POPUP_REQUESTS.with(|q| {
        q.borrow_mut().push(PopupRequest {
            settings,
            menu: popup_menu,
            selected_text,
            pending_action: pending_action.clone(),
        });
    });
    wake_runtime();
}

/// Dismisses this widget's open context-menu popup on an outside click,
/// touch, or Escape.
#[cfg(feature = "wayland")]
pub(crate) fn dismiss_popup_on_event(menu_bar_state: &MenuBarState, event: &event::Event) {
    let is_dismiss = matches!(
        event,
        event::Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left | mouse::Button::Middle
        )) | event::Event::Keyboard(iced_core::keyboard::Event::KeyPressed {
            key: iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape),
            ..
        }) | event::Event::Touch(iced_core::touch::Event::FingerPressed { .. })
    );
    if !is_dismiss {
        return;
    }

    let window_id = current_window_id();
    let popup_id = menu_bar_state
        .inner
        .with_data(|state| state.popup_id.get(&window_id).copied());
    if let Some(popup_id) = popup_id {
        menu_bar_state.inner.with_data_mut(|state| {
            state.popup_id.retain(|_, v| *v != popup_id);
            state.reset();
        });
        queue_destroy_popup(popup_id);
    }
}

/// Widget that wraps [`Menu`] inside a Wayland popup and intercepts
/// [`TextCtxAction`] messages for clipboard operations.
#[derive(Clone)]
struct TextContextMenuPopup<Message: Clone + 'static> {
    menu: Menu<'static, TextCtxAction>,
    selected_text: Option<String>,
    pending_action: PendingAction,
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message: Clone + 'static> iced_core::widget::Widget<Message, crate::Theme, crate::Renderer>
    for TextContextMenuPopup<Message>
{
    fn size(&self) -> Size<iced_core::Length> {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::size(&self.menu)
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::tag(&self.menu)
    }

    fn state(&self) -> iced_core::widget::tree::State {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::state(&self.menu)
    }

    fn children(&self) -> Vec<iced_core::widget::Tree> {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::children(&self.menu)
    }

    fn diff(&mut self, tree: &mut iced_core::widget::Tree) {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::diff(&mut self.menu, tree);
    }

    fn layout(
        &mut self,
        tree: &mut iced_core::widget::Tree,
        renderer: &crate::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::layout(
            &mut self.menu,
            tree,
            renderer,
            limits,
        )
    }

    fn draw(
        &self,
        tree: &iced_core::widget::Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::draw(
            &self.menu, tree, renderer, theme, style, layout, cursor, viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut iced_core::widget::Tree,
        event: &event::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        #[cfg(feature = "wayland")]
        {
            use iced_core::event::wayland::PopupEvent;
            let popup_event = match event {
                event::Event::PlatformSpecific(iced_core::event::PlatformSpecific::Wayland(
                    iced_core::event::wayland::Event::Popup(e, _, _),
                )) => Some(e),
                _ => None,
            };
            if matches!(popup_event, Some(PopupEvent::Done | PopupEvent::Unfocused)) {
                let popup_id = self.menu.window_id;
                self.menu.tree.inner.with_data_mut(|state| {
                    state.popup_id.retain(|_, v| *v != popup_id);
                    state.reset();
                });
                // `Done` means the surface was already destroyed by the
                // compositor; only `Unfocused` needs an explicit destroy.
                if matches!(popup_event, Some(PopupEvent::Unfocused)) {
                    queue_destroy_popup(popup_id);
                }
                return;
            }
        }

        // Escape dismisses the popup. Under the grab, keyboard input is
        // delivered to the popup surface, so handle it here.
        #[cfg(feature = "wayland")]
        if matches!(
            event,
            event::Event::Keyboard(iced_core::keyboard::Event::KeyPressed {
                key: iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape),
                ..
            })
        ) {
            let popup_id = self.menu.window_id;
            self.menu.tree.inner.with_data_mut(|state| {
                state.popup_id.retain(|_, v| *v != popup_id);
                state.reset();
            });
            queue_destroy_popup(popup_id);
            shell.capture_event();
            return;
        }

        let mut local_messages: Vec<TextCtxAction> = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        {
            use iced_core::widget::Widget;
            Widget::<TextCtxAction, crate::Theme, crate::Renderer>::update(
                &mut self.menu,
                tree,
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );
        }

        if local_shell.is_event_captured() {
            shell.capture_event();
        }
        shell.request_redraw_at(local_shell.redraw_request());
        if local_shell.is_layout_invalid() {
            shell.invalidate_layout();
        }

        for action in local_messages {
            match action {
                TextCtxAction::Copy => {
                    if let Some(ref text) = self.selected_text {
                        clipboard.write(clipboard::Kind::Standard, text.clone());
                    }
                }
                TextCtxAction::Cut => {
                    if let Some(ref text) = self.selected_text {
                        clipboard.write(clipboard::Kind::Standard, text.clone());
                    }
                    if let Ok(mut guard) = self.pending_action.lock() {
                        *guard = Some(TextCtxAction::Cut);
                    }
                }
                TextCtxAction::Paste => {
                    if let Ok(mut guard) = self.pending_action.lock() {
                        *guard = Some(TextCtxAction::Paste);
                    }
                }
                TextCtxAction::SelectAll => {
                    if let Ok(mut guard) = self.pending_action.lock() {
                        *guard = Some(TextCtxAction::SelectAll);
                    }
                }
            }
        }

        // Under the popup grab the parent widget receives no events, so the
        // popup must tear itself down once its menu has closed — whether an
        // item was chosen (`click_inside`) or the user clicked away
        // (`click_outside`).
        #[cfg(feature = "wayland")]
        {
            let menu_closed = self.menu.tree.inner.with_data(|state| !state.open);
            if menu_closed {
                let popup_id = self.menu.window_id;
                self.menu.tree.inner.with_data_mut(|state| {
                    state.popup_id.retain(|_, v| *v != popup_id);
                    state.reset();
                });
                queue_destroy_popup(popup_id);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &iced_core::widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        use iced_core::widget::Widget;
        Widget::<TextCtxAction, crate::Theme, crate::Renderer>::mouse_interaction(
            &self.menu, tree, layout, cursor, viewport, renderer,
        )
    }
}

impl<Message: Clone + 'static> From<TextContextMenuPopup<Message>>
    for crate::Element<'static, Message>
{
    fn from(popup: TextContextMenuPopup<Message>) -> Self {
        Self::new(popup)
    }
}
