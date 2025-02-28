// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

use super::menu::{self, Menu};
use crate::widget::icon::{self, Handle};
use crate::Element;
use derive_setters::Setters;
use iced::window;
use iced_core::event::{self, Event};
use iced_core::text::{self, Paragraph, Text};
use iced_core::widget::tree::{self, Tree};
use iced_core::{alignment, keyboard, layout, mouse, overlay, renderer, svg, touch, Shadow};
use iced_core::{
    Clipboard, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector, Widget,
};
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
pub struct Dropdown<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message>
where
    [S]: std::borrow::ToOwned,
{
    #[setters(skip)]
    on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync>,
    #[setters(skip)]
    selections: &'a [S],
    #[setters]
    icons: &'a [icon::Handle],
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
    #[cfg(feature = "wayland")]
    #[setters(skip)]
    on_open: Option<
        Arc<
            dyn Fn(
                    iced_runtime::platform_specific::wayland::popup::SctkPopupSettings,
                    DropdownView<Message>,
                ) -> Message
                + 'a,
        >,
    >,
    #[setters(skip)]
    on_close_popup: Option<Box<dyn Fn(window::Id) -> Message + 'a>>,
    #[setters(strip_option)]
    window_id: Option<window::Id>,
    #[cfg(feature = "wayland")]
    positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
}

impl<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message: 'static> Dropdown<'a, S, Message>
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
        selections: &'a [S],
        selected: Option<usize>,
        on_selected: impl Fn(usize) -> Message + 'static + Send + Sync,
    ) -> Self {
        Self {
            on_selected: Arc::new(on_selected),
            selections,
            icons: &[],
            selected,
            width: Length::Shrink,
            gap: Self::DEFAULT_GAP,
            padding: Self::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::Relative(1.2),
            font: None,
            #[cfg(feature = "wayland")]
            on_open: None,
            window_id: None,
            #[cfg(feature = "wayland")]
            positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner::default(),
            on_close_popup: None,
        }
    }

    #[cfg(feature = "wayland")]
    /// Handle dropdown requests for popup creation.
    /// Intended to be used with [`crate::app::message::get_popup`]
    pub fn with_popup(
        mut self,
        parent_id: window::Id,
        on_open: impl Fn(
                iced_runtime::platform_specific::wayland::popup::SctkPopupSettings,
                DropdownView<Message>,
            ) -> Message
            + 'a,
    ) -> Self {
        self.window_id = Some(parent_id);
        self.on_open = Some(Arc::new(on_open));
        self
    }

    #[cfg(feature = "wayland")]
    pub fn with_positioner(
        mut self,
        positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
    ) -> Self {
        self.positioner = positioner;
        self
    }

    #[cfg(feature = "wayland")]
    /// Handle dropdown requests for popup removal.
    /// Intended to be used with [`crate::app::message::destroy_popup`]
    pub fn on_close_popup(mut self, on_close: impl Fn(window::Id) -> Message + 'a) -> Self {
        self.on_close_popup = Some(Box::new(on_close));
        self
    }
}

impl<S: AsRef<str> + Send + Sync + Clone + 'static, Message: 'static + std::clone::Clone>
    Widget<Message, crate::Theme, crate::Renderer> for Dropdown<'_, S, Message>
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
        update(
            &event,
            layout,
            cursor,
            shell,
            #[cfg(feature = "wayland")]
            self.on_open.clone(),
            #[cfg(feature = "wayland")]
            self.positioner.clone(),
            self.on_selected.clone(),
            self.selected,
            self.selections,
            || tree.state.downcast_mut::<State>(),
            self.window_id,
            self.on_close_popup.as_deref(),
            self.icons,
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        #[cfg(feature = "wayland")]
        return None;

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
            self.selections,
            self.icons,
            self.selected,
            self.on_selected.as_ref(),
            translation,
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

impl<'a, S: AsRef<str> + Send + Sync + Clone + 'static, Message: 'static + std::clone::Clone>
    From<Dropdown<'a, S, Message>> for crate::Element<'a, Message>
where
    [S]: std::borrow::ToOwned,
{
    fn from(pick_list: Dropdown<'a, S, Message>) -> Self {
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
                icon::Data::Name(named) => named
                    .path()
                    .filter(|path| path.extension().is_some_and(|ext| ext == OsStr::new("svg")))
                    .map(iced_core::svg::Handle::from_path),
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
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
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
#[allow(clippy::too_many_arguments)]
pub fn update<
    'a,
    S: AsRef<str> + Send + Sync + std::clone::Clone + 'static,
    Message: Clone + 'static,
>(
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    #[cfg(feature = "wayland")] on_open: Option<
        Arc<
            dyn Fn(
                    iced_runtime::platform_specific::wayland::popup::SctkPopupSettings,
                    DropdownView<Message>,
                ) -> Message
                + 'a,
        >,
    >,
    #[cfg(feature = "wayland")]
    positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
    on_selected: Arc<dyn Fn(usize) -> Message + Send + Sync + 'static>,
    selected: Option<usize>,
    selections: &[S],
    state: impl FnOnce() -> &'a mut State,
    window_id: Option<window::Id>,
    on_close: Option<&'a dyn Fn(window::Id) -> Message>,
    icons: &[icon::Handle],
    gap: f32,
    padding: Padding,
    text_size: Option<f32>,
    font: Option<crate::font::Font>,
    selected_option: Option<usize>,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();
            let is_open = state.is_open.load(Ordering::Relaxed);
            if is_open {
                // Event wasn't processed by overlay, so cursor was clicked either outside it's
                // bounds or on the drop-down, either way we close the overlay.
                state.is_open.store(false, Ordering::Relaxed);
                if let Some(on_close) = on_close {
                    shell.publish(on_close(state.popup_id));
                }
                event::Status::Captured
            } else if cursor.is_over(layout.bounds()) {
                state.is_open.store(true, Ordering::Relaxed);
                let mut hovered_guard = state.hovered_option.lock().unwrap();
                *hovered_guard = selected;
                let id = window::Id::unique();
                state.popup_id = id;
                #[cfg(feature = "wayland")]
                if let Some((on_open, parent)) = on_open.zip(window_id) {
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

                    shell.publish(on_open(
                        SctkPopupSettings {
                            parent,
                            id,
                            input_zone: None,
                            positioner: SctkPositioner {
                                size: Some((selections_width as u32 + gap as u32 + pad_width as u32 + icon_width as u32, 10)),
                                anchor_rect,
                                anchor: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Anchor::Top,
                                gravity:cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::Bottom,
                                reactive: true,
                                ..Default::default()
                            },
                            parent_size: None,
                            grab: true,
                            close_with_children: true,
                        },
                        Arc::new(move || {
                            Element::from(
                                menu_widget(
                                    bounds, &state, gap, padding,text_size.unwrap_or(14.0), selections.clone(), icons.clone(), selected_option, on_selected.clone()
                                )
                            )
                        }),
                    ));
                }
                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { .. },
        }) => {
            let state = state();
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
            let state = state();

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

#[cfg(feature = "wayland")]
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
