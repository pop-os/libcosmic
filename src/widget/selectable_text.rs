use crate::Renderer;
use crate::widget::menu::MenuBarState;

use iced_core::event::Event;
use iced_core::text::LineHeight;
use iced_core::widget::Widget;
use iced_core::widget::text::{Alignment, Catalog, Ellipsize, Shaping, Style, Wrapping};
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Color, Layout, Length, Pixels, Rectangle, Shell, Size, Vector, alignment, mouse,
    overlay, renderer,
};
use std::borrow::Cow;

type InnerText<'a> = iced_core::widget::text::Text<'a, crate::Theme, Renderer>;

/// A selectable text widget with right-click context menu support.
///
/// Wraps the plain [`Text`](crate::widget::Text) widget to add text selection
/// and a context menu (Copy / Select All), both **on by default**. Created
/// with [`selectable_text`]; the plain [`text`](crate::widget::text)
/// constructor and the typography presets return the unwrapped
/// [`Text`](crate::widget::Text), so non-selectable text keeps iced's looser
/// trait bounds.
///
/// Use [`.selectable(false)`](Self::selectable) for plain, non-interactive
/// text, or [`.context_menu(false)`](Self::context_menu) to keep selection
/// without the menu.
pub struct SelectableText<'a> {
    inner: InnerText<'a>,
    selectable: bool,
    context_menu: bool,
}

impl<'a> SelectableText<'a> {
    /// Creates a new [`SelectableText`] widget with the given content.
    ///
    /// Selection and the context menu are enabled by default.
    pub fn new(content: impl iced_core::text::IntoFragment<'a>) -> Self {
        Self {
            inner: InnerText::new(content),
            selectable: true,
            context_menu: true,
        }
    }

    /// Controls whether the text is selectable. On by default.
    ///
    /// Pass `false` to render plain, non-interactive text (this also
    /// suppresses the context menu, which has nothing to act on).
    pub fn selectable(mut self, enabled: bool) -> Self {
        self.selectable = enabled;
        self
    }

    /// Controls whether the right-click context menu is shown. On by default.
    ///
    /// Pass `false` to keep text selection but disable the context menu.
    pub fn context_menu(mut self, enabled: bool) -> Self {
        self.context_menu = enabled;
        self
    }

    /// Whether the wrapper should install its context-menu machinery: only
    /// when the menu is enabled *and* the text is selectable (a menu over
    /// non-selectable text would have nothing to act on).
    fn shows_context_menu(&self) -> bool {
        self.selectable && self.context_menu
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

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.inner = self.inner.line_height(line_height);
        self
    }

    pub fn font(mut self, font: impl Into<<Renderer as iced_core::text::Renderer>::Font>) -> Self {
        self.inner = self.inner.font(font);
        self
    }

    pub fn font_maybe(
        mut self,
        font: Option<impl Into<<Renderer as iced_core::text::Renderer>::Font>>,
    ) -> Self {
        self.inner = self.inner.font_maybe(font);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }

    pub fn center(mut self) -> Self {
        self.inner = self.inner.center();
        self
    }

    pub fn align_x(mut self, alignment: impl Into<Alignment>) -> Self {
        self.inner = self.inner.align_x(alignment);
        self
    }

    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.inner = self.inner.align_y(alignment);
        self
    }

    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.inner = self.inner.shaping(shaping);
        self
    }

    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.inner = self.inner.wrapping(wrapping);
        self
    }

    pub fn ellipsize(mut self, ellipsize: Ellipsize) -> Self {
        self.inner = self.inner.ellipsize(ellipsize);
        self
    }

    #[must_use]
    pub fn style(mut self, style: impl Fn(&crate::Theme) -> Style + 'a) -> Self
    where
        <crate::Theme as Catalog>::Class<'a>:
            From<iced_core::widget::text::StyleFn<'a, crate::Theme>>,
    {
        self.inner = self.inner.style(style);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self
    where
        <crate::Theme as Catalog>::Class<'a>:
            From<iced_core::widget::text::StyleFn<'a, crate::Theme>>,
    {
        self.inner = self.inner.color(color);
        self
    }

    pub fn color_maybe(mut self, color: Option<impl Into<Color>>) -> Self
    where
        <crate::Theme as Catalog>::Class<'a>:
            From<iced_core::widget::text::StyleFn<'a, crate::Theme>>,
    {
        self.inner = self.inner.color_maybe(color);
        self
    }

    #[must_use]
    pub fn class(mut self, class: impl Into<<crate::Theme as Catalog>::Class<'a>>) -> Self {
        self.inner = self.inner.class(class);
        self
    }
}

impl<'a> From<InnerText<'a>> for SelectableText<'a> {
    fn from(inner: InnerText<'a>) -> Self {
        Self {
            inner,
            selectable: true,
            context_menu: true,
        }
    }
}

impl<'a> From<&'a str> for SelectableText<'a> {
    fn from(content: &'a str) -> Self {
        Self::new(content)
    }
}

struct TextWrapperState {
    menu_bar_state: MenuBarState,
    pending_action: crate::widget::text_context_menu::PendingAction,
}

fn w<'x, Message>(inner: &'x InnerText<'_>) -> &'x dyn Widget<Message, crate::Theme, Renderer> {
    inner
}

fn w_mut<'x, Message>(
    inner: &'x mut InnerText<'_>,
) -> &'x mut dyn Widget<Message, crate::Theme, Renderer> {
    inner
}

impl<'a, Message: Clone + 'static> Widget<Message, crate::Theme, Renderer> for SelectableText<'a> {
    fn tag(&self) -> tree::Tag {
        if self.shows_context_menu() {
            tree::Tag::of::<TextWrapperState>()
        } else {
            w::<Message>(&self.inner).tag()
        }
    }

    fn state(&self) -> tree::State {
        if self.shows_context_menu() {
            tree::State::new(TextWrapperState {
                menu_bar_state: MenuBarState::default(),
                pending_action: crate::widget::text_context_menu::pending_action(),
            })
        } else {
            w::<Message>(&self.inner).state()
        }
    }

    fn children(&self) -> Vec<Tree> {
        if self.shows_context_menu() {
            vec![Tree::new(w::<Message>(&self.inner))]
        } else {
            Vec::new()
        }
    }

    fn diff(&mut self, tree: &mut Tree) {
        if self.shows_context_menu() {
            if let Some(child) = tree.children.first_mut() {
                child.diff(&mut self.inner as &mut dyn Widget<Message, crate::Theme, Renderer>);
            }
        }
    }

    fn size(&self) -> Size<Length> {
        w::<Message>(&self.inner).size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let inner_tree = if self.shows_context_menu() {
            &mut tree.children[0]
        } else {
            tree
        };
        w_mut::<Message>(&mut self.inner).layout(inner_tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let inner_tree = if self.shows_context_menu() {
            &tree.children[0]
        } else {
            tree
        };
        w::<Message>(&self.inner).draw(
            inner_tree, renderer, theme, defaults, layout, cursor, viewport,
        );

        // draw a border when the text is focused
        use iced_core::renderer::Renderer as RendererExt;
        use iced_core::widget::text::State as TextState;

        let state = inner_tree
            .state
            .downcast_ref::<TextState<<Renderer as iced_core::text::Renderer>::Paragraph>>();
        if state.is_keyboard_focused() {
            let cosmic = theme.cosmic();
            let accent: Color = cosmic.accent.base.into();
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced_core::Border {
                        color: accent,
                        width: 1.0,
                        radius: cosmic.corner_radii.radius_s.into(),
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.shows_context_menu() {
            w_mut::<Message>(&mut self.inner).update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            #[cfg(feature = "wayland")]
            if self.uses_popup_context_menu() {
                use iced_core::widget::text::HasSelectableText;

                if self
                    .inner
                    .context_menu_position(&tree.children[0])
                    .is_some()
                {
                    let selected_text = self.inner.selected_text(&tree.children[0]);
                    let has_selection = selected_text.is_some();
                    let click_position =
                        self.inner.context_menu_position(&tree.children[0]).unwrap();
                    let wrapper_state = tree.state.downcast_ref::<TextWrapperState>();
                    let menu_bar_state = wrapper_state.menu_bar_state.clone();
                    let pending_action = wrapper_state.pending_action.clone();

                    crate::widget::text_context_menu::create_text_context_popup(
                        click_position,
                        selected_text,
                        false,
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

                {
                    let wrapper_state = tree.state.downcast_ref::<TextWrapperState>();
                    let pending_action = wrapper_state.pending_action.clone();
                    if let Some(action) =
                        crate::widget::text_context_menu::take_pending_action(&pending_action)
                    {
                        match action {
                            crate::widget::text_context_menu::TextCtxAction::Copy => {}
                            crate::widget::text_context_menu::TextCtxAction::SelectAll => {
                                self.inner.select_all(&mut tree.children[0]);
                            }
                            _ => {}
                        }
                    }
                }

                let wrapper_state = tree.state.downcast_ref::<TextWrapperState>();
                let menu_bar_state = wrapper_state.menu_bar_state.clone();
                crate::widget::text_context_menu::dismiss_popup_on_event(&menu_bar_state, event);
            }
        } else {
            w_mut::<Message>(&mut self.inner).update(
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
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let inner_tree = if self.shows_context_menu() {
            &tree.children[0]
        } else {
            tree
        };
        w::<Message>(&self.inner).mouse_interaction(inner_tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        if self.shows_context_menu() && !self.uses_popup_context_menu() {
            let menu_bar_state = tree
                .state
                .downcast_ref::<TextWrapperState>()
                .menu_bar_state
                .clone();
            let inner_tree = &mut tree.children[0];
            crate::widget::text_context_menu::context_menu_overlay(
                &self.inner,
                inner_tree,
                None,
                translation,
                menu_bar_state,
            )
        } else {
            None
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        let inner_tree = if self.shows_context_menu() {
            &mut tree.children[0]
        } else {
            tree
        };
        w_mut::<Message>(&mut self.inner).operate(inner_tree, layout, renderer, operation);
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        cursor: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let inner_state = if self.shows_context_menu() {
            &state.children[0]
        } else {
            state
        };
        w::<Message>(&self.inner).a11y_nodes(layout, inner_state, cursor)
    }

    fn id(&self) -> Option<iced_core::widget::Id> {
        w::<Message>(&self.inner).id()
    }

    fn set_id(&mut self, id: iced_core::widget::Id) {
        w_mut::<Message>(&mut self.inner).set_id(id);
    }
}

impl<'a, Message: Clone + 'static> From<SelectableText<'a>> for crate::Element<'a, Message> {
    fn from(text: SelectableText<'a>) -> Self {
        let SelectableText {
            inner,
            selectable,
            context_menu,
        } = text;
        // Apply selection to the inner widget here, at the last moment before
        // it becomes part of the tree. iced's `Text::selectable()` is one-way
        // (it can only set the flag), so deferring until conversion lets
        // `selectable(false)` work without ever having set it.
        let inner = if selectable {
            inner.selectable()
        } else {
            inner
        };
        Self::new(SelectableText {
            inner,
            selectable,
            context_menu,
        })
    }
}

/// Creates a [`SelectableText`] widget: text the user can select, with a
/// right-click context menu (Copy / Select All).
///
/// Use [`.context_menu(false)`](SelectableText::context_menu) to keep
/// selection but suppress the menu.
pub fn selectable_text<'a>(content: impl Into<Cow<'a, str>> + 'a) -> SelectableText<'a> {
    SelectableText::new(content.into()).font(crate::font::default())
}

// Typography presets, mirroring [`crate::widget::text`]
macro_rules! selectable_preset {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub fn $name<'a>(content: impl Into<Cow<'a, str>> + 'a) -> SelectableText<'a> {
            SelectableText::from(crate::widget::text::$name(content))
        }
    };
}

selectable_preset!(title1);
selectable_preset!(title2);
selectable_preset!(title3);
selectable_preset!(title4);
selectable_preset!(heading);
selectable_preset!(caption_heading);
selectable_preset!(body);
selectable_preset!(caption);
selectable_preset!(monotext);
