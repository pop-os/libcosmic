// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use std::borrow::Cow;

use crate::theme::THEME;

use super::cursor;
pub use super::cursor::Cursor;
use super::editor::Editor;
use super::style::StyleSheet;
pub use super::value::Value;

use apply::Apply;
use iced::Limits;
use iced_core::event::{self, Event};
use iced_core::keyboard;
use iced_core::mouse::{self, click};
use iced_core::overlay::Group;
use iced_core::renderer::{self, Renderer as CoreRenderer};
use iced_core::text::{self, Paragraph, Renderer, Text};
use iced_core::time::{Duration, Instant};
use iced_core::touch;
use iced_core::widget::operation::{self, Operation};
use iced_core::widget::tree::{self, Tree};
use iced_core::widget::Id;
use iced_core::window;
use iced_core::{alignment, Background};
use iced_core::{layout, overlay};
use iced_core::{
    Clipboard, Color, Element, Layout, Length, Padding, Pixels, Point, Rectangle, Shell, Size,
    Vector, Widget,
};
#[cfg(feature = "wayland")]
use iced_renderer::core::event::{wayland, PlatformSpecific};
use iced_renderer::core::widget::OperationOutputWrapper;
#[cfg(feature = "wayland")]
use iced_runtime::command::platform_specific;
use iced_runtime::Command;

#[cfg(feature = "wayland")]
use cctk::sctk::reexports::client::protocol::wl_data_device_manager::DndAction;
#[cfg(feature = "wayland")]
use iced_runtime::command::platform_specific::wayland::data_device::{DataFromMimeType, DndIcon};

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
    let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());

    TextInput::new(placeholder, value)
        .padding([0, spacing, 0, spacing])
        .style(crate::theme::TextInput::Search)
        .leading_icon(
            crate::widget::icon::from_name("system-search-symbolic")
                .size(16)
                .apply(crate::widget::container)
                .padding([spacing, spacing, spacing, spacing])
                .into(),
        )
}
/// Creates a new search [`TextInput`].
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
    let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());
    let mut input = TextInput::new(placeholder, value)
        .padding([0, spacing, 0, spacing])
        .style(crate::theme::TextInput::Default)
        .leading_icon(
            crate::widget::icon::from_name("system-lock-screen-symbolic")
                .size(16)
                .apply(crate::widget::container)
                .padding([spacing, spacing, spacing, spacing])
                .into(),
        );
    if hidden {
        input = input.password();
    }
    if let Some(msg) = on_visible_toggle {
        input.trailing_icon(
            crate::widget::icon::from_name("document-properties-symbolic")
                .size(16)
                .apply(crate::widget::button)
                .style(crate::theme::Button::Icon)
                .on_press(msg)
                .padding([spacing, spacing, spacing, spacing])
                .into(),
        )
    } else {
        input
    }
}

/// Creates a new inline [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn inline_input<'a, Message>(value: impl Into<Cow<'a, str>>) -> TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());

    TextInput::new("", value)
        .style(crate::theme::TextInput::Inline)
        .padding([spacing, spacing, spacing, spacing])
}

#[cfg(feature = "wayland")]
pub(crate) const SUPPORTED_TEXT_MIME_TYPES: &[&str; 6] = &[
    "text/plain;charset=utf-8",
    "text/plain;charset=UTF-8",
    "UTF8_STRING",
    "STRING",
    "text/plain",
    "TEXT",
];
#[cfg(feature = "wayland")]
pub type DnDCommand =
    Box<dyn Send + Sync + Fn() -> platform_specific::wayland::data_device::ActionInner>;
#[cfg(not(feature = "wayland"))]
pub type DnDCommand = ();

/// A field that can be filled with text.
///
/// # Example
/// ```no_run
/// # pub type TextInput<'a, Message> =
/// #     iced_widget::TextInput<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let value = "Some text";
///
/// let input = TextInput::new(
///     "This is the placeholder...",
///     value,
/// )
/// .on_input(Message::TextInputChanged)
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
#[must_use]
pub struct TextInput<'a, Message> {
    id: Option<Id>,
    placeholder: Cow<'a, str>,
    value: Value,
    is_secure: bool,
    font: Option<<crate::Renderer as iced_core::text::Renderer>::Font>,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    helper_size: f32,
    label: Option<&'a str>,
    helper_text: Option<&'a str>,
    error: Option<&'a str>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    leading_icon: Option<Element<'a, Message, crate::Renderer>>,
    trailing_icon: Option<Element<'a, Message, crate::Renderer>>,
    style: <<crate::Renderer as iced_core::Renderer>::Theme as StyleSheet>::Style,
    on_create_dnd_source: Option<Box<dyn Fn(State) -> Message + 'a>>,
    on_dnd_command_produced: Option<Box<dyn Fn(DnDCommand) -> Message + 'a>>,
    surface_ids: Option<(window::Id, window::Id)>,
    dnd_icon: bool,
    line_height: text::LineHeight,
    helper_line_height: text::LineHeight,
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
        let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());

        let v: Cow<'a, str> = value.into();
        TextInput {
            id: None,
            placeholder: placeholder.into(),
            value: Value::new(v.as_ref()),
            is_secure: false,
            font: None,
            width: Length::Fill,
            padding: [spacing, spacing, spacing, spacing].into(),
            size: None,
            helper_size: 10.0,
            helper_line_height: text::LineHeight::from(14.0),
            on_input: None,
            on_paste: None,
            on_submit: None,
            leading_icon: None,
            trailing_icon: None,
            error: None,
            style: crate::theme::TextInput::default(),
            on_dnd_command_produced: None,
            on_create_dnd_source: None,
            surface_ids: None,
            dnd_icon: false,
            line_height: text::LineHeight::default(),
            label: None,
            helper_text: None,
        }
    }

    /// Sets the text of the [`TextInput`].
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets the helper text of the [`TextInput`].
    pub fn helper_text(mut self, helper_text: &'a str) -> Self {
        self.helper_text = Some(helper_text);
        self
    }

    /// Sets the [`Id`] of the [`TextInput`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the error message of the [`TextInput`].
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self
    }

    /// Sets the [`LineHeight`] of the [`TextInput`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(String) -> Message,
    {
        self.on_input = Some(Box::new(callback));
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(self, message: Message) -> Self {
        self.on_submit_maybe(Some(message))
    }

    /// Maybe sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit_maybe(mut self, message: Option<Message>) -> Self {
        self.on_submit = message;
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
    pub fn font(mut self, font: <crate::Renderer as iced_core::text::Renderer>::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the start [`Icon`] of the [`TextInput`].
    pub fn leading_icon(mut self, icon: Element<'a, Message, crate::Renderer>) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Sets the end [`Icon`] of the [`TextInput`].
    pub fn trailing_icon(mut self, icon: Element<'a, Message, crate::Renderer>) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(
        mut self,
        style: impl Into<<<crate::Renderer as iced_core::Renderer>::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &<crate::Renderer as iced_core::Renderer>::Theme,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        value: Option<&Value>,
        style: &renderer::Style,
    ) {
        draw(
            renderer,
            theme,
            layout,
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
            self.error,
            self.label,
            self.helper_text,
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

    /// Sets the dnd command produced handler of the [`TextInput`].
    /// Commands should be returned in the update function of the application.
    #[cfg(feature = "wayland")]
    pub fn on_dnd_command_produced(
        mut self,
        on_dnd_command_produced: impl Fn(
                Box<dyn Send + Sync + Fn() -> platform_specific::wayland::data_device::ActionInner>,
            ) -> Message
            + 'a,
    ) -> Self {
        self.on_dnd_command_produced = Some(Box::new(on_dnd_command_produced));
        self
    }

    /// Sets the window id of the [`TextInput`] and the window id of the drag icon.
    /// Both ids are required to be unique.
    /// This is required for the dnd to work.
    pub fn surface_ids(mut self, window_id: (window::Id, window::Id)) -> Self {
        self.surface_ids = Some(window_id);
        self
    }

    /// Sets the mode of this [`TextInput`] to be a drag and drop icon.
    pub fn dnd_icon(mut self, dnd_icon: bool) -> Self {
        self.dnd_icon = dnd_icon;
        self
    }

    pub fn on_clear(self, on_clear: Message) -> Self {
        let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());

        self.trailing_icon(
            crate::widget::icon::from_name("edit-clear-symbolic")
                .size(16)
                .apply(crate::widget::button)
                .style(crate::theme::Button::Icon)
                .width(32)
                .height(32)
                .on_press(on_clear)
                .padding([spacing, spacing, spacing, spacing])
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
}

impl<'a, Message> Widget<Message, crate::Renderer> for TextInput<'a, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.is_secure))
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Unfocus text input if it becomes disabled
        if self.on_input.is_none() {
            state.last_click = None;
            state.is_focused = None;
            state.is_pasting = None;
            state.dragging_state = None;
        }

        if state.is_secure != self.is_secure {
            state.is_secure = self.is_secure;
            state.dirty = true;
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

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
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

            let bounds = limits.max();

            let value_paragraph = &mut state.value;
            let v = self.value.to_string();
            value_paragraph.update(Text {
                content: if self.value.is_empty() {
                    &self.placeholder
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
            });

            let Size { width, height } = limits.resolve(value_paragraph.min_bounds());

            let size = limits.resolve(Size::new(width, height));
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
                self.label,
                self.helper_text,
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
                    self.value.secure()
                } else {
                    self.value.clone()
                };
                replace_paragraph(
                    state,
                    Layout::new(&res),
                    &value,
                    font,
                    iced::Pixels(size),
                    line_height,
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
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.focusable(state, self.id.as_ref());
        operation.text_input(state, self.id.as_ref());
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
    ) -> Option<overlay::Element<'b, Message, crate::Renderer>> {
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
                child.as_widget_mut().overlay(state, layout, renderer)
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
        let mut index = 0;
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let size = self.size.unwrap_or_else(|| renderer.default_size().0);
        let line_height = self.line_height;

        if let (Some(leading_icon), Some(tree)) =
            (self.leading_icon.as_mut(), tree.children.get_mut(index))
        {
            let mut children = text_layout.children();
            children.next();
            let leading_icon_layout = children.next().unwrap();

            if cursor_position.is_over(leading_icon_layout.bounds())
                || matches!(
                    event,
                    Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft)
                )
            {
                let res = leading_icon.as_widget_mut().on_event(
                    tree,
                    event.clone(),
                    leading_icon_layout,
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
            index += 1;
        }
        if let (Some(trailing_icon), Some(tree)) =
            (self.trailing_icon.as_mut(), tree.children.get_mut(index))
        {
            let mut children = text_layout.children();
            children.next();
            if self.leading_icon.is_some() {
                children.next();
            }
            let trailing_icon_layout = children.next().unwrap();

            if cursor_position.is_over(trailing_icon_layout.bounds())
                | matches!(
                    event,
                    Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft)
                )
            {
                let res = trailing_icon.as_widget_mut().on_event(
                    tree,
                    event.clone(),
                    trailing_icon_layout,
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

        update(
            event,
            text_layout.children().next().unwrap(),
            cursor_position,
            clipboard,
            shell,
            &mut self.value,
            size,
            font,
            self.is_secure,
            self.on_input.as_deref(),
            self.on_paste.as_deref(),
            &self.on_submit,
            || tree.state.downcast_mut::<State>(),
            self.on_create_dnd_source.as_deref(),
            self.dnd_icon,
            self.on_dnd_command_produced.as_deref(),
            self.surface_ids,
            line_height,
            layout,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &<crate::Renderer as iced_core::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            tree,
            &self.value,
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
            self.error,
            self.label,
            self.helper_text,
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
            let mut children = layout.children();
            children.next();
            let leading_icon_layout = children.next().unwrap();

            if cursor_position.is_over(leading_icon_layout.bounds()) {
                return leading_icon.mouse_interaction(
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
                return trailing_icon.mouse_interaction(
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
        mouse_interaction(layout, cursor_position, self.on_input.is_none())
    }
}

impl<'a, Message> From<TextInput<'a, Message>> for Element<'a, Message, crate::Renderer>
where
    Message: 'static + Clone,
{
    fn from(text_input: TextInput<'a, Message>) -> Element<'a, Message, crate::Renderer> {
        Element::new(text_input)
    }
}

/// Produces a [`Command`] that focuses the [`TextInput`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_end(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_front(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<Message: 'static>(id: Id, position: usize) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to(id, position))
}

/// Produces a [`Command`] that selects all the content of the [`TextInput`] with the given [`Id`].
pub fn select_all<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::select_all(id))
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
    leading_icon: Option<&Element<'_, Message, crate::Renderer>>,
    trailing_icon: Option<&Element<'_, Message, crate::Renderer>>,
    line_height: text::LineHeight,
    label: Option<&str>,
    helper_text: Option<&str>,
    helper_text_size: f32,
    helper_text_line_height: text::LineHeight,
    font: iced_core::Font,
    tree: &mut Tree,
) -> layout::Node {
    let limits = limits.width(width);
    let spacing = THEME.with(|t| t.borrow().cosmic().space_xxs());
    let mut nodes = Vec::with_capacity(3);

    let text_pos = if let Some(label) = label {
        let text_bounds = limits.resolve(Size::ZERO);
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
        });
        let label_size = label_paragraph.min_bounds();

        nodes.push(layout::Node::new(label_size));
        Vector::new(0.0, label_size.height + f32::from(spacing))
    } else {
        Vector::ZERO
    };

    let text_size = size.unwrap_or_else(|| renderer.default_size().0);
    let mut text_input_height = text_size * 1.2;
    let padding = padding.fit(Size::ZERO, limits.max());

    let helper_pos = if leading_icon.is_some() || trailing_icon.is_some() {
        let children = &mut tree.children;
        // TODO configurable icon spacing, maybe via appearance
        let limits_copy = limits;

        let limits = limits.pad(padding);
        let icon_spacing = 8.0;
        let mut c_i = 0;
        let (leading_icon_width, mut leading_icon) =
            if let Some((icon, tree)) = leading_icon.zip(children.get_mut(c_i)) {
                let icon_node = icon.as_widget().layout(
                    tree,
                    renderer,
                    &Limits::NONE
                        .width(icon.as_widget().width())
                        .height(icon.as_widget().height()),
                );
                text_input_height = text_input_height.max(icon_node.bounds().height);
                c_i += 1;
                (icon_node.bounds().width + icon_spacing, Some(icon_node))
            } else {
                (0.0, None)
            };

        let (trailing_icon_width, mut trailing_icon) =
            if let Some((icon, tree)) = trailing_icon.zip(children.get_mut(c_i)) {
                let icon_node = icon.layout(
                    tree,
                    renderer,
                    &Limits::NONE
                        .width(icon.as_widget().width())
                        .height(icon.as_widget().height()),
                );
                text_input_height = text_input_height.max(icon_node.bounds().height);
                (icon_node.bounds().width + icon_spacing, Some(icon_node))
            } else {
                (0.0, None)
            };
        let text_limits = limits.width(width).height(text_size * 1.2);

        let text_bounds = text_limits.resolve(Size::ZERO);

        let mut text_node = layout::Node::new(
            text_bounds - Size::new(leading_icon_width + trailing_icon_width, 0.0),
        );

        text_node.move_to(Point::new(
            padding.left + leading_icon_width,
            padding.top + (text_size.mul_add(-1.2, text_input_height) / 2.0).max(0.0),
        ));
        let mut node_list: Vec<_> = Vec::with_capacity(3);

        let text_node_bounds = text_node.bounds();
        node_list.push(text_node);

        if let Some(mut leading_icon) = leading_icon.take() {
            leading_icon.move_to(Point::new(
                padding.left,
                padding.top + ((text_input_height - leading_icon.bounds().height) / 2.0).max(0.0),
            ));
            node_list.push(leading_icon);
        }
        if let Some(mut trailing_icon) = trailing_icon.take() {
            trailing_icon.move_to(Point::new(
                text_node_bounds.x + text_node_bounds.width + f32::from(spacing),
                padding.top + ((text_input_height - trailing_icon.bounds().height) / 2.0).max(0.0),
            ));
            node_list.push(trailing_icon);
        }

        let text_input_size = Size::new(
            text_node_bounds.x + text_node_bounds.width + trailing_icon_width,
            text_input_height,
        )
        .pad(padding);

        let input_limits = limits_copy
            .width(width)
            .height(text_input_height.max(text_input_size.height))
            .min_width(text_input_size.width);
        let input_bounds = input_limits.resolve(text_input_size);
        let input_node = layout::Node::with_children(input_bounds, node_list).translate(text_pos);
        let y_pos = input_node.bounds().y + input_node.bounds().height + f32::from(spacing);
        nodes.push(input_node);

        Vector::new(0.0, y_pos)
    } else {
        let limits = limits
            .width(width)
            .height(text_input_height + padding.vertical())
            .pad(padding);
        let text_bounds = limits.resolve(Size::ZERO);

        let mut text = layout::Node::new(text_bounds);
        text.move_to(Point::new(padding.left, padding.top));

        let node =
            layout::Node::with_children(text_bounds.pad(padding), vec![text]).translate(text_pos);
        let y_pos = node.bounds().y + node.bounds().height + f32::from(spacing);
        nodes.push(node);

        Vector::new(0.0, y_pos)
    };

    if let Some(helper_text) = helper_text {
        let limits = limits
            .width(width)
            .pad(padding)
            .height(helper_text_size * 1.2);
        let text_bounds = limits.resolve(Size::ZERO);

        let state = tree.state.downcast_mut::<State>();
        let helper_text_paragraph = &mut state.label;
        helper_text_paragraph.update(Text {
            content: helper_text,
            font,
            bounds: text_bounds,
            size: iced::Pixels(helper_text_size),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
            line_height: helper_text_line_height,
            shaping: text::Shaping::Advanced,
        });
        let helper_text_size = helper_text_paragraph.min_bounds();

        nodes.push(layout::Node::new(helper_text_size).translate(helper_pos));
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

    layout::Node::with_children(limits.resolve(size), nodes)
}

/// Processes an [`Event`] and updates the [`State`] of a [`TextInput`]
/// accordingly.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
pub fn update<'a, Message>(
    event: Event,
    text_layout: Layout<'_>,
    cursor_position: mouse::Cursor,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    value: &mut Value,
    size: f32,
    font: <crate::Renderer as iced_core::text::Renderer>::Font,
    is_secure: bool,
    on_input: Option<&dyn Fn(String) -> Message>,
    on_paste: Option<&dyn Fn(String) -> Message>,
    on_submit: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
    #[allow(unused_variables)] on_start_dnd_source: Option<&dyn Fn(State) -> Message>,
    #[allow(unused_variables)] dnd_icon: bool,
    #[allow(unused_variables)] on_dnd_command_produced: Option<&dyn Fn(DnDCommand) -> Message>,
    #[allow(unused_variables)] surface_ids: Option<(window::Id, window::Id)>,
    line_height: text::LineHeight,
    layout: Layout<'_>,
) -> event::Status
where
    Message: Clone,
{
    let update_cache = |state, value| {
        replace_paragraph(state, layout, value, font, iced::Pixels(size), line_height);
    };

    let mut secured_value = if is_secure {
        value.secure()
    } else {
        value.clone()
    };
    let unsecured_value = value;
    let value = &mut secured_value;

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();
            let is_clicked = cursor_position.is_over(text_layout.bounds()) && on_input.is_some();

            state.is_focused = if is_clicked {
                state.is_focused.or_else(|| {
                    let now = Instant::now();
                    Some(Focus {
                        updated_at: now,
                        now,
                    })
                })
            } else {
                None
            };

            if is_clicked {
                let Some(pos) = cursor_position.position() else {
                    return event::Status::Ignored;
                };
                let target = pos.x - text_layout.bounds().x;

                let click = mouse::Click::new(pos, state.last_click);

                match (
                    &state.dragging_state,
                    click.kind(),
                    state.cursor().state(value),
                ) {
                    #[cfg(feature = "wayland")]
                    (None, click::Kind::Single, cursor::State::Selection { start, end }) => {
                        // if something is already selected, we can start a drag and drop for a
                        // single click that is on top of the selected text
                        // is the click on selected text?

                        if let (
                            Some(on_start_dnd),
                            Some(on_dnd_command_produced),
                            Some((window_id, icon_id)),
                            Some(on_input),
                        ) = (
                            on_start_dnd_source,
                            on_dnd_command_produced,
                            surface_ids,
                            on_input,
                        ) {
                            let left = start.min(end);
                            let right = end.max(start);

                            let (left_position, _left_offset) = measure_cursor_and_scroll_offset(
                                &state.value,
                                text_layout.bounds(),
                                left,
                            );

                            let (right_position, _right_offset) = measure_cursor_and_scroll_offset(
                                &state.value,
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

                            if cursor_position.is_over(selection_bounds) {
                                // XXX never start a dnd if the input is secure
                                if is_secure {
                                    return event::Status::Ignored;
                                }
                                let text =
                                    state.selected_text(&value.to_string()).unwrap_or_default();
                                state.dragging_state =
                                    Some(DraggingState::Dnd(DndAction::empty(), text.clone()));
                                let mut editor = Editor::new(unsecured_value, &mut state.cursor);
                                editor.delete();

                                let contents = editor.contents();
                                let unsecured_value = Value::new(&contents);
                                let message = (on_input)(contents);
                                shell.publish(message);
                                shell.publish(on_start_dnd(state.clone()));
                                let state_clone = state.clone();
                                shell.publish(on_dnd_command_produced(Box::new(move || {
                                    platform_specific::wayland::data_device::ActionInner::StartDnd {
                                        mime_types: SUPPORTED_TEXT_MIME_TYPES
                                            .iter()
                                            .map(std::string::ToString::to_string)
                                            .collect(),
                                        actions: DndAction::Move,
                                        origin_id: window_id,
                                        icon_id: Some(DndIcon::Widget(
                                            icon_id,
                                            Box::new(state_clone.clone()),
                                        )),
                                        data: Box::new(TextInputString(text.clone())),
                                    }
                                })));

                                update_cache(state, &unsecured_value);
                            } else {
                                update_cache(state, value);
                                // existing logic for setting the selection
                                let position = if target > 0.0 {
                                    find_cursor_position(text_layout.bounds(), value, state, target)
                                } else {
                                    None
                                };

                                state.cursor.move_to(position.unwrap_or(0));
                                state.dragging_state = Some(DraggingState::Selection);
                            }
                        } else {
                            state.dragging_state = None;
                        }
                    }
                    (None, click::Kind::Single, _) => {
                        // existing logic for setting the selection
                        let position = if target > 0.0 {
                            update_cache(state, value);
                            find_cursor_position(text_layout.bounds(), value, state, target)
                        } else {
                            None
                        };

                        state.cursor.move_to(position.unwrap_or(0));
                        state.dragging_state = Some(DraggingState::Selection);
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

                state.last_click = Some(click);

                return event::Status::Captured;
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
            let state = state();
            state.dragging_state = None;
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
        }
        Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else {
                    return event::Status::Ignored;
                };

                if state.is_pasting.is_none()
                    && !state.keyboard_modifiers.command()
                    && !c.is_control()
                {
                    let mut editor = Editor::new(unsecured_value, &mut state.cursor);

                    editor.insert(c);
                    let contents = editor.contents();
                    let unsecured_value = Value::new(&contents);
                    let message = (on_input)(contents);
                    shell.publish(message);

                    focus.updated_at = Instant::now();

                    let value = if is_secure {
                        unsecured_value.secure()
                    } else {
                        unsecured_value
                    };
                    update_cache(state, &value);

                    return event::Status::Captured;
                }
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else {
                    return event::Status::Ignored;
                };

                let modifiers = state.keyboard_modifiers;
                focus.updated_at = Instant::now();

                match key_code {
                    keyboard::KeyCode::Enter | keyboard::KeyCode::NumpadEnter => {
                        if let Some(on_submit) = on_submit.clone() {
                            shell.publish(on_submit);
                        }
                    }
                    keyboard::KeyCode::Backspace => {
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
                        let message = (on_input)(editor.contents());
                        shell.publish(message);

                        let value = if is_secure {
                            unsecured_value.secure()
                        } else {
                            unsecured_value
                        };
                        update_cache(state, &value);
                    }
                    keyboard::KeyCode::Delete => {
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
                        let message = (on_input)(contents);
                        shell.publish(message);
                        let value = if is_secure {
                            unsecured_value.secure()
                        } else {
                            unsecured_value
                        };

                        update_cache(state, &value);
                    }
                    keyboard::KeyCode::Left => {
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
                    keyboard::KeyCode::Right => {
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
                    keyboard::KeyCode::Home => {
                        if modifiers.shift() {
                            state.cursor.select_range(state.cursor.start(value), 0);
                        } else {
                            state.cursor.move_to(0);
                        }
                    }
                    keyboard::KeyCode::End => {
                        if modifiers.shift() {
                            state
                                .cursor
                                .select_range(state.cursor.start(value), value.len());
                        } else {
                            state.cursor.move_to(value.len());
                        }
                    }
                    keyboard::KeyCode::C if state.keyboard_modifiers.command() => {
                        if !is_secure {
                            if let Some((start, end)) = state.cursor.selection(value) {
                                clipboard.write(value.select(start, end).to_string());
                            }
                        }
                    }
                    // XXX if we want to allow cutting of secure text, we need to
                    // update the cache and decide which value to cut
                    keyboard::KeyCode::X if state.keyboard_modifiers.command() => {
                        if !is_secure {
                            if let Some((start, end)) = state.cursor.selection(value) {
                                clipboard.write(value.select(start, end).to_string());
                            }

                            let mut editor = Editor::new(value, &mut state.cursor);
                            editor.delete();

                            let message = (on_input)(editor.contents());

                            shell.publish(message);
                        }
                    }
                    keyboard::KeyCode::V => {
                        if state.keyboard_modifiers.command() {
                            let content = if let Some(content) = state.is_pasting.take() {
                                content
                            } else {
                                let content: String = clipboard
                                    .read()
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
                            let message = if let Some(paste) = &on_paste {
                                (paste)(contents)
                            } else {
                                (on_input)(contents)
                            };
                            shell.publish(message);

                            state.is_pasting = Some(content);

                            let value = if is_secure {
                                unsecured_value.secure()
                            } else {
                                unsecured_value
                            };

                            update_cache(state, &value);
                        } else {
                            state.is_pasting = None;
                        }
                    }
                    keyboard::KeyCode::A if state.keyboard_modifiers.command() => {
                        state.cursor.select_all(value);
                    }
                    keyboard::KeyCode::Escape => {
                        state.is_focused = None;
                        state.dragging_state = None;
                        state.is_pasting = None;

                        state.keyboard_modifiers = keyboard::Modifiers::default();
                    }
                    keyboard::KeyCode::Tab | keyboard::KeyCode::Up | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
            let state = state();

            if state.is_focused.is_some() {
                match key_code {
                    keyboard::KeyCode::V => {
                        state.is_pasting = None;
                    }
                    keyboard::KeyCode::Tab | keyboard::KeyCode::Up | keyboard::KeyCode::Down => {
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
        Event::Window(_, window::Event::RedrawRequested(now)) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                focus.now = now;

                let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                    - (now - focus.updated_at).as_millis() % CURSOR_BLINK_INTERVAL_MILLIS;

                shell.request_redraw(window::RedrawRequest::At(
                    now + Duration::from_millis(u64::try_from(millis_until_redraw).unwrap()),
                ));
            }
        }
        #[cfg(feature = "wayland")]
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DataSource(
            wayland::DataSourceEvent::DndFinished | wayland::DataSourceEvent::Cancelled,
        ))) => {
            let state = state();
            if matches!(state.dragging_state, Some(DraggingState::Dnd(..))) {
                state.dragging_state = None;
                return event::Status::Captured;
            }
        }
        #[cfg(feature = "wayland")]
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DataSource(
            wayland::DataSourceEvent::DndActionAccepted(action),
        ))) => {
            let state = state();
            if let Some(DraggingState::Dnd(_, text)) = state.dragging_state.as_ref() {
                state.dragging_state = Some(DraggingState::Dnd(action, text.clone()));
                return event::Status::Captured;
            }
        }
        #[cfg(feature = "wayland")]
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::Enter { x, y, mime_types },
        ))) => {
            let Some(on_dnd_command_produced) = on_dnd_command_produced else {
                return event::Status::Ignored;
            };

            let state = state();
            let is_clicked = text_layout.bounds().contains(Point {
                x: x as f32,
                y: y as f32,
            });

            if !is_clicked {
                state.dnd_offer = DndOfferState::OutsideWidget(mime_types, DndAction::None);
                return event::Status::Captured;
            }
            let mut accepted = false;
            for m in &mime_types {
                if SUPPORTED_TEXT_MIME_TYPES.contains(&m.as_str()) {
                    let clone = m.clone();
                    accepted = true;
                    shell.publish(on_dnd_command_produced(Box::new(move || {
                        platform_specific::wayland::data_device::ActionInner::Accept(Some(
                            clone.clone(),
                        ))
                    })));
                }
            }
            if accepted {
                shell.publish(on_dnd_command_produced(Box::new(move || {
                    platform_specific::wayland::data_device::ActionInner::SetActions {
                        preferred: DndAction::Move,
                        accepted: DndAction::Move.union(DndAction::Copy),
                    }
                })));
                let target = x as f32 - text_layout.bounds().x;
                state.dnd_offer = DndOfferState::HandlingOffer(mime_types.clone(), DndAction::None);
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
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::Motion { x, y },
        ))) => {
            let Some(on_dnd_command_produced) = on_dnd_command_produced else {
                return event::Status::Ignored;
            };

            let state = state();
            let is_clicked = text_layout.bounds().contains(Point {
                x: x as f32,
                y: y as f32,
            });

            if !is_clicked {
                if let DndOfferState::HandlingOffer(mime_types, action) = state.dnd_offer.clone() {
                    state.dnd_offer = DndOfferState::OutsideWidget(mime_types, action);
                    shell.publish(on_dnd_command_produced(Box::new(move || {
                        platform_specific::wayland::data_device::ActionInner::SetActions {
                            preferred: DndAction::None,
                            accepted: DndAction::None,
                        }
                    })));
                    shell.publish(on_dnd_command_produced(Box::new(move || {
                        platform_specific::wayland::data_device::ActionInner::Accept(None)
                    })));
                }
                return event::Status::Captured;
            } else if let DndOfferState::OutsideWidget(mime_types, action) = state.dnd_offer.clone()
            {
                let mut accepted = false;
                for m in &mime_types {
                    if SUPPORTED_TEXT_MIME_TYPES.contains(&m.as_str()) {
                        accepted = true;
                        let clone = m.clone();
                        shell.publish(on_dnd_command_produced(Box::new(move || {
                            platform_specific::wayland::data_device::ActionInner::Accept(Some(
                                clone.clone(),
                            ))
                        })));
                    }
                }
                if accepted {
                    shell.publish(on_dnd_command_produced(Box::new(move || {
                        platform_specific::wayland::data_device::ActionInner::SetActions {
                            preferred: DndAction::Move,
                            accepted: DndAction::Move.union(DndAction::Copy),
                        }
                    })));
                    state.dnd_offer = DndOfferState::HandlingOffer(mime_types.clone(), action);
                }
            };
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
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::DropPerformed,
        ))) => {
            let Some(on_dnd_command_produced) = on_dnd_command_produced else {
                return event::Status::Ignored;
            };

            let state = state();
            if let DndOfferState::HandlingOffer(mime_types, _action) = state.dnd_offer.clone() {
                let Some(mime_type) = SUPPORTED_TEXT_MIME_TYPES
                    .iter()
                    .find(|m| mime_types.contains(&(**m).to_string()))
                else {
                    state.dnd_offer = DndOfferState::None;
                    return event::Status::Captured;
                };
                state.dnd_offer = DndOfferState::Dropped;
                shell.publish(on_dnd_command_produced(Box::new(move || {
                    platform_specific::wayland::data_device::ActionInner::RequestDndData(
                        (*mime_type).to_string(),
                    )
                })));
            } else if let DndOfferState::OutsideWidget(..) = &state.dnd_offer {
                state.dnd_offer = DndOfferState::None;
                return event::Status::Captured;
            }
            return event::Status::Ignored;
        }
        #[cfg(feature = "wayland")]
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::Leave,
        ))) => {
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
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::DndData { mime_type, data },
        ))) => {
            let Some(on_dnd_command_produced) = on_dnd_command_produced else {
                return event::Status::Ignored;
            };

            let state = state();
            if let DndOfferState::Dropped = state.dnd_offer.clone() {
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

                if let Some(on_paste) = on_paste.as_ref() {
                    let message = (on_paste)(contents);
                    shell.publish(message);
                }

                shell.publish(on_dnd_command_produced(Box::new(move || {
                    platform_specific::wayland::data_device::ActionInner::DndFinished
                })));
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
        #[cfg(feature = "wayland")]
        Event::PlatformSpecific(PlatformSpecific::Wayland(wayland::Event::DndOffer(
            wayland::DndOfferEvent::SourceActions(actions),
        ))) => {
            let Some(on_dnd_command_produced) = on_dnd_command_produced else {
                return event::Status::Ignored;
            };

            let state = state();
            if let DndOfferState::HandlingOffer(..) = state.dnd_offer.clone() {
                shell.publish(on_dnd_command_produced(Box::new(move || {
                    platform_specific::wayland::data_device::ActionInner::SetActions {
                        preferred: actions.intersection(DndAction::Move),
                        accepted: actions,
                    }
                })));
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
    cursor_position: mouse::Cursor,
    tree: &Tree,
    value: &Value,
    placeholder: &str,
    size: Option<f32>,
    font: Option<<crate::Renderer as iced_core::text::Renderer>::Font>,
    is_disabled: bool,
    is_secure: bool,
    icon: Option<&Element<'a, Message, crate::Renderer>>,
    trailing_icon: Option<&Element<'a, Message, crate::Renderer>>,
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
    let text_bounds = children_layout.next().unwrap().bounds();

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
                border_radius: appearance.border_radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            appearance.background,
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: offset_bounds,
                border_radius: appearance.border_radius,
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            Background::Color(Color::TRANSPARENT),
        );
    } else {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: appearance.border_radius,
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            appearance.background,
        );
    }

    // draw the label if it exists
    if let (Some(label_layout), Some(label)) = (label_layout, label) {
        renderer.fill_text(
            Text {
                content: label,
                size: iced::Pixels(size.unwrap_or_else(|| renderer.default_size().0)),
                font: font.unwrap_or_else(|| renderer.default_font()),
                bounds: label_layout.bounds().size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                line_height,
                shaping: text::Shaping::Advanced,
            },
            label_layout.bounds().position(),
            appearance.label_color,
            *viewport,
        );
    }
    let mut child_index = 0;
    let leading_icon_tree = children.get(child_index);
    // draw the start icon in the text input
    if let (Some(icon), Some(tree)) = (icon, leading_icon_tree) {
        let icon_layout = children_layout.next().unwrap();

        icon.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                icon_color: appearance.icon_color,
                text_color: appearance.text_color,
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

    let radius_0 = THEME
        .with(|t| t.borrow().cosmic().corner_radii.radius_0)
        .into();
    let (cursor, offset) = if let Some(focus) = &state.is_focused {
        match state.cursor.state(value) {
            cursor::State::Index(position) => {
                let (text_value_width, offset) =
                    measure_cursor_and_scroll_offset(&state.value, text_bounds, position);

                let is_cursor_visible =
                    ((focus.now - focus.updated_at).as_millis() / CURSOR_BLINK_INTERVAL_MILLIS) % 2
                        == 0;

                if is_cursor_visible {
                    if dnd_icon {
                        (None, 0.0)
                    } else {
                        (
                            Some((
                                renderer::Quad {
                                    bounds: Rectangle {
                                        x: text_bounds.x + text_value_width,
                                        y: text_bounds.y,
                                        width: 1.0,
                                        height: text_bounds.height,
                                    },
                                    border_radius: radius_0,
                                    border_width: 0.0,
                                    border_color: Color::TRANSPARENT,
                                },
                                appearance.text_color,
                            )),
                            offset,
                        )
                    }
                } else {
                    (None, 0.0)
                }
            }
            cursor::State::Selection { start, end } => {
                let left = start.min(end);
                let right = end.max(start);

                let value_paragraph = &state.value;
                let (left_position, left_offset) =
                    measure_cursor_and_scroll_offset(value_paragraph, text_bounds, left);

                let (right_position, right_offset) =
                    measure_cursor_and_scroll_offset(value_paragraph, text_bounds, right);

                let width = right_position - left_position;

                if dnd_icon {
                    (None, 0.0)
                } else {
                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                border_radius: radius_0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
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

    let text_width = state.value.min_width();

    let render = |renderer: &mut crate::Renderer| {
        if let Some((cursor, color)) = cursor {
            renderer.fill_quad(cursor, color);
        } else {
            renderer.with_translation(Vector::ZERO, |_| {});
        }

        let bounds = Rectangle {
            y: text_bounds.center_y(),
            width: f32::INFINITY,
            ..text_bounds
        };
        let color = if text.is_empty() {
            appearance.placeholder_color
        } else {
            appearance.text_color
        };
        renderer.fill_text(
            Text {
                content: if text.is_empty() { placeholder } else { &text },
                font,
                bounds: bounds.size(),
                size: iced::Pixels(size),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Advanced,
            },
            bounds.position(),
            color,
            *viewport,
        );
    };

    if text_width > text_bounds.width {
        renderer.with_layer(text_bounds, |renderer| {
            renderer.with_translation(Vector::new(-offset, 0.0), render);
        });
    } else {
        render(renderer);
    }

    let trailing_icon_tree = children.get(child_index);

    // draw the end icon in the text input
    if let (Some(icon), Some(tree)) = (trailing_icon, trailing_icon_tree) {
        let icon_layout = children_layout.next().unwrap();

        icon.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                icon_color: renderer_style.icon_color,
                text_color: appearance.text_color,
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
                content: helper_text,
                size: iced::Pixels(helper_text_size),
                font,
                bounds: helper_text_layout.bounds().size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                line_height: helper_line_height,
                shaping: text::Shaping::Advanced,
            },
            helper_text_layout.bounds().position(),
            appearance.text_color,
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
impl DataFromMimeType for TextInputString {
    fn from_mime_type(&self, mime_type: &str) -> Option<Vec<u8>> {
        SUPPORTED_TEXT_MIME_TYPES
            .contains(&mime_type)
            .then(|| self.0.as_bytes().to_vec())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DraggingState {
    Selection,
    #[cfg(feature = "wayland")]
    Dnd(DndAction, String),
}

#[cfg(feature = "wayland")]
#[derive(Debug, Default, Clone)]
pub(crate) enum DndOfferState {
    #[default]
    None,
    OutsideWidget(Vec<String>, DndAction),
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
    pub value: crate::Paragraph,
    pub placeholder: crate::Paragraph,
    pub label: crate::Paragraph,
    pub dirty: bool,
    pub is_secure: bool,
    is_focused: Option<Focus>,
    dragging_state: Option<DraggingState>,
    #[cfg(feature = "wayland")]
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
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new(is_secure: bool) -> Self {
        Self {
            is_secure,
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
    pub fn focused(is_secure: bool) -> Self {
        Self {
            is_secure,
            value: crate::Paragraph::new(),
            placeholder: crate::Paragraph::new(),
            label: crate::Paragraph::new(),

            is_focused: None,
            dragging_state: None,
            #[cfg(feature = "wayland")]
            dnd_offer: DndOfferState::default(),
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            dirty: false,
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
        });

        self.move_cursor_to_end();
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`TextInput`].
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        Self::is_focused(self)
    }

    fn focus(&mut self) {
        Self::focus(self);
    }

    fn unfocus(&mut self) {
        Self::unfocus(self);
    }
}

impl operation::TextInput for State {
    fn move_cursor_to_front(&mut self) {
        Self::move_cursor_to_front(self);
    }

    fn move_cursor_to_end(&mut self) {
        Self::move_cursor_to_end(self);
    }

    fn move_cursor_to(&mut self, position: usize) {
        Self::move_cursor_to(self, position);
    }

    fn select_all(&mut self) {
        Self::select_all(self);
    }
}

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

fn replace_paragraph(
    state: &mut State,
    layout: Layout<'_>,
    value: &Value,
    font: <crate::Renderer as iced_core::text::Renderer>::Font,
    text_size: Pixels,
    line_height: text::LineHeight,
) {
    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap().bounds();

    state.value = crate::Paragraph::with_text(Text {
        font,
        line_height,
        content: &value.to_string(),
        bounds: Size::new(f32::INFINITY, text_bounds.height),
        size: text_size,
        horizontal_alignment: alignment::Horizontal::Left,
        vertical_alignment: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
    });
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

mod platform {
    use iced_core::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}

fn offset(text_bounds: Rectangle, value: &Value, state: &State) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) =
            measure_cursor_and_scroll_offset(&state.value, text_bounds, focus_position);

        offset
    } else {
        0.0
    }
}
