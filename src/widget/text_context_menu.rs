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
use std::mem;
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
use std::collections::HashMap;
use std::sync::LazyLock;

thread_local! {
    static CURRENT_WINDOW_ID: Cell<iced_core::window::Id> = const { Cell::new(iced_core::window::Id::NONE) };
}

/// Data needed to construct a popup view. Stored in a global registry
/// so `Cosmic::view()` can build the `Element` with the correct message type.
#[cfg(feature = "wayland")]
struct PopupViewData {
    menu: Menu<'static, TextCtxAction>,
    selected_text: Option<String>,
    pending_action: PendingAction,
}

#[cfg(feature = "wayland")]
static POPUP_VIEW_REGISTRY: LazyLock<Mutex<HashMap<iced_core::window::Id, PopupViewData>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn set_current_window_id(id: iced_core::window::Id) {
    CURRENT_WINDOW_ID.set(id);
}

pub(crate) fn current_window_id() -> iced_core::window::Id {
    CURRENT_WINDOW_ID.get()
}

/// Returns the popup view `Element` for a given popup ID, if one exists.
/// Called by `Cosmic::view()` where `Message` is known.
#[cfg(feature = "wayland")]
pub(crate) fn popup_view_for_id<Message: Clone + 'static>(
    id: iced_core::window::Id,
) -> Option<crate::Element<'static, crate::Action<Message>>> {
    let registry = POPUP_VIEW_REGISTRY.lock().ok()?;
    let data = registry.get(&id)?;
    let popup_widget: TextContextMenuPopup<Message> = TextContextMenuPopup {
        menu: data.menu.clone(),
        selected_text: data.selected_text.clone(),
        pending_action: data.pending_action.clone(),
        _phantom: std::marker::PhantomData,
    };
    drop(registry);
    Some(
        crate::Element::from(
            crate::widget::container(popup_widget).center(iced_core::Length::Fill),
        )
        .map(crate::action::app),
    )
}

/// Sends a wayland popup action directly to the SCTK event loop,
/// bypassing the iced Task system.
#[cfg(feature = "wayland")]
fn send_popup_direct(settings: iced_runtime::platform_specific::wayland::popup::SctkPopupSettings) {
    iced_winit::send_wayland_action_direct(
        iced_runtime::platform_specific::wayland::Action::Popup(
            iced_runtime::platform_specific::wayland::popup::Action::Popup { popup: settings },
        ),
    );
}

/// Sends a destroy popup action directly to the SCTK event loop.
#[cfg(feature = "wayland")]
fn send_destroy_popup_direct(id: iced_core::window::Id) {
    iced_winit::send_wayland_action_direct(
        iced_runtime::platform_specific::wayland::Action::Popup(
            iced_runtime::platform_specific::wayland::popup::Action::Destroy { id },
        ),
    );
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

    let item_count = menu_roots[0].children.len();
    menu_bar_state.inner.with_data_mut(|state| {
        let stale = state
            .menu_states
            .first()
            .is_some_and(|ms| ms.menu_bounds.child_positions.len() != item_count);
        if !state.open || stale {
            state.menu_states.clear();
            state.active_root.clear();
            state.open = true;
        }
        menu_roots_diff(&mut menu_roots, &mut state.tree);
    });

    let offscreen = Rectangle::new(Point::new(-10000.0, -10000.0), Size::ZERO);

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
        bar_bounds: offscreen,
        main_offset: -240,
        cross_offset: 0,
        root_bounds_list: vec![offscreen],
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
        click_position,
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
    click_position: Point,
}

impl<W, Message> overlay::Overlay<Message, crate::Theme, crate::Renderer>
    for TextMenuOverlay<'_, W, Message>
where
    W: HasSelectableText,
    Message: Clone + 'static,
{
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> iced_core::layout::Node {
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
        // Pre-initialize the menu if it hasn't been yet.
        let needs_init = self
            .menu
            .tree
            .inner
            .with_data(|state| state.open && state.menu_states.is_empty());

        if needs_init {
            let viewport = layout.bounds();
            let viewport_size = viewport.size();
            let overlay_offset = Point::ORIGIN - viewport.position();
            let overlay_cursor = cursor.position().unwrap_or_default() - overlay_offset;

            let init_bounds = Rectangle {
                x: self.click_position.x,
                y: self.click_position.y,
                width: 240.0,
                height: 240.0,
            };

            // Temporarily set real bounds so init_root_menu can find
            // root_bounds_list entries that contain the cursor.
            let main_offset = self.menu.main_offset as f32;
            let saved_bounds = mem::replace(&mut self.menu.bar_bounds, init_bounds);
            let saved_roots = mem::replace(&mut self.menu.root_bounds_list, vec![init_bounds]);

            let mut init_messages: Vec<TextCtxAction> = Vec::new();
            let mut init_shell = Shell::new(&mut init_messages);
            menu::init_root_menu(
                &mut self.menu,
                renderer,
                &mut init_shell,
                overlay_cursor,
                viewport_size,
                overlay_offset,
                init_bounds,
                main_offset,
            );

            // Restore offscreen bounds
            self.menu.bar_bounds = saved_bounds;
            self.menu.root_bounds_list = saved_roots;

            if init_shell.is_layout_invalid() {
                shell.invalidate_layout();
            }
        }

        // Right-button releases are meaningless for menu interaction
        if matches!(
            event,
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
        ) {
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

/// Creates a Wayland popup surface containing the text context menu.
///
/// Sends the popup action directly to the SCTK event loop and registers
/// the view data so `Cosmic::view()` can render it. No app-level wiring
/// is needed
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
            send_destroy_popup_direct(id);
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

    // Register view data so Cosmic::view() can render this popup.
    if let Ok(mut registry) = POPUP_VIEW_REGISTRY.lock() {
        registry.insert(
            id,
            PopupViewData {
                menu: popup_menu,
                selected_text,
                pending_action: pending_action.clone(),
            },
        );
    }

    // Send the popup creation directly to the SCTK event loop.
    send_popup_direct(SctkPopupSettings {
        parent: window_id,
        id,
        positioner,
        parent_size: None,
        grab: true,
        close_with_children: false,
        input_zone: None,
    });
}

/// Checks whether a popup menu has closed and destroys it if needed.
/// Call this from the text widget's `update()` on each frame.
#[cfg(feature = "wayland")]
pub(crate) fn cleanup_text_popup(menu_bar_state: &MenuBarState) {
    let window_id = current_window_id();
    let should_destroy = menu_bar_state
        .inner
        .with_data(|state| !state.open && state.popup_id.contains_key(&window_id));
    if should_destroy {
        menu_bar_state.inner.with_data_mut(|state| {
            if let Some(popup_id) = state.popup_id.remove(&window_id) {
                send_destroy_popup_direct(popup_id);
            }
            state.reset();
        });
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
