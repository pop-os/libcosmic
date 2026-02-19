//! An expandable stack of cards
use std::time::Duration;

use self::iced_core::{
    Element, Event, Length, Size, Vector, Widget, border::Radius, id::Id, layout::Node,
    renderer::Quad, widget::Tree,
};
use crate::{
    anim,
    iced_core::{self, Border, Shadow},
    widget::{
        button,
        card::style::Style,
        column,
        icon::{self, Handle},
        row, text,
    },
};
use float_cmp::approx_eq;
use iced::widget;
use iced_core::{widget::tree, window};

const ICON_SIZE: u16 = 16;
const TOP_SPACING: u16 = 4;
const VERTICAL_SPACING: f32 = 8.0;
const PADDING: u16 = 16;
const BG_CARD_VISIBLE_HEIGHT: f32 = 4.0;
const BG_CARD_BORDER_RADIUS: f32 = 8.0;
const BG_CARD_MARGIN_STEP: f32 = 8.0;

/// get an expandable stack of cards
#[allow(clippy::too_many_arguments)]
pub fn cards<'a, Message, F, G>(
    id: widget::Id,
    card_inner_elements: Vec<Element<'a, Message, crate::Theme, crate::Renderer>>,
    on_clear_all: Message,
    on_show_more: Option<F>,
    on_activate: Option<G>,
    show_more_label: &'a str,
    show_less_label: &'a str,
    clear_all_label: &'a str,
    show_less_icon: Option<Handle>,
    expanded: bool,
) -> Cards<'a, Message, crate::Renderer>
where
    Message: 'static + Clone,
    F: 'a + Fn(bool) -> Message,
    G: 'a + Fn(usize) -> Message,
{
    Cards::new(
        id,
        card_inner_elements,
        on_clear_all,
        on_show_more,
        on_activate,
        show_more_label,
        show_less_label,
        clear_all_label,
        show_less_icon,
        expanded,
    )
}

impl<'a, Message, Renderer> Cards<'a, Message, Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    fn fully_expanded(&self, t: f32) -> bool {
        self.expanded && self.elements.len() > 1 && self.can_show_more && approx_eq!(f32, t, 1.0)
    }

    fn fully_unexpanded(&self, t: f32) -> bool {
        self.elements.len() == 1
            || (!self.expanded && (!self.can_show_more || approx_eq!(f32, t, 0.0)))
    }
}

/// An expandable stack of cards.
#[allow(missing_debug_implementations)]
pub struct Cards<'a, Message, Renderer = crate::Renderer>
where
    Renderer: iced_core::text::Renderer,
{
    id: Id,
    show_less_button: Element<'a, Message, crate::Theme, Renderer>,
    clear_all_button: Element<'a, Message, crate::Theme, Renderer>,
    elements: Vec<Element<'a, Message, crate::Theme, Renderer>>,
    expanded: bool,
    can_show_more: bool,
    width: Length,
    anim_multiplier: f32,
    duration: Duration,
}

impl<'a, Message> Cards<'a, Message, crate::Renderer>
where
    Message: Clone + 'static,
{
    /// Get an expandable stack of cards
    #[allow(clippy::too_many_arguments)]
    pub fn new<F, G>(
        id: widget::Id,
        card_inner_elements: Vec<Element<'a, Message, crate::Theme, crate::Renderer>>,
        on_clear_all: Message,
        on_show_more: Option<F>,
        on_activate: Option<G>,
        show_more_label: &'a str,
        show_less_label: &'a str,
        clear_all_label: &'a str,
        show_less_icon: Option<Handle>,
        expanded: bool,
    ) -> Self
    where
        F: 'a + Fn(bool) -> Message,
        G: 'a + Fn(usize) -> Message,
    {
        let can_show_more = card_inner_elements.len() > 1 && on_show_more.is_some();

        Self {
            can_show_more,
            id: Id::unique(),
            show_less_button: {
                let mut show_less_children = Vec::with_capacity(3);
                if let Some(source) = show_less_icon {
                    show_less_children.push(icon::icon(source).size(ICON_SIZE).into());
                }
                show_less_children.push(text::body(show_less_label).width(Length::Shrink).into());
                show_less_children.push(
                    icon::from_name("pan-up-symbolic")
                        .size(ICON_SIZE)
                        .icon()
                        .into(),
                );

                let button_content = row::with_children(show_less_children)
                    .align_y(iced_core::Alignment::Center)
                    .spacing(TOP_SPACING)
                    .width(Length::Shrink);

                Element::from(
                    button::custom(button_content)
                        .class(crate::theme::Button::Text)
                        .width(Length::Shrink)
                        .on_press_maybe(on_show_more.as_ref().map(|f| f(false)))
                        .padding([PADDING / 2, PADDING]),
                )
            },
            clear_all_button: Element::from(
                button::custom(text(clear_all_label))
                    .class(crate::theme::Button::Text)
                    .width(Length::Shrink)
                    .on_press(on_clear_all)
                    .padding([PADDING / 2, PADDING]),
            ),
            elements: card_inner_elements
                .into_iter()
                .enumerate()
                .map(|(i, w)| {
                    let custom_content = if i == 0 && !expanded && can_show_more {
                        column::with_capacity(2)
                            .push(w)
                            .push(text::caption(show_more_label))
                            .spacing(VERTICAL_SPACING)
                            .align_x(iced_core::Alignment::Center)
                            .into()
                    } else {
                        w
                    };

                    let b = crate::iced::widget::button(custom_content)
                        .class(crate::theme::iced::Button::Card)
                        .padding(PADDING);
                    if i == 0 && !expanded && can_show_more {
                        b.on_press_maybe(on_show_more.as_ref().map(|f| f(true)))
                    } else {
                        b.on_press_maybe(on_activate.as_ref().map(|f| f(i)))
                    }
                    .into()
                })
                // we will set the width of the container to shrink, then when laying out the top bar
                // we will set the fill limit to the max of the shrink top bar width and the max shrink width of the
                // cards
                .collect(),
            width: Length::Shrink,
            anim_multiplier: 1.0,
            expanded,
            duration: Duration::from_millis(200),
        }
    }

    ///  Set the width of the cards stack
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    /// The default animation time is 100ms, to speed up the toggle
    /// animation use a value less than 1.0, and to slow down the
    /// animation use a value greater than 1.0.
    pub fn anim_multiplier(mut self, multiplier: f32) -> Self {
        self.anim_multiplier = multiplier;
        self
    }

    pub fn duration(mut self, dur: Duration) -> Self {
        self.duration = dur;
        self
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, crate::Theme, Renderer> for Cards<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_core::Renderer + iced_core::text::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        [&self.show_less_button, &self.clear_all_button]
            .iter()
            .map(|w| Tree::new(w.as_widget()))
            .chain(self.elements.iter().map(|w| Tree::new(w.as_widget())))
            .collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        let mut children: Vec<_> = vec![
            self.show_less_button.as_widget_mut(),
            self.clear_all_button.as_widget_mut(),
        ]
        .into_iter()
        .chain(
            self.elements
                .iter_mut()
                .map(iced_core::Element::as_widget_mut),
        )
        .collect();

        tree.diff_children(children.as_mut_slice());
    }

    #[allow(clippy::too_many_lines)]
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let my_state = tree.state.downcast_ref::<State>();

        let mut children = Vec::with_capacity(1 + self.elements.len());
        let mut size = Size::new(0.0, 0.0);
        let tree_children = &mut tree.children;
        let count = self.elements.len();
        if self.elements.is_empty() {
            return Node::with_children(Size::new(1., 1.), children);
        }
        let s = anim::smootherstep(my_state.anim.t(self.duration, self.expanded));
        let fully_expanded: bool = self.fully_expanded(s);
        let fully_unexpanded: bool = self.fully_unexpanded(s);

        let show_less = &mut self.show_less_button;
        let clear_all = &mut self.clear_all_button;

        let show_less_node = if self.can_show_more {
            show_less
                .as_widget_mut()
                .layout(&mut tree_children[0], renderer, limits)
        } else {
            Node::new(Size::default())
        };
        let clear_all_node =
            clear_all
                .as_widget_mut()
                .layout(&mut tree_children[1], renderer, limits);
        size.width += show_less_node.size().width + clear_all_node.size().width;

        let custom_limits = limits.min_width(size.width);
        for (c, t) in self.elements.iter_mut().zip(tree_children[2..].iter_mut()) {
            let card_node = c.as_widget_mut().layout(t, renderer, &custom_limits);
            size.width = size.width.max(card_node.size().width);
        }

        if fully_expanded {
            let show_less = &mut self.show_less_button;
            let clear_all = &mut self.clear_all_button;

            let show_less_node = if self.can_show_more {
                show_less
                    .as_widget_mut()
                    .layout(&mut tree_children[0], renderer, limits)
            } else {
                Node::new(Size::default())
            };
            let clear_all_node = if self.can_show_more {
                let mut n =
                    clear_all
                        .as_widget_mut()
                        .layout(&mut tree_children[1], renderer, limits);
                let clear_all_node_size = n.size();
                n = clear_all_node
                    .translate(Vector::new(size.width - clear_all_node_size.width, 0.0));
                size.height += show_less_node.size().height.max(n.size().height) + VERTICAL_SPACING;
                n
            } else {
                Node::new(Size::default())
            };

            children.push(show_less_node);
            children.push(clear_all_node);
        }

        let custom_limits = limits
            .min_width(size.width)
            .max_width(size.width)
            .width(Length::Fixed(size.width));

        for (i, (c, t)) in self
            .elements
            .iter_mut()
            .zip(tree_children[2..].iter_mut())
            .enumerate()
        {
            let progress = s * size.height;
            let card_node = c
                .as_widget_mut()
                .layout(t, renderer, &custom_limits)
                .translate(Vector::new(0.0, progress));

            size.height = size.height.max(progress + card_node.size().height);

            children.push(card_node);

            if fully_unexpanded {
                let width = children.last().unwrap().bounds().width;

                // push the background card nodes
                for i in 1..self.elements.len().min(3) {
                    // height must be 16px for 8px padding
                    // but we only want 4px visible

                    let margin = f32::from(u8::try_from(i).unwrap()) * BG_CARD_MARGIN_STEP;
                    let node =
                        Node::new(Size::new(width - 2.0 * margin, BG_CARD_BORDER_RADIUS * 2.0))
                            .translate(Vector::new(
                                margin,
                                size.height - BG_CARD_BORDER_RADIUS * 2.0 + BG_CARD_VISIBLE_HEIGHT,
                            ));
                    size.height += BG_CARD_VISIBLE_HEIGHT;
                    children.push(node);
                }
                break;
            }

            if i + 1 < count {
                size.height += VERTICAL_SPACING;
            }
        }

        Node::with_children(size, children)
    }

    fn draw(
        &self,
        state: &iced_core::widget::Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        let my_state = state.state.downcast_ref::<State>();

        // there are 4 cases for drawing
        // 1. empty entries list
        //      Nothing to draw
        // 2. un-expanded
        //      go through the layout, draw the card, the inner card, and the bg cards
        // 3. expanding / unexpanding
        //      go through the layout. draw each card and its inner card
        // 4. expanded =>
        //      go through the layout. draw the top bar, and do all of 3
        // cards may be hovered
        // any buttons may have a hover state as well
        if self.elements.is_empty() {
            return;
        }

        let t = my_state.anim.t(self.duration, self.expanded);
        let fully_unexpanded = self.fully_unexpanded(t);
        let fully_expanded = self.fully_expanded(t);

        let mut layout = layout.children();
        let mut tree_children = state.children.iter();

        if fully_expanded {
            let show_less = &self.show_less_button;
            let clear_all = &self.clear_all_button;

            let show_less_layout = layout.next().unwrap();
            let clear_all_layout = layout.next().unwrap();

            show_less.as_widget().draw(
                tree_children.next().unwrap(),
                renderer,
                theme,
                style,
                show_less_layout,
                cursor,
                viewport,
            );

            clear_all.as_widget().draw(
                tree_children.next().unwrap(),
                renderer,
                theme,
                style,
                clear_all_layout,
                cursor,
                viewport,
            );
        } else {
            _ = tree_children.next();
            _ = tree_children.next();
        }

        // Draw first to appear behind
        if fully_unexpanded {
            let card_layout = layout.next().unwrap();
            let appearance = Style::default();
            let bg_layout = layout.collect::<Vec<_>>();
            for (i, layout) in (0..2).zip(bg_layout.into_iter()).rev() {
                renderer.fill_quad(
                    Quad {
                        bounds: layout.bounds(),
                        border: Border {
                            radius: Radius::from([
                                0.0,
                                0.0,
                                BG_CARD_BORDER_RADIUS,
                                BG_CARD_BORDER_RADIUS,
                            ]),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    if i == 0 {
                        appearance.card_1
                    } else {
                        appearance.card_2
                    },
                );
            }
            self.elements[0].as_widget().draw(
                tree_children.next().unwrap(),
                renderer,
                theme,
                style,
                card_layout,
                cursor,
                viewport,
            );
        } else {
            let layout = layout.collect::<Vec<_>>();
            // draw in reverse order so later cards appear behind earlier cards
            for ((inner, layout), c_state) in self
                .elements
                .iter()
                .rev()
                .zip(layout.into_iter().rev())
                .zip(tree_children.rev())
            {
                inner
                    .as_widget()
                    .draw(c_state, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) {
        if self.elements.is_empty() {
            return;
        }

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = state.state.downcast_mut::<State>();

            state.anim.anim_done(self.duration);
            if state.anim.last_change.is_some() {
                shell.request_redraw();
                shell.invalidate_layout();
            }
        }

        let my_state = state.state.downcast_ref::<State>();

        let mut layout = layout.children();
        let mut tree_children = state.children.iter_mut();
        let t = my_state.anim.t(self.duration, self.expanded);
        let fully_expanded = self.fully_expanded(t);
        let fully_unexpanded = self.fully_unexpanded(t);
        let show_less_state = tree_children.next();
        let clear_all_state = tree_children.next();

        if fully_expanded {
            let c_layout = layout.next().unwrap();
            let state = show_less_state.unwrap();
            self.show_less_button.as_widget_mut().update(
                state, event, c_layout, cursor, renderer, clipboard, shell, viewport,
            );

            if shell.is_event_captured() {
                return;
            }

            let c_layout = layout.next().unwrap();
            let state = clear_all_state.unwrap();
            self.clear_all_button.as_widget_mut().update(
                state, &event, c_layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        if shell.is_event_captured() {
            return;
        }

        for ((inner, layout), c_state) in self.elements.iter_mut().zip(layout).zip(tree_children) {
            inner.as_widget_mut().update(
                c_state, &event, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if shell.is_event_captured() || fully_unexpanded {
                break;
            }
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message> From<Cards<'a, Message>> for Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'a,
{
    fn from(cards: Cards<'a, Message>) -> Self {
        Self::new(cards)
    }
}

#[derive(Debug, Default)]
pub struct State {
    anim: anim::State,
}
