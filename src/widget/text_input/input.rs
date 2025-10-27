// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use std::borrow::Cow;
use std::cell::{Cell, LazyCell};

use crate::ext::ColorExt;
use crate::theme::THEME;

use super::cursor;
pub use super::cursor::Cursor;
use super::editor::Editor;
use super::style::StyleSheet;
pub use super::value::Value;

use apply::Apply;
use iced::Limits;
use iced::clipboard::dnd::{DndAction, DndEvent, OfferEvent, SourceEvent};
use iced::clipboard::mime::AsMimeTypes;
use iced_core::event::{self, Event};
use iced_core::mouse::{self, click};
use iced_core::overlay::Group;
use iced_core::renderer::{self, Renderer as CoreRenderer};
use iced_core::text::{self, Paragraph, Renderer, Text};
use iced_core::time::{Duration, Instant};
use iced_core::touch;
use iced_core::widget::Id;
use iced_core::widget::operation::{self, Operation};
use iced_core::widget::tree::{self, Tree};
use iced_core::window;
use iced_core::{Background, alignment};
use iced_core::{Border, Shadow, keyboard};
use iced_core::{
    Clipboard, Color, Element, Layout, Length, Padding, Pixels, Point, Rectangle, Shell, Size,
    Vector, Widget,
};
use iced_core::{layout, overlay};
use iced_runtime::{Action, Task, task};

thread_local! {
    // Prevents two inputs from being focused at the same time.
    static LAST_FOCUS_UPDATE: LazyCell<Cell<Instant>> = LazyCell::new(|| Cell::new(Instant::now()));
}

/// Creates a new [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn text_input<'a, Message>(
    placeholder: impl Into<Cow<'a, str>>,
    value: impl Into<Cow<'a, str>>,
) -> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    TextInput::new(placeholder, value)
}

/// A text label which can transform into a text input on activation.
pub fn editable_input<'a, Message: Clone + 'static>(
    placeholder: impl Into<Cow<'a, str>>,
    text: impl Into<Cow<'a, str>>,
    editing: bool,
    on_toggle_edit: impl Fn(bool) -> Message + 'a,
) -> TextInput<'a, Message> {
    let icon = crate::widget::icon::from_name(if editing {
        "edit-clear-symbolic"
    } else {
        "edit-symbolic"
    });

    TextInput::new(placeholder, text)
        .style(crate::theme::TextInput::EditableText)
        .editable()
        .editing(editing)
        .on_toggle_edit(on_toggle_edit)
        .trailing_icon(icon.size(16).into())
}

/// Creates a new search [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn search_input<'a, Message>(
    placeholder: impl Into<Cow<'a, str>>,
    value: impl Into<Cow<'a, str>>,
) -> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    let spacing = THEME.lock().unwrap().cosmic().space_xxs();

    TextInput::new(placeholder, value)
        .padding([0, spacing])
        .style(crate::theme::TextInput::Search)
        .leading_icon(
            crate::widget::icon::from_name("system-search-symbolic")
                .size(16)
                .apply(crate::widget::container)
                .padding(8)
                .into(),
        )
}
/// Creates a new secure [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn secure_input<'a, Message>(
    placeholder: impl Into<Cow<'a, str>>,
    value: impl Into<Cow<'a, str>>,
    on_visible_toggle: Option<Message>,
    hidden: bool,
) -> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    let spacing = THEME.lock().unwrap().cosmic().space_xxs();
    let mut input = TextInput::new(placeholder, value)
        .padding([0, spacing])
        .style(crate::theme::TextInput::Default)
        .leading_icon(
            crate::widget::icon::from_name("system-lock-screen-symbolic")
                .size(16)
                .apply(crate::widget::container)
                .padding(8)
                .into(),
        );
    if hidden {
        input = input.password();
    }
    if let Some(msg) = on_visible_toggle {
        input.trailing_icon(
            crate::widget::icon::from_name(if hidden {
                "document-properties-symbolic"
            } else {
                "image-red-eye-symbolic"
            })
            .size(16)
            .apply(crate::widget::button::custom)
            .class(crate::theme::Button::Icon)
            .on_press(msg)
            .padding(8)
            .into(),
        )
    } else {
        input
    }
}

/// Creates a new inline [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn inline_input<'a, Message>(
    placeholder: impl Into<Cow<'a, str>>,
    value: impl Into<Cow<'a, str>>,
) -> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    let spacing = THEME.lock().unwrap().cosmic().space_xxs();

    TextInput::new(placeholder, value)
        .style(crate::theme::TextInput::Inline)
        .padding(spacing)
}

pub(crate) const SUPPORTED_TEXT_MIME_TYPES: &[&str; 6] = &[
    "text/plain;charset=utf-8",
    "text/plain;charset=UTF-8",
    "UTF8_STRING",
    "STRING",
    "text/plain",
    "TEXT",
];

/// A field that can be filled with text.
#[allow(missing_debug_implementations)]
#[must_use]
pub struct TextInput<'a, Message> {
    id: Id,
    placeholder: Cow<'a, str>,
    value: Value,
    is_secure: bool,
    is_editable_variant: bool,
    is_read_only: bool,
    select_on_focus: bool,
    font: Option<<crate::Renderer as iced_core::text::Renderer>::Font>,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    helper_size: f32,
    label: Option<Cow<'a, str>>,
    helper_text: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    on_focus: Option<Message>,
    on_unfocus: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_tab: Option<Message>,
    on_submit: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_toggle_edit: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    leading_icon: Option<Element<'a, Message, crate::Theme, crate::Renderer>>,
    trailing_icon: Option<Element<'a, Message, crate::Theme, crate::Renderer>>,
    style: <crate::Theme as StyleSheet>::Style,
    on_create_dnd_source: Option<Box<dyn Fn(State) -> Message + 'a>>,
    surface_ids: Option<(window::Id, window::Id)>,
    dnd_icon: bool,
    line_height: text::LineHeight,
    helper_line_height: text::LineHeight,
    always_active: bool,
    /// The text input tracks and manages the input value in its state.
    manage_value: bool,
    drag_threshold: f32,
}

impl<'a, Message> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - a placeholder,
    /// - the current value
    pub fn new(placeholder: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        let spacing = THEME.lock().unwrap().cosmic().space_xxs();

        let v: Cow<'a, str> = value.into();
        TextInput {
            id: Id::unique(),
            placeholder: placeholder.into(),
            value: Value::new(v.as_ref()),
            is_secure: false,
            is_editable_variant: false,
            is_read_only: false,
            select_on_focus: false,
            font: None,
            width: Length::Fill,
            padding: spacing.into(),
            size: None,
            helper_size: 10.0,
            helper_line_height: text::LineHeight::Absolute(14.0.into()),
            on_focus: None,
            on_unfocus: None,
            on_input: None,
            on_paste: None,
            on_submit: None,
            on_tab: None,
            on_toggle_edit: None,
            leading_icon: None,
            trailing_icon: None,
            error: None,
            style: crate::theme::TextInput::default(),
            on_create_dnd_source: None,
            surface_ids: None,
            dnd_icon: false,
            line_height: text::LineHeight::default(),
            label: None,
            helper_text: None,
            always_active: false,
            manage_value: false,
            drag_threshold: 20.0,
        }
    }

    #[inline]
    fn dnd_id(&self) -> u128 {
        match &self.id.0 {
            iced_core::id::Internal::Custom(id, _) | iced_core::id::Internal::Unique(id) => {
                *id as u128
            }
            _ => unreachable!(),
        }
    }

    /// Sets the input to be always active.
    /// This makes it behave as if it was always focused.
    #[inline]
    pub const fn always_active(mut self) -> Self {
        self.always_active = true;
        self
    }

    /// Sets the text of the [`TextInput`].
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the helper text of the [`TextInput`].
    pub fn helper_text(mut self, helper_text: impl Into<Cow<'a, str>>) -> Self {
        self.helper_text = Some(helper_text.into());
        self
    }

    /// Sets the [`Id`] of the [`TextInput`].
    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the error message of the [`TextInput`].
    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`TextInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    #[inline]
    pub const fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Applies behaviors unique to the `editable_input` variable.
    #[inline]
    pub(crate) const fn editable(mut self) -> Self {
        self.is_editable_variant = true;
        self
    }

    #[inline]
    pub const fn editing(mut self, enable: bool) -> Self {
        self.is_read_only = !enable;
        self
    }

    /// Selects all text when the text input is focused
    #[inline]
    pub const fn select_on_focus(mut self, select_on_focus: bool) -> Self {
        self.select_on_focus = select_on_focus;
        self
    }

    /// Emits a message when an unfocused text input has been focused by click.
    ///
    /// This will not trigger if the input was focused externally by the application.
    #[inline]
    pub fn on_focus(mut self, on_focus: Message) -> Self {
        self.on_focus = Some(on_focus);
        self
    }

    /// Emits a message when a focused text input has been unfocused via the Tab or Esc key.
    ///
    /// This will not trigger if the input was unfocused externally by the application.
    #[inline]
    pub fn on_unfocus(mut self, on_unfocus: Message) -> Self {
        self.on_unfocus = Some(on_unfocus);
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input(mut self, callback: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(callback));
        self
    }

    /// Emits a message when a focused text input receives the Enter/Return key.
    pub fn on_submit(mut self, callback: impl Fn(String) -> Message + 'a) -> Self {
        self.on_submit = Some(Box::new(callback));
        self
    }

    /// Optionally emits a message when a focused text input receives the Enter/Return key.
    pub fn on_submit_maybe(self, callback: Option<impl Fn(String) -> Message + 'a>) -> Self {
        if let Some(callback) = callback {
            self.on_submit(callback)
        } else {
            self
        }
    }

    /// Emits a message when the Tab key has been captured, which prevents focus from changing.
    ///
    /// If you do no want to capture the Tab key, use [`TextInput::on_unfocus`] instead.
    #[inline]
    pub fn on_tab(mut self, on_tab: Message) -> Self {
        self.on_tab = Some(on_tab);
        self
    }

    /// Emits a message when the editable state of the input changes.
    pub fn on_toggle_edit(mut self, callback: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle_edit = Some(Box::new(callback));
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`].
    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    #[inline]
    pub const fn font(
        mut self,
        font: <crate::Renderer as iced_core::text::Renderer>::Font,
    ) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the start [`Icon`] of the [`TextInput`].
    #[inline]
    pub fn leading_icon(
        mut self,
        icon: Element<'a, Message, crate::Theme, crate::Renderer>,
    ) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Sets the end [`Icon`] of the [`TextInput`].
    #[inline]
    pub fn trailing_icon(
        mut self,
        icon: Element<'a, Message, crate::Theme, crate::Renderer>,
    ) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(mut self, style: impl Into<<crate::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the text input to manage its input value or not
    #[inline]
    pub const fn manage_value(mut self, manage_value: bool) -> Self {
        self.manage_value = manage_value;
        self
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        value: Option<&Value>,
        style: &renderer::Style,
    ) {
        let text_layout = self.text_layout(layout);
        draw(
            renderer,
            theme,
            layout,
            text_layout,
            cursor_position,
            tree,
            value.unwrap_or(&self.value),
            &self.placeholder,
            self.size,
            self.font,
            self.on_input.is_none(),
            self.is_secure,
            self.leading_icon.as_ref(),
            self.trailing_icon.as_ref(),
            &self.style,
            self.dnd_icon,
            self.line_height,
            self.error.as_deref(),
            self.label.as_deref(),
            self.helper_text.as_deref(),
            self.helper_size,
            self.helper_line_height,
            &layout.bounds(),
            style,
        );
    }

    /// Sets the start dnd handler of the [`TextInput`].
    #[cfg(feature = "wayland")]
    pub fn on_start_dnd(mut self, on_start_dnd: impl Fn(State) -> Message + 'a) -> Self {
        self.on_create_dnd_source = Some(Box::new(on_start_dnd));
        self
    }

    /// Sets the window id of the [`TextInput`] and the window id of the drag icon.
    /// Both ids are required to be unique.
    /// This is required for the dnd to work.
    #[inline]
    pub const fn surface_ids(mut self, window_id: (window::Id, window::Id)) -> Self {
        self.surface_ids = Some(window_id);
        self
    }

    /// Sets the mode of this [`TextInput`] to be a drag and drop icon.
    #[inline]
    pub const fn dnd_icon(mut self, dnd_icon: bool) -> Self {
        self.dnd_icon = dnd_icon;
        self
    }

    pub fn on_clear(self, on_clear: Message) -> Self {
        self.trailing_icon(
            crate::widget::icon::from_name("edit-clear-symbolic")
                .size(16)
                .apply(crate::widget::button::custom)
                .class(crate::theme::Button::Icon)
                .on_press(on_clear)
                .padding(8)
                .into(),
        )
    }

    /// Get the layout node of the actual text input
    fn text_layout<'b>(&'a self, layout: Layout<'b>) -> Layout<'b> {
        if self.dnd_icon {
            layout
        } else if self.label.is_some() {
            let mut nodes = layout.children();
            nodes.next();
            nodes.next().unwrap()
        } else {
            layout.children().next().unwrap()
        }
    }

    /// Set the drag threshold.
    pub fn drag_threshold(mut self, drag_threshold: f32) -> Self {
        self.drag_threshold = drag_threshold;
        self
    }
}

impl<Message> Widget<Message, crate::Theme, crate::Renderer> for TextInput<'_, Message>
where
    Message: Clone + 'static,
{
    #[inline]
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    #[inline]
    fn state(&self) -> tree::State {
        tree::State::new(State::new(
            self.is_secure,
            self.is_read_only,
            self.always_active,
            self.select_on_focus,
        ))
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        if !self.manage_value || !self.value.is_empty() && state.tracked_value != self.value {
            state.tracked_value = self.value.clone();
        } else if self.value.is_empty() {
            self.value = state.tracked_value.clone();
            // std::mem::swap(&mut state.tracked_value, &mut self.value);
        }
        // Unfocus text input if it becomes disabled
        if self.on_input.is_none() && !self.manage_value {
            state.last_click = None;
            state.is_focused = state.is_focused.map(|mut f| {
                f.focused = false;
                f
            });
            state.is_pasting = None;
            state.dragging_state = None;
        }
        let old_value = state
            .value
            .raw()
            .buffer()
            .lines
            .iter()
            .map(|l| l.text())
            .collect::<String>();
        if state.is_secure != self.is_secure
            || old_value != self.value.to_string()
            || state
                .label
                .raw()
                .buffer()
                .lines
                .iter()
                .map(|l| l.text())
                .collect::<String>()
                != self.label.as_deref().unwrap_or_default()
            || state
                .helper_text
                .raw()
                .buffer()
                .lines
                .iter()
                .map(|l| l.text())
                .collect::<String>()
                != self.helper_text.as_deref().unwrap_or_default()
        {
            state.is_secure = self.is_secure;
            state.dirty = true;
        }

        if self.always_active && !state.is_focused() {
            let now = Instant::now();
            LAST_FOCUS_UPDATE.with(|x| x.set(now));
            state.is_focused = Some(Focus {
                updated_at: now,
                now,
                focused: true,
                needs_update: false,
            });
        }

        // if the previous state was at the end of the text, keep it there
        let old_value = Value::new(&old_value);
        if state.is_focused() {
            if let cursor::State::Index(index) = state.cursor.state(&old_value) {
                if index == old_value.len() {
                    state.cursor.move_to(self.value.len());
                }
            }
        }

        if let Some(f) = state.is_focused.as_ref().filter(|f| f.focused) {
            if f.updated_at != LAST_FOCUS_UPDATE.with(|f| f.get()) {
                state.unfocus();
                state.emit_unfocus = true;
            }
        }

        self.is_read_only = state.is_read_only;

        // Stop pasting if input becomes disabled
        if !self.manage_value && self.on_input.is_none() {
            state.is_pasting = None;
        }

        let mut children: Vec<_> = self
            .leading_icon
            .iter_mut()
            .chain(self.trailing_icon.iter_mut())
            .map(iced_core::Element::as_widget_mut)
            .collect();
        tree.diff_children(children.as_mut_slice());
    }

    fn children(&self) -> Vec<Tree> {
        self.leading_icon
            .iter()
            .chain(self.trailing_icon.iter())
            .map(|icon| Tree::new(icon))
            .collect()
    }

    #[inline]
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        if self.dnd_icon {
            let state = tree.state.downcast_mut::<State>();
            let limits = limits.width(Length::Shrink).height(Length::Shrink);

            let size = self.size.unwrap_or_else(|| renderer.default_size().0);

            let bounds = limits.resolve(Length::Shrink, Length::Fill, Size::INFINITY);
            let value_paragraph = &mut state.value;
            let v = self.value.to_string();
            value_paragraph.update(Text {
                content: if self.value.is_empty() {
                    self.placeholder.as_ref()
                } else {
                    &v
                },
                font,
                bounds,
                size: iced::Pixels(size),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            });

            let Size { width, height } =
                limits.resolve(Length::Shrink, Length::Shrink, value_paragraph.min_bounds());

            let size = limits.resolve(width, height, Size::new(width, height));
            layout::Node::with_children(size, vec![layout::Node::new(size)])
        } else {
            let res = layout(
                renderer,
                limits,
                self.width,
                self.padding,
                self.size,
                self.leading_icon.as_ref(),
                self.trailing_icon.as_ref(),
                self.line_height,
                self.label.as_deref(),
                self.helper_text.as_deref(),
                self.helper_size,
                self.helper_line_height,
                font,
                tree,
            );

            // XXX not ideal, but we need to update the cache when is_secure changes
            let size = self.size.unwrap_or_else(|| renderer.default_size().0);
            let line_height = self.line_height;
            let state = tree.state.downcast_mut::<State>();
            if state.dirty {
                state.dirty = false;
                let value = if self.is_secure {
                    &self.value.secure()
                } else {
                    &self.value
                };
                replace_paragraph(
                    state,
                    Layout::new(&res),
                    value,
                    font,
                    iced::Pixels(size),
                    line_height,
                    limits,
                );
            }
            res
        }
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &crate::Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.custom(state, Some(&self.id));
        operation.focusable(state, Some(&self.id));
        operation.text_input(state, Some(&self.id));
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let mut layout_ = Vec::with_capacity(2);
        if self.leading_icon.is_some() {
            let mut children = self.text_layout(layout).children();
            children.next();
            layout_.push(children.next().unwrap());
        }
        if self.trailing_icon.is_some() {
            let mut children = self.text_layout(layout).children();
            children.next();
            if self.leading_icon.is_some() {
                children.next();
            }
            layout_.push(children.next().unwrap());
        };
        let children = self
            .leading_icon
            .iter_mut()
            .chain(self.trailing_icon.iter_mut())
            .zip(&mut tree.children)
            .zip(layout_)
            .filter_map(|((child, state), layout)| {
                child
                    .as_widget_mut()
                    .overlay(state, layout, renderer, translation)
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let text_layout = self.text_layout(layout);
        let mut trailing_icon_layout = None;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let size = self.size.unwrap_or_else(|| renderer.default_size().0);
        let line_height = self.line_height;

        // Disables editing of the editable variant when clicking outside of, or for tab focus changes.
        if self.is_editable_variant {
            if let Some(ref on_edit) = self.on_toggle_edit {
                let state = tree.state.downcast_mut::<State>();
                if !state.is_read_only && state.is_focused.is_some_and(|f| !f.focused) {
                    state.is_read_only = true;
                    shell.publish((on_edit)(false));
                } else if state.is_focused() && state.is_read_only {
                    state.is_read_only = false;
                    shell.publish((on_edit)(true));
                } else if let Some(f) = state.is_focused.as_mut().filter(|f| f.needs_update) {
                    // TODO do we want to just move this to on_focus or on_unfocus for all inputs?
                    f.needs_update = false;
                    state.is_read_only = true;
                    shell.publish((on_edit)(f.focused));
                }
            }
        }

        // Calculates the layout of the trailing icon button element.
        if !tree.children.is_empty() {
            let index = tree.children.len() - 1;
            if let (Some(trailing_icon), Some(tree)) =
                (self.trailing_icon.as_mut(), tree.children.get_mut(index))
            {
                trailing_icon_layout = Some(text_layout.children().last().unwrap());

                // Enable custom buttons defined on the trailing icon position to be handled.
                if !self.is_editable_variant {
                    if let Some(trailing_layout) = trailing_icon_layout {
                        let res = trailing_icon.as_widget_mut().on_event(
                            tree,
                            event.clone(),
                            trailing_layout,
                            cursor_position,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );

                        if res == event::Status::Captured {
                            return res;
                        }
                    }
                }
            }
        }

        let state = tree.state.downcast_mut::<State>();

        if let Some(on_unfocus) = self.on_unfocus.as_ref() {
            if state.emit_unfocus {
                state.emit_unfocus = false;
                shell.publish(on_unfocus.clone());
            }
        }

        let dnd_id = self.dnd_id();
        let id = Widget::id(self);
        update(
            id,
            event,
            text_layout.children().next().unwrap(),
            trailing_icon_layout,
            cursor_position,
            clipboard,
            shell,
            &mut self.value,
            size,
            font,
            self.is_editable_variant,
            self.is_secure,
            self.on_focus.as_ref(),
            self.on_unfocus.as_ref(),
            self.on_input.as_deref(),
            self.on_paste.as_deref(),
            self.on_submit.as_deref(),
            self.on_tab.as_ref(),
            self.on_toggle_edit.as_deref(),
            || tree.state.downcast_mut::<State>(),
            self.on_create_dnd_source.as_deref(),
            dnd_id,
            line_height,
            layout,
            self.manage_value,
            self.drag_threshold,
        )
    }

    #[inline]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let text_layout = self.text_layout(layout);
        draw(
            renderer,
            theme,
            layout,
            text_layout,
            cursor_position,
            tree,
            &self.value,
            &self.placeholder,
            self.size,
            self.font,
            self.on_input.is_none() && !self.manage_value,
            self.is_secure,
            self.leading_icon.as_ref(),
            self.trailing_icon.as_ref(),
            &self.style,
            self.dnd_icon,
            self.line_height,
            self.error.as_deref(),
            self.label.as_deref(),
            self.helper_text.as_deref(),
            self.helper_size,
            self.helper_line_height,
            viewport,
            style,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let layout = self.text_layout(layout);
        let mut index = 0;
        if let (Some(leading_icon), Some(tree)) =
            (self.leading_icon.as_ref(), state.children.get(index))
        {
            let leading_icon_layout = layout.children().nth(1).unwrap();

            if cursor_position.is_over(leading_icon_layout.bounds()) {
                return leading_icon.as_widget().mouse_interaction(
                    tree,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                );
            }
            index += 1;
        }

        if let (Some(trailing_icon), Some(tree)) =
            (self.trailing_icon.as_ref(), state.children.get(index))
        {
            let mut children = layout.children();
            children.next();
            // skip if there is no leading icon
            if self.leading_icon.is_some() {
                children.next();
            }
            let trailing_icon_layout = children.next().unwrap();

            if cursor_position.is_over(trailing_icon_layout.bounds()) {
                return trailing_icon.as_widget().mouse_interaction(
                    tree,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                );
            }
        }
        let mut children = layout.children();
        let layout = children.next().unwrap();
        mouse_interaction(
            layout,
            cursor_position,
            self.on_input.is_none() && !self.manage_value,
        )
    }

    #[inline]
    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    #[inline]
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn drag_destinations(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        _renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        if let Some(input) = layout.children().last() {
            let Rectangle {
                x,
                y,
                width,
                height,
            } = input.bounds();
            dnd_rectangles.push(iced::clipboard::dnd::DndDestinationRectangle {
                id: self.dnd_id(),
                rectangle: iced::clipboard::dnd::Rectangle {
                    x: x as f64,
                    y: y as f64,
                    width: width as f64,
                    height: height as f64,
                },
                mime_types: SUPPORTED_TEXT_MIME_TYPES
                    .iter()
                    .map(|s| Cow::Borrowed(*s))
                    .collect(),
                actions: DndAction::Move,
                preferred: DndAction::Move,
            });
        }
    }
}

impl<'a, Message> From<TextInput<'a, Message>>
    for Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: 'static + Clone,
{
    fn from(
        text_input: TextInput<'a, Message>,
    ) -> Element<'a, Message, crate::Theme, crate::Renderer> {
        Element::new(text_input)
    }
}

/// Produces a [`Task`] that focuses the [`TextInput`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::widget(operation::focusable::focus(id)))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_end(
        id,
    )))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_front(
        id,
    )))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<Message: 'static>(id: Id, position: usize) -> Task<Message> {
    task::effect(Action::widget(operation::text_input::move_cursor_to(
        id, position,
    )))
}

/// Produces a [`Task`] that selects all the content of the [`TextInput`] with the given [`Id`].
pub fn select_all<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::widget(operation::text_input::select_all(id)))
}

/// Computes the layout of a [`TextInput`].
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn layout<Message>(
    renderer: &crate::Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    leading_icon: Option<&Element<'_, Message, crate::Theme, crate::Renderer>>,
    trailing_icon: Option<&Element<'_, Message, crate::Theme, crate::Renderer>>,
    line_height: text::LineHeight,
    label: Option<&str>,
    helper_text: Option<&str>,
    helper_text_size: f32,
    helper_text_line_height: text::LineHeight,
    font: iced_core::Font,
    tree: &mut Tree,
) -> layout::Node {
    let limits = limits.width(width);
    let spacing = THEME.lock().unwrap().cosmic().space_xxs();
    let mut nodes = Vec::with_capacity(3);

    let text_pos = if let Some(label) = label {
        let text_bounds = limits.resolve(width, Length::Shrink, Size::INFINITY);
        let state = tree.state.downcast_mut::<State>();
        let label_paragraph = &mut state.label;
        label_paragraph.update(Text {
            content: label,
            font,
            bounds: text_bounds,
            size: iced::Pixels(size.unwrap_or_else(|| renderer.default_size().0)),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
            line_height,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        });
        let label_size = label_paragraph.min_bounds();

        nodes.push(layout::Node::new(label_size));
        Vector::new(0.0, label_size.height + f32::from(spacing))
    } else {
        Vector::ZERO
    };

    let text_size = size.unwrap_or_else(|| renderer.default_size().0);
    let mut text_input_height = line_height.to_absolute(text_size.into()).0;
    let padding = padding.fit(Size::ZERO, limits.max());

    let helper_pos = if leading_icon.is_some() || trailing_icon.is_some() {
        let children = &mut tree.children;
        // TODO configurable icon spacing, maybe via appearance
        let limits_copy = limits;

        let limits = limits.shrink(padding);
        let icon_spacing = 8.0;
        let mut c_i = 0;
        let (leading_icon_width, mut leading_icon) =
            if let Some((icon, tree)) = leading_icon.zip(children.get_mut(c_i)) {
                let size = icon.as_widget().size();
                let icon_node = icon.as_widget().layout(
                    tree,
                    renderer,
                    &Limits::NONE.width(size.width).height(size.height),
                );
                text_input_height = text_input_height.max(icon_node.bounds().height);
                c_i += 1;
                (icon_node.bounds().width + icon_spacing, Some(icon_node))
            } else {
                (0.0, None)
            };

        let (trailing_icon_width, mut trailing_icon) =
            if let Some((icon, tree)) = trailing_icon.zip(children.get_mut(c_i)) {
                let size = icon.as_widget().size();
                let icon_node = icon.as_widget().layout(
                    tree,
                    renderer,
                    &Limits::NONE.width(size.width).height(size.height),
                );
                text_input_height = text_input_height.max(icon_node.bounds().height);
                (icon_node.bounds().width + icon_spacing, Some(icon_node))
            } else {
                (0.0, None)
            };
        let text_limits = limits
            .width(width)
            .height(line_height.to_absolute(text_size.into()));
        let text_bounds = text_limits.resolve(Length::Shrink, Length::Shrink, Size::INFINITY);
        let text_node = layout::Node::new(
            text_bounds - Size::new(leading_icon_width + trailing_icon_width, 0.0),
        )
        .move_to(Point::new(
            padding.left + leading_icon_width,
            padding.top
                + ((text_input_height - line_height.to_absolute(text_size.into()).0) / 2.0)
                    .max(0.0),
        ));
        let mut node_list: Vec<_> = Vec::with_capacity(3);

        let text_node_bounds = text_node.bounds();
        node_list.push(text_node);

        if let Some(leading_icon) = leading_icon.take() {
            node_list.push(leading_icon.clone().move_to(Point::new(
                padding.left,
                padding.top + ((text_input_height - leading_icon.bounds().height) / 2.0).max(0.0),
            )));
        }
        if let Some(trailing_icon) = trailing_icon.take() {
            let trailing_icon = trailing_icon.clone().move_to(Point::new(
                text_node_bounds.x + text_node_bounds.width + f32::from(spacing),
                padding.top + ((text_input_height - trailing_icon.bounds().height) / 2.0).max(0.0),
            ));
            node_list.push(trailing_icon);
        }

        let text_input_size = Size::new(
            text_node_bounds.x + text_node_bounds.width + trailing_icon_width,
            text_input_height,
        )
        .expand(padding);

        let input_limits = limits_copy
            .width(width)
            .height(text_input_height.max(text_input_size.height))
            .min_width(text_input_size.width);
        let input_bounds = input_limits.resolve(
            width,
            text_input_height.max(text_input_size.height),
            text_input_size,
        );
        let input_node = layout::Node::with_children(input_bounds, node_list).translate(text_pos);
        let y_pos = input_node.bounds().y + input_node.bounds().height + f32::from(spacing);
        nodes.push(input_node);

        Vector::new(0.0, y_pos)
    } else {
        let limits = limits
            .width(width)
            .height(text_input_height + padding.vertical())
            .shrink(padding);
        let text_bounds = limits.resolve(Length::Shrink, Length::Shrink, Size::INFINITY);

        let text = layout::Node::new(text_bounds).move_to(Point::new(padding.left, padding.top));

        let node = layout::Node::with_children(text_bounds.expand(padding), vec![text])
            .translate(text_pos);
        let y_pos = node.bounds().y + node.bounds().height + f32::from(spacing);

        nodes.push(node);

        Vector::new(0.0, y_pos)
    };

    if let Some(helper_text) = helper_text {
        let limits = limits
            .width(width)
            .shrink(padding)
            .height(helper_text_line_height.to_absolute(helper_text_size.into()));
        let text_bounds = limits.resolve(width, Length::Shrink, Size::INFINITY);
        let state = tree.state.downcast_mut::<State>();
        let helper_text_paragraph = &mut state.helper_text;
        helper_text_paragraph.update(Text {
            content: helper_text,
            font,
            bounds: text_bounds,
            size: iced::Pixels(helper_text_size),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
            line_height: helper_text_line_height,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        });
        let helper_text_size = helper_text_paragraph.min_bounds();
        let helper_text_node = layout::Node::new(helper_text_size).translate(helper_pos);
        nodes.push(helper_text_node);
    };

    let mut size = nodes.iter().fold(Size::ZERO, |size, node| {
        Size::new(
            size.width.max(node.bounds().width),
            size.height + node.bounds().height,
        )
    });
    size.height += (nodes.len() - 1) as f32 * f32::from(spacing);

    let limits = limits
        .width(width)
        .height(size.height)
        .min_width(size.width);

    layout::Node::with_children(limits.resolve(width, size.height, size), nodes)
}

// TODO: Merge into widget method since iced has done the same.
/// Processes an [`Event`] and updates the [`State`] of a [`TextInput`]
/// accordingly.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
pub fn update<'a, Message: Clone + 'static>(
    id: Option<Id>,
    event: Event,
    text_layout: Layout<'_>,
    edit_button_layout: Option<Layout<'_>>,
    cursor: mouse::Cursor,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    value: &mut Value,
    size: f32,
    font: <crate::Renderer as iced_core::text::Renderer>::Font,
    is_editable_variant: bool,
    is_secure: bool,
    on_focus: Option<&Message>,
    on_unfocus: Option<&Message>,
    on_input: Option<&dyn Fn(String) -> Message>,
    on_paste: Option<&dyn Fn(String) -> Message>,
    on_submit: Option<&dyn Fn(String) -> Message>,
    on_tab: Option<&Message>,
    on_toggle_edit: Option<&dyn Fn(bool) -> Message>,
    state: impl FnOnce() -> &'a mut State,
    #[allow(unused_variables)] on_start_dnd_source: Option<&dyn Fn(State) -> Message>,
    #[allow(unused_variables)] dnd_id: u128,
    line_height: text::LineHeight,
    layout: Layout<'_>,
    manage_value: bool,
    drag_threshold: f32,
) -> event::Status {
    let update_cache = |state, value| {
        replace_paragraph(
            state,
            layout,
            value,
            font,
            iced::Pixels(size),
            line_height,
            &Limits::NONE.max_width(text_layout.bounds().width),
        );
    };

    let mut secured_value = if is_secure {
        value.secure()
    } else {
        value.clone()
    };
    let unsecured_value = value;
    let value = &mut secured_value;

    // NOTE: Clicks must be captured to prevent mouse areas behind them handling the same clicks.

    /// Mark a branch as cold
    #[inline]
    #[cold]
    fn cold() {}

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            cold();
            let state = state();

            let click_position = if on_input.is_some() || manage_value {
                cursor.position_over(layout.bounds())
            } else {
                None
            };

            if let Some(cursor_position) = click_position {
                // Check if the edit button was clicked.
                if state.dragging_state.is_none()
                    && edit_button_layout.is_some_and(|l| cursor.is_over(l.bounds()))
                {
                    if is_editable_variant {
                        state.is_read_only = !state.is_read_only;
                        state.move_cursor_to_end();

                        if let Some(on_toggle_edit) = on_toggle_edit {
                            shell.publish(on_toggle_edit(!state.is_read_only));
                        }

                        let now = Instant::now();
                        LAST_FOCUS_UPDATE.with(|x| x.set(now));
                        state.is_focused = Some(Focus {
                            updated_at: now,
                            now,
                            focused: true,
                            needs_update: false,
                        });
                    }

                    return event::Status::Captured;
                }

                let target = cursor_position.x - text_layout.bounds().x;

                let click =
                    mouse::Click::new(cursor_position, mouse::Button::Left, state.last_click);

                match (
                    &state.dragging_state,
                    click.kind(),
                    state.cursor().state(value),
                ) {
                    #[cfg(feature = "wayland")]
                    (None, click::Kind::Single, cursor::State::Selection { start, end }) => {
                        let left = start.min(end);
                        let right = end.max(start);

                        let (left_position, _left_offset) = measure_cursor_and_scroll_offset(
                            state.value.raw(),
                            text_layout.bounds(),
                            left,
                        );

                        let (right_position, _right_offset) = measure_cursor_and_scroll_offset(
                            state.value.raw(),
                            text_layout.bounds(),
                            right,
                        );

                        let width = right_position - left_position;
                        let selection_bounds = Rectangle {
                            x: text_layout.bounds().x + left_position,
                            y: text_layout.bounds().y,
                            width,
                            height: text_layout.bounds().height,
                        };

                        if cursor.is_over(selection_bounds) && (on_input.is_some() || manage_value)
                        {
                            state.dragging_state = Some(DraggingState::PrepareDnd(cursor_position));
                            return event::Status::Captured;
                        }
                        // clear selection and place cursor at click position
                        update_cache(state, value);
                        state.setting_selection(value, text_layout.bounds(), target);
                        state.dragging_state = None;
                        return event::Status::Captured;
                    }
                    (None, click::Kind::Single, _) => {
                        state.setting_selection(value, text_layout.bounds(), target);
                    }
                    (None | Some(DraggingState::Selection), click::Kind::Double, _) => {
                        update_cache(state, value);

                        if is_secure {
                            state.cursor.select_all(value);
                        } else {
                            let position =
                                find_cursor_position(text_layout.bounds(), value, state, target)
                                    .unwrap_or(0);

                            state.cursor.select_range(
                                value.previous_start_of_word(position),
                                value.next_end_of_word(position),
                            );
                        }
                        state.dragging_state = Some(DraggingState::Selection);
                    }
                    (None | Some(DraggingState::Selection), click::Kind::Triple, _) => {
                        update_cache(state, value);
                        state.cursor.select_all(value);
                        state.dragging_state = Some(DraggingState::Selection);
                    }
                    _ => {
                        state.dragging_state = None;
                    }
                }

                // Focus on click of the text input, and ensure that the input is writable.
                if !state.is_focused()
                    && matches!(state.dragging_state, None | Some(DraggingState::Selection))
                {
                    if let Some(on_focus) = on_focus {
                        shell.publish(on_focus.clone());
                    }

                    if state.is_read_only {
                        state.is_read_only = false;
                        if let Some(on_toggle_edit) = on_toggle_edit {
                            let message = (on_toggle_edit)(true);
                            shell.publish(message);
                        }
                    }

                    let now = Instant::now();
                    LAST_FOCUS_UPDATE.with(|x| x.set(now));

                    state.is_focused = Some(Focus {
                        updated_at: now,
                        now,
                        focused: true,
                        needs_update: false,
                    });
                }

                state.last_click = Some(click);

                return event::Status::Captured;
            } else {
                state.unfocus();

                if let Some(on_unfocus) = on_unfocus {
                    shell.publish(on_unfocus.clone());
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
            cold();
            let state = state();
            #[cfg(feature = "wayland")]
            if matches!(state.dragging_state, Some(DraggingState::PrepareDnd(_))) {
                // clear selection and place cursor at click position
                update_cache(state, value);
                if let Some(position) = cursor.position_over(layout.bounds()) {
                    let target = position.x - text_layout.bounds().x;
                    state.setting_selection(value, text_layout.bounds(), target);
                }
            }
            state.dragging_state = None;

            return if cursor.is_over(layout.bounds()) {
                event::Status::Captured
            } else {
                event::Status::Ignored
            };
        }
        Event::Mouse(mouse::Event::CursorMoved { position })
        | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
            let state = state();

            if matches!(state.dragging_state, Some(DraggingState::Selection)) {
                let target = position.x - text_layout.bounds().x;

                update_cache(state, value);
                let position =
                    find_cursor_position(text_layout.bounds(), value, state, target).unwrap_or(0);

                state
                    .cursor
                    .select_range(state.cursor.start(value), position);

                return event::Status::Captured;
            }
            #[cfg(feature = "wayland")]
            if let Some(DraggingState::PrepareDnd(start_position)) = state.dragging_state {
                let distance = ((position.x - start_position.x).powi(2)
                    + (position.y - start_position.y).powi(2))
                .sqrt();

                if distance >= drag_threshold {
                    if is_secure {
                        return event::Status::Ignored;
                    }

                    let input_text = state.selected_text(&value.to_string()).unwrap_or_default();
                    state.dragging_state =
                        Some(DraggingState::Dnd(DndAction::empty(), input_text.clone()));
                    let mut editor = Editor::new(unsecured_value, &mut state.cursor);
                    editor.delete();

                    let contents = editor.contents();
                    let unsecured_value = Value::new(&contents);
                    state.tracked_value = unsecured_value.clone();
                    if let Some(on_input) = on_input {
                        let message = (on_input)(contents);
                        shell.publish(message);
                    }
                    if let Some(on_start_dnd) = on_start_dnd_source {
                        shell.publish(on_start_dnd(state.clone()));
                    }
                    let state_clone = state.clone();

                    iced_core::clipboard::start_dnd(
                        clipboard,
                        false,
                        id.map(iced_core::clipboard::DndSource::Widget),
                        Some(iced_core::clipboard::IconSurface::new(
                            Element::from(
                                TextInput::<'static, ()>::new("", input_text.clone())
                                    .dnd_icon(true),
                            ),
                            iced_core::widget::tree::State::new(state_clone),
                            Vector::ZERO,
                        )),
                        Box::new(TextInputString(input_text)),
                        DndAction::Move,
                    );

                    update_cache(state, &unsecured_value);
                } else {
                    state.dragging_state = Some(DraggingState::PrepareDnd(start_position));
                }

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            text,
            physical_key,
            modifiers,
            ..
        }) => {
            let state = state();
            state.keyboard_modifiers = modifiers;

            if let Some(focus) = state.is_focused.as_mut().filter(|f| f.focused) {
                if state.is_read_only || (!manage_value && on_input.is_none()) {
                    return event::Status::Ignored;
                };
                let modifiers = state.keyboard_modifiers;
                focus.updated_at = Instant::now();
                LAST_FOCUS_UPDATE.with(|x| x.set(focus.updated_at));

                // Check if Ctrl+A/C/V/X was pressed.
                if state.keyboard_modifiers == keyboard::Modifiers::COMMAND
                    || state.keyboard_modifiers
                        == keyboard::Modifiers::COMMAND | keyboard::Modifiers::CAPS_LOCK
                {
                    match key.as_ref() {
                        keyboard::Key::Character("c") | keyboard::Key::Character("C") => {
                            if !is_secure {
                                if let Some((start, end)) = state.cursor.selection(value) {
                                    clipboard.write(
                                        iced_core::clipboard::Kind::Standard,
                                        value.select(start, end).to_string(),
                                    );
                                }
                            }
                        }
                        // XXX if we want to allow cutting of secure text, we need to
                        // update the cache and decide which value to cut
                        keyboard::Key::Character("x") | keyboard::Key::Character("X") => {
                            if !is_secure {
                                if let Some((start, end)) = state.cursor.selection(value) {
                                    clipboard.write(
                                        iced_core::clipboard::Kind::Standard,
                                        value.select(start, end).to_string(),
                                    );
                                }

                                let mut editor = Editor::new(value, &mut state.cursor);
                                editor.delete();
                                let content = editor.contents();
                                state.tracked_value = Value::new(&content);
                                if let Some(on_input) = on_input {
                                    let message = (on_input)(content);
                                    shell.publish(message);
                                }
                            }
                        }
                        keyboard::Key::Character("v") | keyboard::Key::Character("V") => {
                            let content = if let Some(content) = state.is_pasting.take() {
                                content
                            } else {
                                let content: String = clipboard
                                    .read(iced_core::clipboard::Kind::Standard)
                                    .unwrap_or_default()
                                    .chars()
                                    .filter(|c| !c.is_control())
                                    .collect();

                                Value::new(&content)
                            };

                            let mut editor = Editor::new(unsecured_value, &mut state.cursor);

                            editor.paste(content.clone());

                            let contents = editor.contents();
                            let unsecured_value = Value::new(&contents);
                            state.tracked_value = unsecured_value.clone();

                            if let Some(on_input) = on_input {
                                let message = if let Some(paste) = &on_paste {
                                    (paste)(contents)
                                } else {
                                    (on_input)(contents)
                                };

                                shell.publish(message);
                            }

                            state.is_pasting = Some(content);

                            let value = if is_secure {
                                unsecured_value.secure()
                            } else {
                                unsecured_value
                            };

                            update_cache(state, &value);
                            return event::Status::Captured;
                        }

                        keyboard::Key::Character("a") | keyboard::Key::Character("A") => {
                            state.cursor.select_all(value);
                            return event::Status::Captured;
                        }

                        _ => {}
                    }
                }

                // Capture keyboard inputs that should be submitted.
                if let Some(c) = text.and_then(|t| t.chars().next().filter(|c| !c.is_control())) {
                    if state.is_read_only || (!manage_value && on_input.is_none()) {
                        return event::Status::Ignored;
                    };

                    state.is_pasting = None;

                    if !state.keyboard_modifiers.command() && !modifiers.control() {
                        let mut editor = Editor::new(unsecured_value, &mut state.cursor);

                        editor.insert(c);

                        let contents = editor.contents();
                        let unsecured_value = Value::new(&contents);
                        state.tracked_value = unsecured_value.clone();

                        if let Some(on_input) = on_input {
                            let message = (on_input)(contents);
                            shell.publish(message);
                        }

                        focus.updated_at = Instant::now();
                        LAST_FOCUS_UPDATE.with(|x| x.set(focus.updated_at));

                        let value = if is_secure {
                            unsecured_value.secure()
                        } else {
                            unsecured_value
                        };

                        update_cache(state, &value);

                        return event::Status::Captured;
                    }
                }

                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if let Some(on_submit) = on_submit {
                            shell.publish((on_submit)(unsecured_value.to_string()));
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(value);
                                state.cursor.select_range(0, cursor_pos);
                            } else {
                                state.cursor.select_left_by_words(value);
                            }
                        }

                        let mut editor = Editor::new(unsecured_value, &mut state.cursor);
                        editor.backspace();

                        let contents = editor.contents();
                        let unsecured_value = Value::new(&contents);
                        state.tracked_value = unsecured_value.clone();
                        if let Some(on_input) = on_input {
                            let message = (on_input)(editor.contents());
                            shell.publish(message);
                        }
                        let value = if is_secure {
                            unsecured_value.secure()
                        } else {
                            unsecured_value
                        };
                        update_cache(state, &value);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(unsecured_value);
                                state.cursor.select_range(cursor_pos, unsecured_value.len());
                            } else {
                                state.cursor.select_right_by_words(unsecured_value);
                            }
                        }

                        let mut editor = Editor::new(unsecured_value, &mut state.cursor);
                        editor.delete();
                        let contents = editor.contents();
                        let unsecured_value = Value::new(&contents);
                        if let Some(on_input) = on_input {
                            let message = (on_input)(contents);
                            state.tracked_value = unsecured_value.clone();
                            shell.publish(message);
                        }

                        let value = if is_secure {
                            unsecured_value.secure()
                        } else {
                            unsecured_value
                        };

                        update_cache(state, &value);
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if platform::is_jump_modifier_pressed(modifiers) && !is_secure {
                            if modifiers.shift() {
                                state.cursor.select_left_by_words(value);
                            } else {
                                state.cursor.move_left_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_left(value);
                        } else {
                            state.cursor.move_left(value);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if platform::is_jump_modifier_pressed(modifiers) && !is_secure {
                            if modifiers.shift() {
                                state.cursor.select_right_by_words(value);
                            } else {
                                state.cursor.move_right_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_right(value);
                        } else {
                            state.cursor.move_right(value);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Home) => {
                        if modifiers.shift() {
                            state.cursor.select_range(state.cursor.start(value), 0);
                        } else {
                            state.cursor.move_to(0);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::End) => {
                        if modifiers.shift() {
                            state
                                .cursor
                                .select_range(state.cursor.start(value), value.len());
                        } else {
                            state.cursor.move_to(value.len());
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        state.unfocus();
                        state.is_read_only = true;

                        if let Some(on_unfocus) = on_unfocus {
                            shell.publish(on_unfocus.clone());
                        }
                    }

                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        if let Some(on_tab) = on_tab {
                            // Allow the application to decide how the event is handled.
                            // This could be to connect the text input to another text input.
                            // Or to connect the text input to a button.
                            shell.publish(on_tab.clone());
                        } else {
                            state.is_read_only = true;

                            if let Some(on_unfocus) = on_unfocus {
                                shell.publish(on_unfocus.clone());
                            }

                            return event::Status::Ignored;
                        };
                    }

                    keyboard::Key::Named(
                        keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowDown,
                    ) => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
            let state = state();

            if state.is_focused() {
                match key {
                    keyboard::Key::Character(c) if "v" == c => {
                        state.is_pasting = None;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab)
                    | keyboard::Key::Named(keyboard::key::Named::ArrowUp)
                    | keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            let state = state();

            state.keyboard_modifiers = modifiers;
        }
        Event::Window(window::Event::RedrawRequested(now)) => {
            let state = state();

            if let Some(focus) = state.is_focused.as_mut().filter(|f| f.focused) {
                focus.now = now;

                let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                    - (now - focus.updated_at).as_millis() % CURSOR_BLINK_INTERVAL_MILLIS;

                shell.request_redraw(window::RedrawRequest::At(
                    now + Duration::from_millis(u64::try_from(millis_until_redraw).unwrap()),
                ));
            }
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Source(SourceEvent::Finished | SourceEvent::Cancelled)) => {
            cold();
            let state = state();
            if matches!(state.dragging_state, Some(DraggingState::Dnd(..))) {
                // TODO: restore value in text input
                state.dragging_state = None;
                return event::Status::Captured;
            }
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(
            rectangle,
            OfferEvent::Enter {
                x,
                y,
                mime_types,
                surface,
            },
        )) if rectangle == Some(dnd_id) => {
            cold();
            let state = state();
            let is_clicked = text_layout.bounds().contains(Point {
                x: x as f32,
                y: y as f32,
            });

            let mut accepted = false;
            for m in &mime_types {
                if SUPPORTED_TEXT_MIME_TYPES.contains(&m.as_str()) {
                    let clone = m.clone();
                    accepted = true;
                }
            }
            if accepted {
                let target = x as f32 - text_layout.bounds().x;
                state.dnd_offer =
                    DndOfferState::HandlingOffer(mime_types.clone(), DndAction::empty());
                // existing logic for setting the selection
                let position = if target > 0.0 {
                    update_cache(state, value);
                    find_cursor_position(text_layout.bounds(), value, state, target)
                } else {
                    None
                };

                state.cursor.move_to(position.unwrap_or(0));
                return event::Status::Captured;
            }
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(rectangle, OfferEvent::Motion { x, y }))
            if rectangle == Some(dnd_id) =>
        {
            let state = state();

            let target = x as f32 - text_layout.bounds().x;
            // existing logic for setting the selection
            let position = if target > 0.0 {
                update_cache(state, value);
                find_cursor_position(text_layout.bounds(), value, state, target)
            } else {
                None
            };

            state.cursor.move_to(position.unwrap_or(0));
            return event::Status::Captured;
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(rectangle, OfferEvent::Drop)) if rectangle == Some(dnd_id) => {
            cold();
            let state = state();
            if let DndOfferState::HandlingOffer(mime_types, _action) = state.dnd_offer.clone() {
                let Some(mime_type) = SUPPORTED_TEXT_MIME_TYPES
                    .iter()
                    .find(|&&m| mime_types.iter().any(|t| t == m))
                else {
                    state.dnd_offer = DndOfferState::None;
                    return event::Status::Captured;
                };
                state.dnd_offer = DndOfferState::Dropped;
            }

            return event::Status::Ignored;
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(id, OfferEvent::LeaveDestination)) if Some(dnd_id) != id => {}
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(
            rectangle,
            OfferEvent::Leave | OfferEvent::LeaveDestination,
        )) => {
            cold();
            let state = state();
            // ASHLEY TODO we should be able to reset but for now we don't if we are handling a
            // drop
            match state.dnd_offer {
                DndOfferState::Dropped => {}
                _ => {
                    state.dnd_offer = DndOfferState::None;
                }
            };
            return event::Status::Captured;
        }
        #[cfg(feature = "wayland")]
        Event::Dnd(DndEvent::Offer(rectangle, OfferEvent::Data { data, mime_type }))
            if rectangle == Some(dnd_id) =>
        {
            cold();
            let state = state();
            if matches!(&state.dnd_offer, DndOfferState::Dropped) {
                state.dnd_offer = DndOfferState::None;
                if !SUPPORTED_TEXT_MIME_TYPES.contains(&mime_type.as_str()) || data.is_empty() {
                    return event::Status::Captured;
                }
                let Ok(content) = String::from_utf8(data) else {
                    return event::Status::Captured;
                };

                let mut editor = Editor::new(unsecured_value, &mut state.cursor);

                editor.paste(Value::new(content.as_str()));
                let contents = editor.contents();
                let unsecured_value = Value::new(&contents);
                state.tracked_value = unsecured_value.clone();
                if let Some(on_paste) = on_paste.as_ref() {
                    let message = (on_paste)(contents);
                    shell.publish(message);
                }

                let value = if is_secure {
                    unsecured_value.secure()
                } else {
                    unsecured_value
                };
                update_cache(state, &value);
                return event::Status::Captured;
            }
            return event::Status::Ignored;
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws the [`TextInput`] with the given [`Renderer`], overriding its
/// [`Value`] if provided.
///
/// [`Renderer`]: text::Renderer
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_panics_doc)]
pub fn draw<'a, Message>(
    renderer: &mut crate::Renderer,
    theme: &crate::Theme,
    layout: Layout<'_>,
    text_layout: Layout<'_>,
    cursor_position: mouse::Cursor,
    tree: &Tree,
    value: &Value,
    placeholder: &str,
    size: Option<f32>,
    font: Option<<crate::Renderer as iced_core::text::Renderer>::Font>,
    is_disabled: bool,
    is_secure: bool,
    icon: Option<&Element<'a, Message, crate::Theme, crate::Renderer>>,
    trailing_icon: Option<&Element<'a, Message, crate::Theme, crate::Renderer>>,
    style: &<crate::Theme as StyleSheet>::Style,
    dnd_icon: bool,
    line_height: text::LineHeight,
    error: Option<&str>,
    label: Option<&str>,
    helper_text: Option<&str>,
    helper_text_size: f32,
    helper_line_height: text::LineHeight,
    viewport: &Rectangle,
    renderer_style: &renderer::Style,
) {
    // all children should be icon images
    let children = &tree.children;

    let state = tree.state.downcast_ref::<State>();
    let secure_value = is_secure.then(|| value.secure());
    let value = secure_value.as_ref().unwrap_or(value);

    let mut children_layout = layout.children();

    let (label_layout, layout, helper_text_layout) = if label.is_some() && helper_text.is_some() {
        let label_layout = children_layout.next();
        let layout = children_layout.next().unwrap();
        let helper_text_layout = children_layout.next();
        (label_layout, layout, helper_text_layout)
    } else if label.is_some() {
        let label_layout = children_layout.next();
        let layout = children_layout.next().unwrap();
        (label_layout, layout, None)
    } else if helper_text.is_some() {
        let layout = children_layout.next().unwrap();
        let helper_text_layout = children_layout.next();
        (None, layout, helper_text_layout)
    } else {
        let layout = children_layout.next().unwrap();

        (None, layout, None)
    };

    let mut children_layout = layout.children();
    let bounds = layout.bounds();
    // XXX Dnd widget may not have a layout with children, so we just use the text_layout
    let text_bounds = children_layout.next().unwrap_or(text_layout).bounds();

    let is_mouse_over = cursor_position.is_over(bounds);

    let appearance = if is_disabled {
        theme.disabled(style)
    } else if error.is_some() {
        theme.error(style)
    } else if state.is_focused() {
        theme.focused(style)
    } else if is_mouse_over {
        theme.hovered(style)
    } else {
        theme.active(style)
    };

    let mut icon_color = appearance.icon_color.unwrap_or(renderer_style.icon_color);
    let mut text_color = appearance.text_color.unwrap_or(renderer_style.text_color);

    // TODO: iced will not render alpha itself on text or icon colors.
    if is_disabled {
        let background = theme.current_container().component.base.into();
        icon_color = icon_color.blend_alpha(background, 0.5);
        text_color = text_color.blend_alpha(background, 0.5);
    }

    // draw background and its border
    if let Some(border_offset) = appearance.border_offset {
        let offset_bounds = Rectangle {
            x: bounds.x - border_offset,
            y: bounds.y - border_offset,
            width: border_offset.mul_add(2.0, bounds.width),
            height: border_offset.mul_add(2.0, bounds.height),
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: appearance.border_radius,
                    width: appearance.border_width,
                    ..Default::default()
                },
                shadow: Shadow {
                    offset: Vector::new(0.0, 1.0),
                    color: Color::TRANSPARENT,
                    blur_radius: 0.0,
                },
            },
            appearance.background,
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: offset_bounds,
                border: Border {
                    width: appearance.border_width,
                    color: appearance.border_color,
                    radius: appearance.border_radius,
                },
                shadow: Shadow {
                    offset: Vector::new(0.0, 1.0),
                    color: Color::TRANSPARENT,
                    blur_radius: 0.0,
                },
            },
            Background::Color(Color::TRANSPARENT),
        );
    } else {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    width: appearance.border_width,
                    color: appearance.border_color,
                    radius: appearance.border_radius,
                },
                shadow: Shadow {
                    offset: Vector::new(0.0, 1.0),
                    color: Color::TRANSPARENT,
                    blur_radius: 0.0,
                },
            },
            appearance.background,
        );
    }

    // draw the label if it exists
    if let (Some(label_layout), Some(label)) = (label_layout, label) {
        renderer.fill_text(
            Text {
                content: label.to_string(),
                size: iced::Pixels(size.unwrap_or_else(|| renderer.default_size().0)),
                font: font.unwrap_or_else(|| renderer.default_font()),
                bounds: label_layout.bounds().size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                line_height,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
            label_layout.bounds().position(),
            appearance.label_color,
            *viewport,
        );
    }
    let mut child_index = 0;
    let leading_icon_tree = children.get(child_index);
    // draw the start icon in the text input
    let has_start_icon = icon.is_some();
    if let (Some(icon), Some(tree)) = (icon, leading_icon_tree) {
        let mut children = text_layout.children();
        let _ = children.next().unwrap();
        let icon_layout = children.next().unwrap();

        icon.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                icon_color,
                text_color,
                scale_factor: renderer_style.scale_factor,
            },
            icon_layout,
            cursor_position,
            viewport,
        );
        child_index += 1;
    }

    let text = value.to_string();
    let font = font.unwrap_or_else(|| renderer.default_font());
    let size = size.unwrap_or_else(|| renderer.default_size().0);
    let text_width = state.value.min_width();
    let actual_width = text_width.max(text_bounds.width);

    let radius_0 = THEME.lock().unwrap().cosmic().corner_radii.radius_0.into();
    #[cfg(feature = "wayland")]
    let handling_dnd_offer = !matches!(state.dnd_offer, DndOfferState::None);
    #[cfg(not(feature = "wayland"))]
    let handling_dnd_offer = false;
    let (cursor, offset) = if let Some(focus) =
        state.is_focused.filter(|f| f.focused).or_else(|| {
            let now = Instant::now();
            handling_dnd_offer.then_some(Focus {
                needs_update: false,
                updated_at: now,
                now,
                focused: true,
            })
        }) {
        match state.cursor.state(value) {
            cursor::State::Index(position) => {
                let (text_value_width, offset) =
                    measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, position);

                let is_cursor_visible = handling_dnd_offer
                    || ((focus.now - focus.updated_at).as_millis() / CURSOR_BLINK_INTERVAL_MILLIS)
                        % 2
                        == 0;
                if is_cursor_visible {
                    if dnd_icon {
                        (None, 0.0)
                    } else {
                        (
                            Some((
                                renderer::Quad {
                                    bounds: Rectangle {
                                        x: text_bounds.x + text_value_width - offset
                                            + if text_value_width < 0. {
                                                actual_width
                                            } else {
                                                0.
                                            },
                                        y: text_bounds.y,
                                        width: 1.0,
                                        height: text_bounds.height,
                                    },
                                    border: Border {
                                        width: 0.0,
                                        color: Color::TRANSPARENT,
                                        radius: radius_0,
                                    },
                                    shadow: Shadow {
                                        offset: Vector::ZERO,
                                        color: Color::TRANSPARENT,
                                        blur_radius: 0.0,
                                    },
                                },
                                text_color,
                            )),
                            offset,
                        )
                    }
                } else {
                    (None, offset)
                }
            }
            cursor::State::Selection { start, end } => {
                let left = start.min(end);
                let right = end.max(start);

                let value_paragraph = &state.value;
                let (left_position, left_offset) =
                    measure_cursor_and_scroll_offset(value_paragraph.raw(), text_bounds, left);

                let (right_position, right_offset) =
                    measure_cursor_and_scroll_offset(value_paragraph.raw(), text_bounds, right);

                let width = right_position - left_position;
                if dnd_icon {
                    (None, 0.0)
                } else {
                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x
                                        + left_position
                                        + if left_position < 0. || right_position < 0. {
                                            actual_width
                                        } else {
                                            0.
                                        },
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                border: Border {
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                    radius: radius_0,
                                },
                                shadow: Shadow {
                                    offset: Vector::ZERO,
                                    color: Color::TRANSPARENT,
                                    blur_radius: 0.0,
                                },
                            },
                            appearance.selected_fill,
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                    )
                }
            }
        }
    } else {
        (None, 0.0)
    };

    let render = |renderer: &mut crate::Renderer| {
        if let Some((cursor, color)) = cursor {
            renderer.fill_quad(cursor, color);
        } else {
            renderer.with_translation(Vector::ZERO, |_| {});
        }

        let bounds = Rectangle {
            x: text_bounds.x - offset,
            y: text_bounds.center_y(),
            width: actual_width,
            ..text_bounds
        };
        let color = if text.is_empty() {
            appearance.placeholder_color
        } else {
            text_color
        };

        renderer.fill_text(
            Text {
                content: if text.is_empty() {
                    placeholder.to_string()
                } else {
                    text.clone()
                },
                font,
                bounds: bounds.size(),
                size: iced::Pixels(size),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
            bounds.position(),
            color,
            *viewport,
        );
    };

    renderer.with_layer(text_bounds, render);

    let trailing_icon_tree = children.get(child_index);

    // draw the end icon in the text input
    if let (Some(icon), Some(tree)) = (trailing_icon, trailing_icon_tree) {
        let mut children = text_layout.children();
        let mut icon_layout = children.next().unwrap();
        if has_start_icon {
            icon_layout = children.next().unwrap();
        }
        icon_layout = children.next().unwrap();

        icon.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                icon_color,
                text_color,
                scale_factor: renderer_style.scale_factor,
            },
            icon_layout,
            cursor_position,
            viewport,
        );
    }

    // draw the helper text if it exists
    if let (Some(helper_text_layout), Some(helper_text)) = (helper_text_layout, helper_text) {
        renderer.fill_text(
            Text {
                content: helper_text.to_string(), // TODO remove to_string?
                size: iced::Pixels(helper_text_size),
                font,
                bounds: helper_text_layout.bounds().size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                line_height: helper_line_height,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
            helper_text_layout.bounds().position(),
            text_color,
            *viewport,
        );
    }
}

/// Computes the current [`mouse::Interaction`] of the [`TextInput`].
#[must_use]
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: mouse::Cursor,
    is_disabled: bool,
) -> mouse::Interaction {
    if cursor_position.is_over(layout.bounds()) {
        if is_disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Text
        }
    } else {
        mouse::Interaction::default()
    }
}

/// A string which can be sent to the clipboard or drag-and-dropped.
#[derive(Debug, Clone)]
pub struct TextInputString(pub String);

#[cfg(feature = "wayland")]
impl AsMimeTypes for TextInputString {
    fn available(&self) -> Cow<'static, [String]> {
        Cow::Owned(
            SUPPORTED_TEXT_MIME_TYPES
                .iter()
                .cloned()
                .map(String::from)
                .collect::<Vec<_>>(),
        )
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        if SUPPORTED_TEXT_MIME_TYPES.contains(&mime_type) {
            Some(Cow::Owned(self.0.clone().into_bytes()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DraggingState {
    Selection,
    #[cfg(feature = "wayland")]
    PrepareDnd(Point),
    #[cfg(feature = "wayland")]
    Dnd(DndAction, String),
}

#[cfg(feature = "wayland")]
#[derive(Debug, Default, Clone)]
pub(crate) enum DndOfferState {
    #[default]
    None,
    HandlingOffer(Vec<String>, DndAction),
    Dropped,
}
#[derive(Debug, Default, Clone)]
#[cfg(not(feature = "wayland"))]
pub(crate) struct DndOfferState;

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
#[must_use]
pub struct State {
    pub tracked_value: Value,
    pub value: crate::Plain,
    pub placeholder: crate::Plain,
    pub label: crate::Plain,
    pub helper_text: crate::Plain,
    pub dirty: bool,
    pub is_secure: bool,
    pub is_read_only: bool,
    pub emit_unfocus: bool,
    select_on_focus: bool,
    is_focused: Option<Focus>,
    dragging_state: Option<DraggingState>,
    dnd_offer: DndOfferState,
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    focused: bool,
    needs_update: bool,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new(
        is_secure: bool,
        is_read_only: bool,
        always_active: bool,
        select_on_focus: bool,
    ) -> Self {
        Self {
            is_secure,
            is_read_only,
            is_focused: always_active.then(|| {
                let now = Instant::now();
                Focus {
                    updated_at: now,
                    now,
                    focused: true,
                    needs_update: false,
                }
            }),
            select_on_focus,
            ..Self::default()
        }
    }

    /// Returns the current value of the selected text in the [`TextInput`].
    #[must_use]
    pub fn selected_text(&self, text: &str) -> Option<String> {
        let value = Value::new(text);
        match self.cursor.state(&value) {
            cursor::State::Index(_) => None,
            cursor::State::Selection { start, end } => {
                let left = start.min(end);
                let right = end.max(start);
                Some(text[left..right].to_string())
            }
        }
    }

    #[cfg(feature = "wayland")]
    /// Returns the current value of the dragged text in the [`TextInput`].
    #[must_use]
    pub fn dragged_text(&self) -> Option<String> {
        match self.dragging_state.as_ref() {
            Some(DraggingState::Dnd(_, text)) => Some(text.clone()),
            _ => None,
        }
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    pub fn focused(is_secure: bool, is_read_only: bool) -> Self {
        Self {
            tracked_value: Value::default(),
            is_secure,
            value: crate::Plain::default(),
            placeholder: crate::Plain::default(),
            label: crate::Plain::default(),
            helper_text: crate::Plain::default(),
            is_read_only,
            emit_unfocus: false,
            is_focused: None,
            select_on_focus: false,
            dragging_state: None,
            dnd_offer: DndOfferState::default(),
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            dirty: false,
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    #[inline]
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some_and(|f| f.focused)
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    #[cold]
    pub fn focus(&mut self) {
        let now = Instant::now();
        LAST_FOCUS_UPDATE.with(|x| x.set(now));
        let was_focused = self.is_focused.is_some_and(|f| f.focused);
        self.is_read_only = false;
        self.is_focused = Some(Focus {
            updated_at: now,
            now,
            focused: true,
            needs_update: false,
        });

        if was_focused {
            return;
        }
        if self.select_on_focus {
            self.select_all()
        } else {
            self.move_cursor_to_end();
        }
    }

    /// Unfocuses the [`TextInput`].
    #[cold]
    pub(super) fn unfocus(&mut self) {
        self.move_cursor_to_front();
        self.last_click = None;
        self.is_focused = self.is_focused.map(|mut f| {
            f.focused = false;
            f.needs_update = false;
            f
        });
        self.dragging_state = None;
        self.is_pasting = None;
        self.keyboard_modifiers = keyboard::Modifiers::default();
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    #[inline]
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    #[inline]
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    #[inline]
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`TextInput`].
    #[inline]
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }

    pub(super) fn setting_selection(&mut self, value: &Value, bounds: Rectangle<f32>, target: f32) {
        let position = if target > 0.0 {
            find_cursor_position(bounds, value, self, target)
        } else {
            None
        };

        self.cursor.move_to(position.unwrap_or(0));
        self.dragging_state = Some(DraggingState::Selection);
    }
}

impl operation::Focusable for State {
    #[inline]
    fn is_focused(&self) -> bool {
        Self::is_focused(self)
    }

    #[inline]
    fn focus(&mut self) {
        Self::focus(self);
        if let Some(focus) = self.is_focused.as_mut() {
            focus.needs_update = true;
        }
    }

    #[inline]
    fn unfocus(&mut self) {
        Self::unfocus(self);
        if let Some(focus) = self.is_focused.as_mut() {
            focus.needs_update = true;
        }
    }
}

impl operation::TextInput for State {
    #[inline]
    fn move_cursor_to_front(&mut self) {
        Self::move_cursor_to_front(self);
    }

    #[inline]
    fn move_cursor_to_end(&mut self) {
        Self::move_cursor_to_end(self);
    }

    #[inline]
    fn move_cursor_to(&mut self, position: usize) {
        Self::move_cursor_to(self, position);
    }

    #[inline]
    fn select_all(&mut self) {
        Self::select_all(self);
    }
}

#[inline(never)]
fn measure_cursor_and_scroll_offset(
    paragraph: &impl text::Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
#[inline(never)]
fn find_cursor_position(
    text_bounds: Rectangle,
    value: &Value,
    state: &State,
    x: f32,
) -> Option<usize> {
    let offset = offset(text_bounds, value, state);
    let value = value.to_string();

    let char_offset = state
        .value
        .raw()
        .hit_test(Point::new(x + offset, text_bounds.height / 2.0))
        .map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset.min(value.len())],
            true,
        )
        .count(),
    )
}

#[inline(never)]
fn replace_paragraph(
    state: &mut State,
    layout: Layout<'_>,
    value: &Value,
    font: <crate::Renderer as iced_core::text::Renderer>::Font,
    text_size: Pixels,
    line_height: text::LineHeight,
    limits: &layout::Limits,
) {
    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap();
    let bounds = limits.resolve(
        Length::Shrink,
        Length::Fill,
        Size::new(0., text_bounds.bounds().height),
    );

    state.value = crate::Plain::new(Text {
        font,
        line_height,
        content: &value.to_string(),
        bounds,
        size: text_size,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    });
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

mod platform {
    use iced_core::keyboard;

    #[inline]
    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}

#[inline(never)]
fn offset(text_bounds: Rectangle, value: &Value, state: &State) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) =
            measure_cursor_and_scroll_offset(state.value.raw(), text_bounds, focus_position);

        offset
    } else {
        0.0
    }
}
