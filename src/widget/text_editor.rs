// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A multi-line text editor

pub use iced_widget::text_editor::{
    Action, Binding, Catalog, Content, Cursor, Edit, Id, KeyPress, Line, LineEnding, Motion,
    Position, Selection, State, Status, Style, StyleFn,
};

use crate::widget::menu::MenuBarState;

use iced_core::event::Event;
use iced_core::text::highlighter;
use iced_core::widget::Widget;
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Layout, Length, Rectangle, Shell, Size, Vector, mouse, overlay, renderer,
};

type InnerEditor<'a, Message> =
    iced_widget::TextEditor<'a, highlighter::PlainText, Message, crate::Theme, crate::Renderer>;

pub struct TextEditor<'a, Message> {
    inner: InnerEditor<'a, Message>,
    has_context_menu: bool,
}

struct EditorWrapperState {
    menu_bar_state: MenuBarState,
    pending_action: crate::widget::text_context_menu::PendingAction,
}

impl<'a, Message: Clone + 'static> TextEditor<'a, Message> {
    /// Creates a new [`TextEditor`] from the given [`Content`].
    pub fn new(content: &'a Content<crate::Renderer>) -> Self {
        Self {
            inner: iced_widget::text_editor(content),
            has_context_menu: true,
        }
    }

    /// Controls whether the right-click context menu is shown.
    ///
    /// The context menu is enabled by default. Pass `false` to disable it.
    pub fn context_menu(mut self, enabled: bool) -> Self {
        self.has_context_menu = enabled;
        self
    }

    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.inner = self.inner.id(id);
        self
    }

    pub fn placeholder(mut self, placeholder: impl iced_core::text::IntoFragment<'a>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn width(mut self, width: impl Into<iced_core::Pixels>) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }

    pub fn min_height(mut self, min_height: impl Into<iced_core::Pixels>) -> Self {
        self.inner = self.inner.min_height(min_height);
        self
    }

    pub fn max_height(mut self, max_height: impl Into<iced_core::Pixels>) -> Self {
        self.inner = self.inner.max_height(max_height);
        self
    }

    pub fn on_action(mut self, on_edit: impl Fn(Action) -> Message + 'a) -> Self {
        self.inner = self.inner.on_action(on_edit);
        self
    }

    pub fn font(
        mut self,
        font: impl Into<<crate::Renderer as iced_core::text::Renderer>::Font>,
    ) -> Self {
        self.inner = self.inner.font(font);
        self
    }

    pub fn size(mut self, size: impl Into<iced_core::Pixels>) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<iced_core::text::LineHeight>) -> Self {
        self.inner = self.inner.line_height(line_height);
        self
    }

    pub fn padding(mut self, padding: impl Into<iced_core::Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }

    pub fn wrapping(mut self, wrapping: iced_core::text::Wrapping) -> Self {
        self.inner = self.inner.wrapping(wrapping);
        self
    }

    pub fn key_binding(
        mut self,
        key_binding: impl Fn(KeyPress) -> Option<Binding<Message>> + 'a,
    ) -> Self {
        self.inner = self.inner.key_binding(key_binding);
        self
    }

    #[must_use]
    pub fn style(mut self, style: impl Fn(&crate::Theme, Status) -> Style + 'a) -> Self
    where
        <crate::Theme as Catalog>::Class<'a>:
            From<iced_widget::text_editor::StyleFn<'a, crate::Theme>>,
    {
        self.inner = self.inner.style(style);
        self
    }

    fn uses_popup_context_menu(&self) -> bool {
        #[cfg(all(feature = "wayland", feature = "winit"))]
        if matches!(
            crate::app::cosmic::WINDOWING_SYSTEM.get(),
            Some(crate::app::cosmic::WindowingSystem::Wayland)
        ) {
            return true;
        }
        false
    }
}

/// Creates a new [`TextEditor`] from the given [`Content`].
pub fn text_editor<'a, Message: Clone + 'static>(
    content: &'a Content<crate::Renderer>,
) -> TextEditor<'a, Message> {
    TextEditor::new(content)
}

fn ew<'x, Message>(
    inner: &'x InnerEditor<'_, Message>,
) -> &'x dyn Widget<Message, crate::Theme, crate::Renderer> {
    inner
}

fn ew_mut<'x, Message>(
    inner: &'x mut InnerEditor<'_, Message>,
) -> &'x mut dyn Widget<Message, crate::Theme, crate::Renderer> {
    inner
}

impl<'a, Message: Clone + 'static> Widget<Message, crate::Theme, crate::Renderer>
    for TextEditor<'a, Message>
{
    fn tag(&self) -> tree::Tag {
        if self.has_context_menu {
            tree::Tag::of::<EditorWrapperState>()
        } else {
            ew::<Message>(&self.inner).tag()
        }
    }

    fn state(&self) -> tree::State {
        if self.has_context_menu {
            tree::State::new(EditorWrapperState {
                menu_bar_state: MenuBarState::default(),
                pending_action: crate::widget::text_context_menu::pending_action(),
            })
        } else {
            ew::<Message>(&self.inner).state()
        }
    }

    fn children(&self) -> Vec<Tree> {
        if self.has_context_menu {
            vec![Tree::new(ew::<Message>(&self.inner))]
        } else {
            ew::<Message>(&self.inner).children()
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        if self.has_context_menu {
            if let Some(child) = tree.children.first_mut() {
                child.diff(
                    &mut self.inner as &mut dyn Widget<Message, crate::Theme, crate::Renderer>,
                );
            }
        } else {
            ew_mut::<Message>(&mut self.inner).diff(tree);
        }
    }

    fn size(&self) -> Size<Length> {
        ew::<Message>(&self.inner).size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let inner_tree = if self.has_context_menu {
            &mut tree.children[0]
        } else {
            tree
        };
        ew_mut::<Message>(&mut self.inner).layout(inner_tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let inner_tree = if self.has_context_menu {
            &tree.children[0]
        } else {
            tree
        };
        ew::<Message>(&self.inner).draw(
            inner_tree, renderer, theme, defaults, layout, cursor, viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.has_context_menu {
            ew_mut::<Message>(&mut self.inner).update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            use iced_core::widget::text::HasSelectableText;
            if self.inner.is_focused(&tree.children[0])
                && matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(_))
                        | Event::Touch(iced_core::touch::Event::FingerPressed { .. })
                )
                && cursor.is_over(layout.bounds())
            {
                crate::widget::text_input::notify_focus_change();
            }

            #[cfg(feature = "wayland")]
            if self.uses_popup_context_menu() {
                if self
                    .inner
                    .context_menu_position(&tree.children[0])
                    .is_some()
                {
                    let selected_text = self.inner.selected_text(&tree.children[0]);
                    let has_selection = selected_text.is_some();
                    let click_position =
                        self.inner.context_menu_position(&tree.children[0]).unwrap();
                    let wrapper_state = tree.state.downcast_ref::<EditorWrapperState>();
                    let menu_bar_state = wrapper_state.menu_bar_state.clone();
                    let pending_action = wrapper_state.pending_action.clone();

                    crate::widget::text_context_menu::create_text_context_popup(
                        click_position,
                        selected_text,
                        self.inner.is_editable(),
                        has_selection,
                        &menu_bar_state,
                        &pending_action,
                        renderer,
                        viewport,
                        cursor,
                    );

                    self.inner
                        .set_context_menu_position(&mut tree.children[0], None);
                }

                // Process deferred actions from the popup.
                {
                    let wrapper_state = tree.state.downcast_ref::<EditorWrapperState>();
                    let pending_action = wrapper_state.pending_action.clone();
                    if let Some(action) =
                        crate::widget::text_context_menu::take_pending_action(&pending_action)
                    {
                        match action {
                            crate::widget::text_context_menu::TextCtxAction::Copy => {}
                            crate::widget::text_context_menu::TextCtxAction::Cut => {
                                self.inner.delete_selection(&mut tree.children[0]);
                            }
                            crate::widget::text_context_menu::TextCtxAction::Paste => {
                                let content: String = clipboard
                                    .read(iced_core::clipboard::Kind::Standard)
                                    .unwrap_or_default();
                                self.inner.paste_text(&mut tree.children[0], &content);
                            }
                            crate::widget::text_context_menu::TextCtxAction::SelectAll => {
                                self.inner.select_all(&mut tree.children[0]);
                            }
                        }
                    }
                }

                // Dismiss popup on outside click / Escape.
                let wrapper_state = tree.state.downcast_ref::<EditorWrapperState>();
                let menu_bar_state = wrapper_state.menu_bar_state.clone();
                crate::widget::text_context_menu::dismiss_popup_on_event(&menu_bar_state, event);
            }
        } else {
            ew_mut::<Message>(&mut self.inner).update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let inner_tree = if self.has_context_menu {
            &tree.children[0]
        } else {
            tree
        };
        ew::<Message>(&self.inner).mouse_interaction(inner_tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &crate::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        if self.has_context_menu {
            if !self.uses_popup_context_menu() {
                use iced_core::widget::text::HasSelectableText;
                let inner_tree = &mut tree.children[0];
                if self.inner.context_menu_position(inner_tree).is_some() {
                    let menu_bar_state = tree
                        .state
                        .downcast_ref::<EditorWrapperState>()
                        .menu_bar_state
                        .clone();
                    let inner_tree = &mut tree.children[0];
                    return crate::widget::text_context_menu::context_menu_overlay(
                        &self.inner,
                        inner_tree,
                        None,
                        translation,
                        menu_bar_state,
                    );
                }
            }

            ew_mut::<Message>(&mut self.inner).overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        } else {
            ew_mut::<Message>(&mut self.inner).overlay(
                tree,
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        let inner_tree = if self.has_context_menu {
            &mut tree.children[0]
        } else {
            tree
        };
        ew_mut::<Message>(&mut self.inner).operate(inner_tree, layout, renderer, operation);
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        cursor: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let inner_state = if self.has_context_menu {
            &state.children[0]
        } else {
            state
        };
        ew::<Message>(&self.inner).a11y_nodes(layout, inner_state, cursor)
    }

    fn id(&self) -> Option<iced_core::widget::Id> {
        ew::<Message>(&self.inner).id()
    }

    fn set_id(&mut self, id: iced_core::widget::Id) {
        ew_mut::<Message>(&mut self.inner).set_id(id);
    }
}

impl<'a, Message: Clone + 'static> From<TextEditor<'a, Message>> for crate::Element<'a, Message> {
    fn from(editor: TextEditor<'a, Message>) -> Self {
        Self::new(editor)
    }
}
