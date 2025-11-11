// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

use super::Id;
use super::menu::{self, Menu};
use crate::widget::icon::{self, Handle};
use crate::{Element, surface};
use derive_setters::Setters;
use iced::window;
use iced_core::event::{self, Event};
use iced_core::text::{self, Paragraph, Text};
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector, Widget,
};
use iced_core::{Shadow, alignment, keyboard, layout, mouse, overlay, renderer, svg, touch};
use iced_widget::pick_list::{self, Catalog};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

pub type DropdownView<Message> = Arc<dyn Fn() -> Element<'static, Message> + Send + Sync>;
static AUTOSIZE_ID: LazyLock<crate::widget::Id> =
    LazyLock::new(|| crate::widget::Id::new("cosmic-applet-autosize"));

/// A widget for selecting a single value from a list of selections.
#[derive(Setters)]
pub struct Dropdown<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message, AppMessage>
where
    [S]: std::borrow::ToOwned,
{
    #[setters(skip)]
    id: Option<Id>,
    #[setters(skip)]
    on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync>,
    #[setters(skip)]
    selections: Cow<'a, [S]>,
    #[setters]
    icons: Cow<'a, [icon::Handle]>,
    #[setters(skip)]
    selected: Option<usize>,
    #[setters(into)]
    width: Length,
    gap: f32,
    #[setters(into)]
    padding: Padding,
    #[setters(strip_option)]
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    #[setters(strip_option)]
    font: Option<crate::font::Font>,
    #[setters(skip)]
    on_surface_action: Option<Arc<dyn Fn(surface::Action) -> Message + Send + Sync + 'static>>,
    #[setters(skip)]
    action_map: Option<Arc<dyn Fn(Message) -> AppMessage + 'static + Send + Sync>>,
    #[setters(strip_option)]
    window_id: Option<window::Id>,
    #[cfg(all(feature = "winit", feature = "wayland"))]
    positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
}

impl<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message: 'static, AppMessage: 'static>
    Dropdown<'a, S, Message, AppMessage>
where
    [S]: std::borrow::ToOwned,
{
    /// The default gap.
    pub const DEFAULT_GAP: f32 = 4.0;

    /// The default padding.
    pub const DEFAULT_PADDING: Padding = Padding::new(8.0);

    /// Creates a new [`Dropdown`] with the given list of selections, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(
        selections: Cow<'a, [S]>,
        selected: Option<usize>,
        on_selected: impl Fn(usize) -> Message + 'static + Send + Sync,
    ) -> Self {
        Self {
            id: None,
            on_selected: Arc::new(on_selected),
            selections,
            icons: Cow::Borrowed(&[]),
            selected,
            width: Length::Shrink,
            gap: Self::DEFAULT_GAP,
            padding: Self::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::Relative(1.2),
            font: None,
            window_id: None,
            #[cfg(all(feature = "winit", feature = "wayland"))]
            positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner::default(),
            on_surface_action: None,
            action_map: None,
        }
    }

    #[cfg(all(feature = "winit", feature = "wayland"))]
    /// Handle dropdown requests for popup creation.
    /// Intended to be used with [`crate::app::message::get_popup`]
    pub fn with_popup<NewAppMessage>(
        self,
        parent_id: window::Id,
        on_surface_action: impl Fn(surface::Action) -> Message + Send + Sync + 'static,
        action_map: impl Fn(Message) -> NewAppMessage + Send + Sync + 'static,
    ) -> Dropdown<'a, S, Message, NewAppMessage> {
        let Self {
            id,
            on_selected,
            selections,
            icons,
            selected,
            width,
            gap,
            padding,
            text_size,
            text_line_height,
            font,
            positioner,
            ..
        } = self;

        Dropdown::<'a, S, Message, NewAppMessage> {
            id,
            on_selected,
            selections,
            icons,
            selected,
            width,
            gap,
            padding,
            text_size,
            text_line_height,
            font,
            on_surface_action: Some(Arc::new(on_surface_action)),
            action_map: Some(Arc::new(action_map)),
            window_id: Some(parent_id),
            positioner,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    #[cfg(all(feature = "winit", feature = "wayland"))]
    pub fn with_positioner(
        mut self,
        positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
    ) -> Self {
        self.positioner = positioner;
        self
    }
}

impl<
    S: AsRef<str> + Send + Sync + Clone + 'static,
    Message: 'static + Clone,
    AppMessage: 'static + Clone,
> Widget<Message, crate::Theme, crate::Renderer> for Dropdown<'_, S, Message, AppMessage>
where
    [S]: std::borrow::ToOwned,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        let mut selections_changed = state.selections.len() != self.selections.len();

        state
            .selections
            .resize_with(self.selections.len(), crate::Plain::default);
        state.hashes.resize(self.selections.len(), 0);

        for (i, selection) in self.selections.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            selection.as_ref().hash(&mut hasher);
            let text_hash = hasher.finish();

            if state.hashes[i] == text_hash {
                continue;
            }

            selections_changed = true;
            state.hashes[i] = text_hash;
            state.selections[i].update(Text {
                content: selection.as_ref(),
                bounds: Size::INFINITY,
                // TODO use the renderer default size
                size: iced::Pixels(self.text_size.unwrap_or(14.0)),
                line_height: self.text_line_height,
                font: self.font.unwrap_or_else(crate::font::default),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            });
        }

        if state.is_open.load(Ordering::SeqCst) && selections_changed {
            state.close_operation = true;
            state.open_operation = true;
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
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
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.text_line_height,
            self.font,
            self.selected.and_then(|id| {
                self.selections
                    .get(id)
                    .map(AsRef::as_ref)
                    .zip(tree.state.downcast_mut::<State>().selections.get_mut(id))
            }),
            !self.icons.is_empty(),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &crate::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update::<S, Message, AppMessage>(
            &event,
            layout,
            cursor,
            shell,
            #[cfg(all(feature = "winit", feature = "wayland"))]
            self.positioner.clone(),
            self.on_selected.clone(),
            self.selected,
            &self.selections,
            || tree.state.downcast_mut::<State>(),
            self.window_id,
            self.on_surface_action.clone(),
            self.action_map.clone(),
            &self.icons,
            self.gap,
            self.padding,
            self.text_size,
            self.font,
            self.selected,
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        _style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let font = self.font.unwrap_or_else(crate::font::default);
        draw(
            renderer,
            theme,
            layout,
            cursor,
            self.gap,
            self.padding,
            self.text_size,
            self.text_line_height,
            font,
            self.selected.and_then(|id| self.selections.get(id)),
            self.selected.and_then(|id| self.icons.get(id)),
            tree.state.downcast_ref::<State>(),
            viewport,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.custom(state, self.id.as_ref());
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        #[cfg(all(feature = "winit", feature = "wayland"))]
        if self.window_id.is_some() || self.on_surface_action.is_some() {
            return None;
        }

        let state = tree.state.downcast_mut::<State>();

        overlay(
            layout,
            renderer,
            state,
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.text_line_height,
            self.font,
            &self.selections,
            &self.icons,
            self.selected,
            self.on_selected.as_ref(),
            translation,
            None,
        )
    }

    // #[cfg(feature = "a11y")]
    // /// get the a11y nodes for the widget
    // fn a11y_nodes(
    //     &self,
    //     layout: Layout<'_>,
    //     state: &Tree,
    //     p: mouse::Cursor,
    // ) -> iced_accessibility::A11yTree {
    //     // TODO
    // }
}

impl<
    'a,
    S: AsRef<str> + Send + Sync + Clone + 'static,
    Message: 'static + std::clone::Clone,
    AppMessage: 'static + std::clone::Clone,
> From<Dropdown<'a, S, Message, AppMessage>> for crate::Element<'a, Message>
where
    [S]: std::borrow::ToOwned,
{
    fn from(pick_list: Dropdown<'a, S, Message, AppMessage>) -> Self {
        Self::new(pick_list)
    }
}

/// The local state of a [`Dropdown`].
#[derive(Debug, Clone)]
pub struct State {
    icon: Option<svg::Handle>,
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: Arc<AtomicBool>,
    close_operation: bool,
    open_operation: bool,
    hovered_option: Arc<Mutex<Option<usize>>>,
    hashes: Vec<u64>,
    selections: Vec<crate::Plain>,
    popup_id: window::Id,
}

impl State {
    /// Creates a new [`State`] for a [`Dropdown`].
    pub fn new() -> Self {
        Self {
            icon: match icon::from_name("pan-down-symbolic").size(16).handle().data {
                icon::Data::Svg(handle) => Some(handle),
                icon::Data::Image(_) => None,
            },
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: Arc::new(AtomicBool::new(false)),
            hovered_option: Arc::new(Mutex::new(None)),
            selections: Vec::new(),
            hashes: Vec::new(),
            popup_id: window::Id::unique(),
            close_operation: false,
            open_operation: false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl super::operation::Dropdown for State {
    fn close(&mut self) {
        self.close_operation = true;
    }

    fn open(&mut self) {
        self.open_operation = true;
    }
}

/// Computes the layout of a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn layout(
    renderer: &crate::Renderer,
    limits: &layout::Limits,
    width: Length,
    gap: f32,
    padding: Padding,
    text_size: f32,
    text_line_height: text::LineHeight,
    font: Option<crate::font::Font>,
    selection: Option<(&str, &mut crate::Plain)>,
    has_icons: bool,
) -> layout::Node {
    use std::f32;

    let limits = limits.width(width).height(Length::Shrink).shrink(padding);

    let max_width = match width {
        Length::Shrink => {
            let measure = move |(label, paragraph): (_, &mut crate::Plain)| -> f32 {
                paragraph.update(Text {
                    content: label,
                    bounds: Size::new(f32::MAX, f32::MAX),
                    size: iced::Pixels(text_size),
                    line_height: text_line_height,
                    font: font.unwrap_or_else(crate::font::default),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::default(),
                });
                paragraph.min_width().round()
            };

            selection.map(measure).unwrap_or_default()
        }
        _ => 0.0,
    };

    let icon_size = if has_icons { 24.0 } else { 0.0 };

    let size = {
        let intrinsic = Size::new(
            max_width + icon_size + gap + 16.0,
            f32::from(text_line_height.to_absolute(Pixels(text_size))),
        );

        limits
            .resolve(width, Length::Shrink, intrinsic)
            .expand(padding)
    };

    layout::Node::new(size)
}

/// Processes an [`Event`] and updates the [`State`] of a [`Dropdown`]
/// accordingly.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn update<
    'a,
    S: AsRef<str> + Send + Sync + Clone + 'static,
    Message: Clone + 'static,
    AppMessage: Clone + 'static,
>(
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    #[cfg(all(feature = "winit", feature = "wayland"))]
    positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
    on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync + 'static>,
    selected: Option<usize>,
    selections: &[S],
    state: impl FnOnce() -> &'a mut State,
    _window_id: Option<window::Id>,
    on_surface_action: Option<Arc<dyn Fn(surface::Action) -> Message + Send + Sync + 'static>>,
    action_map: Option<Arc<dyn Fn(Message) -> AppMessage + Send + Sync + 'static>>,
    icons: &[icon::Handle],
    gap: f32,
    padding: Padding,
    text_size: Option<f32>,
    font: Option<crate::font::Font>,
    selected_option: Option<usize>,
) -> event::Status {
    let state = state();

    let open = |shell: &mut Shell<'_, Message>,
                state: &mut State,
                on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync + 'static>| {
        state.is_open.store(true, Ordering::Relaxed);
        let mut hovered_guard = state.hovered_option.lock().unwrap();
        *hovered_guard = selected;
        let id = window::Id::unique();
        state.popup_id = id;
        #[cfg(all(feature = "winit", feature = "wayland"))]
        if let Some(((on_surface_action, parent), action_map)) = on_surface_action
            .as_ref()
            .zip(_window_id)
            .zip(action_map.clone())
        {
            use iced_runtime::platform_specific::wayland::popup::{
                SctkPopupSettings, SctkPositioner,
            };
            let bounds = layout.bounds();
            let anchor_rect = Rectangle {
                x: bounds.x as i32,
                y: bounds.y as i32,
                width: bounds.width as i32,
                height: bounds.height as i32,
            };
            let icon_width = if icons.is_empty() { 0.0 } else { 24.0 };
            let measure = |_label: &str, selection_paragraph: &crate::Paragraph| -> f32 {
                selection_paragraph.min_width().round()
            };
            let pad_width = padding.horizontal().mul_add(2.0, 16.0);

            let selections_width = selections
                .iter()
                .zip(state.selections.iter_mut())
                .map(|(label, selection)| measure(label.as_ref(), selection.raw()))
                .fold(0.0, |next, current| current.max(next));

            let icons: Cow<'static, [Handle]> = Cow::Owned(icons.to_vec());
            let selections: Cow<'static, [S]> = Cow::Owned(selections.to_vec());
            let state = state.clone();
            let on_close = surface::action::destroy_popup(id);
            let on_surface_action_clone = on_surface_action.clone();
            let translation = layout.virtual_offset();
            let get_popup_action = surface::action::simple_popup::<AppMessage>(
                move || {
                    SctkPopupSettings {
                parent,
                id,
                input_zone: None,
                positioner: SctkPositioner {
                    size: Some((selections_width as u32 + gap as u32 + pad_width as u32 + icon_width as u32, 10)),
                    anchor_rect,
                    // TODO: left or right alignment based on direction?
                    anchor: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Anchor::BottomLeft,
                    gravity: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::BottomRight,
                    reactive: true,
                    offset: ((-padding.left - translation.x) as i32, -translation.y as i32),
                    constraint_adjustment: 9,
                    ..Default::default()
                },
                parent_size: None,
                grab: true,
                close_with_children: true,
            }
                },
                Some(Box::new(move || {
                    let action_map = action_map.clone();
                    let on_selected = on_selected.clone();
                    let e: Element<'static, crate::Action<AppMessage>> =
                        Element::from(menu_widget(
                            bounds,
                            &state,
                            gap,
                            padding,
                            text_size.unwrap_or(14.0),
                            selections.clone(),
                            icons.clone(),
                            selected_option,
                            Arc::new(move |i| on_selected.clone()(i)),
                            Some(on_surface_action_clone(on_close.clone())),
                        ))
                        .map(move |m| crate::Action::App(action_map.clone()(m)));
                    e
                })),
            );
            shell.publish(on_surface_action(get_popup_action));
        }
    };

    let is_open = state.is_open.load(Ordering::Relaxed);
    let refresh = state.close_operation && state.open_operation;

    if state.close_operation {
        state.close_operation = false;
        state.is_open.store(false, Ordering::SeqCst);
        if is_open {
            #[cfg(all(feature = "winit", feature = "wayland"))]
            if let Some(ref on_close) = on_surface_action {
                shell.publish(on_close(surface::action::destroy_popup(state.popup_id)));
            }
        }
    }

    if state.open_operation {
        state.open_operation = false;
        state.is_open.store(true, Ordering::SeqCst);
        if (refresh && is_open) || (!refresh && !is_open) {
            open(shell, state, on_selected.clone());
        }
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let is_open = state.is_open.load(Ordering::Relaxed);
            if is_open {
                // Event wasn't processed by overlay, so cursor was clicked either outside it's
                // bounds or on the drop-down, either way we close the overlay.
                state.is_open.store(false, Ordering::Relaxed);
                #[cfg(all(feature = "winit", feature = "wayland"))]
                if let Some(on_close) = on_surface_action {
                    shell.publish(on_close(surface::action::destroy_popup(state.popup_id)));
                }
                event::Status::Captured
            } else if cursor.is_over(layout.bounds()) {
                open(shell, state, on_selected);
                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { .. },
        }) => {
            let is_open = state.is_open.load(Ordering::Relaxed);

            if state.keyboard_modifiers.command() && cursor.is_over(layout.bounds()) && !is_open {
                let next_index = selected.map(|index| index + 1).unwrap_or_default();

                if selections.len() < next_index {
                    shell.publish((on_selected)(next_index));
                }

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            state.keyboard_modifiers = *modifiers;

            event::Status::Ignored
        }
        _ => event::Status::Ignored,
    }
}

/// Returns the current [`mouse::Interaction`] of a [`Dropdown`].
#[must_use]
pub fn mouse_interaction(layout: Layout<'_>, cursor: mouse::Cursor) -> mouse::Interaction {
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    if is_mouse_over {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

#[cfg(all(feature = "winit", feature = "wayland"))]
/// Returns the current menu widget of a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn menu_widget<
    S: AsRef<str> + Send + Sync + Clone + 'static,
    Message: 'static + std::clone::Clone,
>(
    bounds: Rectangle,
    state: &State,
    gap: f32,
    padding: Padding,
    text_size: f32,
    selections: Cow<'static, [S]>,
    icons: Cow<'static, [icon::Handle]>,
    selected_option: Option<usize>,
    on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync + 'static>,
    close_on_selected: Option<Message>,
) -> crate::Element<'static, Message>
where
    [S]: std::borrow::ToOwned,
{
    let icon_width = if icons.is_empty() { 0.0 } else { 24.0 };
    let measure = |_label: &str, selection_paragraph: &crate::Paragraph| -> f32 {
        selection_paragraph.min_width().round()
    };
    let selections_width = selections
        .iter()
        .zip(state.selections.iter())
        .map(|(label, selection)| measure(label.as_ref(), selection.raw()))
        .fold(0.0, |next, current| current.max(next));
    let pad_width = padding.horizontal().mul_add(2.0, 16.0);

    let width = selections_width + gap + pad_width + icon_width;
    let is_open = state.is_open.clone();
    let menu: Menu<'static, S, Message> = Menu::new(
        state.menu.clone(),
        selections,
        icons,
        state.hovered_option.clone(),
        selected_option,
        move |option| {
            is_open.store(false, Ordering::Relaxed);

            (on_selected)(option)
        },
        None,
        close_on_selected,
    )
    .width(width)
    .padding(padding)
    .text_size(text_size);

    crate::widget::autosize::autosize(
        menu.popup(iced::Point::new(0., 0.), bounds.height),
        AUTOSIZE_ID.clone(),
    )
    .auto_height(true)
    .auto_width(true)
    .min_height(1.)
    .min_width(width)
    .into()
}

/// Returns the current overlay of a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn overlay<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message: std::clone::Clone + 'a>(
    layout: Layout<'_>,
    _renderer: &crate::Renderer,
    state: &'a mut State,
    gap: f32,
    padding: Padding,
    text_size: f32,
    _text_line_height: text::LineHeight,
    _font: Option<crate::font::Font>,
    selections: &'a [S],
    icons: &'a [icon::Handle],
    selected_option: Option<usize>,
    on_selected: &'a dyn Fn(usize) -> Message,
    translation: Vector,
    close_on_selected: Option<Message>,
) -> Option<overlay::Element<'a, Message, crate::Theme, crate::Renderer>>
where
    [S]: std::borrow::ToOwned,
{
    if state.is_open.load(Ordering::Relaxed) {
        let bounds = layout.bounds();

        let menu = Menu::new(
            state.menu.clone(),
            Cow::Borrowed(selections),
            Cow::Borrowed(icons),
            state.hovered_option.clone(),
            selected_option,
            |option| {
                state.is_open.store(false, Ordering::Relaxed);

                (on_selected)(option)
            },
            None,
            close_on_selected,
        )
        .width({
            let measure = |_label: &str, selection_paragraph: &crate::Paragraph| -> f32 {
                selection_paragraph.min_width().round()
            };

            let pad_width = padding.horizontal().mul_add(2.0, 16.0);

            let icon_width = if icons.is_empty() { 0.0 } else { 24.0 };

            selections
                .iter()
                .zip(state.selections.iter_mut())
                .map(|(label, selection)| measure(label.as_ref(), selection.raw()))
                .fold(0.0, |next, current| current.max(next))
                + gap
                + pad_width
                + icon_width
        })
        .padding(padding)
        .text_size(text_size);

        let mut position = layout.position();
        position.x -= padding.left;
        position.x += translation.x;
        position.y += translation.y;
        Some(menu.overlay(position, bounds.height))
    } else {
        None
    }
}

/// Draws a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn draw<'a, S>(
    renderer: &mut crate::Renderer,
    theme: &crate::Theme,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    gap: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    font: crate::font::Font,
    selected: Option<&'a S>,
    icon: Option<&'a icon::Handle>,
    state: &'a State,
    viewport: &Rectangle,
) where
    S: AsRef<str> + 'a,
{
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    let style = if is_mouse_over {
        theme.style(&(), pick_list::Status::Hovered)
    } else {
        theme.style(&(), pick_list::Status::Active)
    };

    iced_core::Renderer::fill_quad(
        renderer,
        renderer::Quad {
            bounds,
            border: style.border,
            shadow: Shadow::default(),
        },
        style.background,
    );

    if let Some(handle) = state.icon.clone() {
        let svg_handle = svg::Svg::new(handle).color(style.text_color);

        svg::Renderer::draw_svg(
            renderer,
            svg_handle,
            Rectangle {
                x: bounds.x + bounds.width - gap - 16.0,
                y: bounds.center_y() - 8.0,
                width: 16.0,
                height: 16.0,
            },
        );
    }

    if let Some(content) = selected.map(AsRef::as_ref) {
        let text_size = text_size.unwrap_or_else(|| text::Renderer::default_size(renderer).0);

        let mut bounds = Rectangle {
            x: bounds.x + padding.left,
            y: bounds.center_y(),
            width: bounds.width - padding.horizontal(),
            height: f32::from(text_line_height.to_absolute(Pixels(text_size))),
        };

        if let Some(handle) = icon {
            let icon_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y - (bounds.height / 2.0) - 2.0,
                width: 20.0,
                height: 20.0,
            };

            bounds.x += 24.0;
            icon::draw(renderer, handle, icon_bounds);
        }

        text::Renderer::fill_text(
            renderer,
            Text {
                content: content.to_string(),
                size: iced::Pixels(text_size),
                line_height: text_line_height,
                font,
                bounds: bounds.size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            },
            bounds.position(),
            style.text_color,
            *viewport,
        );
    }
}
