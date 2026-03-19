// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::cosmic_theme::{Density, Spacing};
use crate::{Element, theme, widget};
use apply::Apply;
use derive_setters::Setters;
use iced::{Border, Color, Length};
use iced_core::font::Weight;
use iced_core::{Vector, Widget, widget::tree};
use std::borrow::Cow;

#[must_use]
pub fn header_bar<'a, Message>() -> HeaderBar<'a, Message> {
    HeaderBar {
        title: Cow::Borrowed(""),
        app_icon: None,
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
        on_right_click: None,
        start: Vec::new(),
        center: Vec::new(),
        end: Vec::new(),
        density: None,
        focused: false,
        maximized: false,
        sharp_corners: false,
        is_ssd: false,
        on_double_click: None,
        is_condensed: false,
        transparent: false,
        corner_radius: None,
    }
}

#[derive(Setters)]
pub struct HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[setters(skip)]
    title: Cow<'a, str>,

    /// Optional app icon displayed before the title
    #[setters(skip)]
    app_icon: Option<widget::icon::Handle>,

    /// A message emitted when the close button is pressed.
    #[setters(strip_option)]
    on_close: Option<Message>,

    /// A message emitted when dragged.
    #[setters(strip_option)]
    on_drag: Option<Message>,

    /// A message emitted when the maximize button is pressed.
    #[setters(strip_option)]
    on_maximize: Option<Message>,

    /// A message emitted when the minimize button is pressed.
    #[setters(strip_option)]
    on_minimize: Option<Message>,

    /// A message emitted when the header is double clicked,
    /// usually used to maximize the window.
    #[setters(strip_option)]
    on_double_click: Option<Message>,

    /// A message emitted when the header is right clicked.
    #[setters(strip_option)]
    on_right_click: Option<Message>,

    /// Elements packed at the start of the headerbar.
    #[setters(skip)]
    start: Vec<Element<'a, Message>>,

    /// Elements packed in the center of the headerbar.
    #[setters(skip)]
    center: Vec<Element<'a, Message>>,

    /// Elements packed at the end of the headerbar.
    #[setters(skip)]
    end: Vec<Element<'a, Message>>,

    /// Controls the density of the headerbar.
    #[setters(strip_option)]
    density: Option<Density>,

    /// Focused state of the window
    focused: bool,

    /// Maximized state of the window
    maximized: bool,

    /// Whether the corners of the window should be sharp
    sharp_corners: bool,

    /// HeaderBar used for server-side decorations
    is_ssd: bool,

    /// Whether the headerbar should be compact
    is_condensed: bool,

    /// Whether the headerbar should be transparent
    transparent: bool,

    /// Explicit corner radius [TL, TR, BR, BL] from the compositor.
    /// When set, overrides the theme's radius_window() for SSD headers.
    #[setters(strip_option)]
    corner_radius: Option<[f32; 4]>,
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    /// Defines the title of the window
    #[must_use]
    pub fn title(mut self, title: impl Into<Cow<'a, str>> + 'a) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the app icon displayed before the title
    #[must_use]
    pub fn app_icon(mut self, icon: widget::icon::Handle) -> Self {
        self.app_icon = Some(icon);
        self
    }

    /// Pushes an element to the start region.
    #[must_use]
    pub fn start(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.start.push(widget.into());
        self
    }

    /// Pushes an element to the center region.
    #[must_use]
    pub fn center(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.center.push(widget.into());
        self
    }

    /// Pushes an element to the end region.
    #[must_use]
    pub fn end(mut self, widget: impl Into<Element<'a, Message>> + 'a) -> Self {
        self.end.push(widget.into());
        self
    }

    /// Build the widget
    #[must_use]
    #[inline]
    pub fn build(self) -> HeaderBarWidget<'a, Message> {
        HeaderBarWidget {
            header_bar_inner: self.view(),
        }
    }
}

pub struct HeaderBarWidget<'a, Message> {
    header_bar_inner: Element<'a, Message>,
}

impl<Message: Clone + 'static> Widget<Message, crate::Theme, crate::Renderer>
    for HeaderBarWidget<'_, Message>
{
    fn diff(&mut self, tree: &mut tree::Tree) {
        tree.diff_children(&mut [&mut self.header_bar_inner]);
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.header_bar_inner)]
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.header_bar_inner.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &crate::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let child_tree = &mut tree.children[0];
        let child = self
            .header_bar_inner
            .as_widget()
            .layout(child_tree, renderer, limits);
        iced_core::layout::Node::with_children(child.size(), vec![child])
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        let layout_children = layout.children().next().unwrap();
        let state_children = &tree.children[0];
        self.header_bar_inner.as_widget().draw(
            state_children,
            renderer,
            theme,
            style,
            layout_children,
            cursor,
            viewport,
        );
    }

    fn on_event(
        &mut self,
        state: &mut tree::Tree,
        event: iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced_core::Rectangle,
    ) -> iced_core::event::Status {
        let child_state = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget_mut().on_event(
            child_state,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &crate::Renderer,
    ) -> iced_core::mouse::Interaction {
        let child_tree = &state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget().mouse_interaction(
            child_tree,
            child_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        let child_tree = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner
            .as_widget()
            .operate(child_tree, child_layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let child_tree = &mut state.children[0];
        let child_layout = layout.children().next().unwrap();
        self.header_bar_inner.as_widget_mut().overlay(
            child_tree,
            child_layout,
            renderer,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        state: &tree::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        if let Some((child_tree, child_layout)) =
            state.children.iter().zip(layout.children()).next()
        {
            self.header_bar_inner.as_widget().drag_destinations(
                child_tree,
                child_layout,
                renderer,
                dnd_rectangles,
            );
        }
    }

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &tree::Tree,
        p: iced::mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let c_layout = layout.children().next().unwrap();
        let c_state = &state.children[0];
        self.header_bar_inner
            .as_widget()
            .a11y_nodes(c_layout, c_state, p)
    }
}

impl<'a, Message: Clone + 'static> HeaderBar<'a, Message> {
    #[allow(clippy::too_many_lines)]
    /// Converts the headerbar builder into an Iced element.
    pub fn view(mut self) -> Element<'a, Message> {
        let Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::spacing();

        // Take ownership of the regions to be packed.
        let mut start = std::mem::take(&mut self.start);
        let center = std::mem::take(&mut self.center);
        let mut end = std::mem::take(&mut self.end);

        let window_control_cnt = self.on_close.is_some() as usize
            + self.on_maximize.is_some() as usize
            + self.on_minimize.is_some() as usize;
        // Also packs the window controls at the very end.
        end.push(self.window_controls());

        // Build the title element (with optional app icon) and place it in the start region.
        if !self.title.is_empty() && !self.is_condensed {
            let mut title = Cow::default();
            std::mem::swap(&mut title, &mut self.title);

            // SSD uses custom font: 15px, Inter (interface font), weight 500, color #474747
            let title_text: Element<'a, Message> = if self.is_ssd {
                let medium_font = iced::Font {
                    weight: Weight::Medium,
                    ..crate::font::default()
                };
                widget::text(title)
                    .size(15.0)
                    .font(medium_font)
                    .class(Color::from_rgb8(0x1B, 0x1B, 0x1B))
                    .into()
            } else {
                widget::text::title3(title).into()
            };

            // SSD icon: request high-res source (128px lookup in resolve_app_icon),
            // display at 18px with Contain to preserve aspect ratio.
            let title_element: Element<'a, Message> =
                if let Some(icon_handle) = self.app_icon.take() {
                    let (icon_size, icon_gap) = if self.is_ssd {
                        (18, 8)
                    } else {
                        (24, space_xxs)
                    };
                    let icon_widget = if self.is_ssd {
                        widget::icon::icon(icon_handle)
                            .size(icon_size)
                            .content_fit(iced::ContentFit::Contain)
                    } else {
                        widget::icon::icon(icon_handle).size(icon_size)
                    };
                    widget::row::with_capacity(2)
                        .push(icon_widget)
                        .push(title_text)
                        .spacing(icon_gap)
                        .align_y(iced::Alignment::Center)
                        .into()
                } else {
                    title_text
                };

            start.push(title_element);
        }

        // Center content depending on window border
        // Non-maximized windows use larger horizontal padding to clear rounded corners.
        let padding = match self.density.unwrap_or_else(crate::config::header_size) {
            Density::Compact => {
                if self.maximized {
                    [4, 8, 4, 8]
                } else {
                    [3, 16, 4, 16]
                }
            }
            _ => {
                if self.maximized {
                    [8, 8, 8, 8]
                } else {
                    [7, 16, 8, 16]
                }
            }
        };

        let acc_count = |v: &[Element<'a, Message>]| {
            v.iter().fold(0, |acc, e| {
                acc + match e.as_widget().size().width {
                    Length::Fixed(w) if w > 30. => (w / 30.0).ceil() as usize,
                    _ => 1,
                }
            })
        };

        let left_len = acc_count(&start);
        let right_len = acc_count(&end);

        let portion = ((left_len.max(right_len + window_control_cnt) as f32
            / center.len().max(1) as f32)
            .round() as u16)
            .max(1);
        let (left_portion, right_portion) = if center.is_empty() {
            let left_to_right_ratio = left_len as f32 / right_len.max(1) as f32;
            let right_to_left_ratio = right_len as f32 / left_len.max(1) as f32;
            if right_to_left_ratio > 2. || left_len < 1 {
                (1, 2)
            } else if left_to_right_ratio > 2. || right_len < 1 {
                (2, 1)
            } else {
                (left_len as u16, (right_len + window_control_cnt) as u16)
            }
        } else {
            (portion, portion)
        };

        let (header_height, header_padding) = (Length::Fixed(48.0), [8, 12, 8, 12]);

        // Creates the headerbar widget.
        let widget = widget::row::with_capacity(3)
            // Start region: includes app icon + title + user start elements.
            .push(
                widget::row::with_children(start)
                    .spacing(space_xxxs)
                    .align_y(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::Alignment::Start)
                    // SSD: title fills all remaining space; non-SSD: use portion ratio
                    .width(if self.is_ssd {
                        Length::Fill
                    } else {
                        Length::FillPortion(left_portion)
                    }),
            )
            // Center region: only explicit center elements.
            .push_maybe(if !center.is_empty() {
                Some(
                    widget::row::with_children(center)
                        .spacing(space_xxxs)
                        .align_y(iced::Alignment::Center)
                        .apply(widget::container)
                        .center_x(Length::Fill)
                        .into(),
                )
            } else {
                None::<Element<'a, Message>>
            })
            .push(
                widget::row::with_children(end)
                    .spacing(space_xxs)
                    .align_y(iced::Alignment::Center)
                    .apply(widget::container)
                    .align_x(iced::Alignment::End)
                    // SSD: buttons shrink to fit; non-SSD: use portion ratio
                    .width(if self.is_ssd {
                        Length::Shrink
                    } else {
                        Length::FillPortion(right_portion)
                    }),
            )
            .align_y(iced::Alignment::Center)
            .height(header_height)
            .padding(header_padding)
            .spacing(8)
            .apply(widget::container)
            .class({
                crate::theme::Container::custom(move |theme| {
                    let cosmic = theme.cosmic();
                    iced_widget::container::Style {
                        icon_color: Some(Color::from(cosmic.background.on)),
                        text_color: Some(Color::from(cosmic.background.on)),
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: [0.0; 4].into(),
                            ..Default::default()
                        },
                        shadow: Default::default(),
                    }
                })
            })
            .center_y(Length::Shrink);

        let widget = {
            use iced::widget::{horizontal_rule, rule};
            widget::column::with_capacity(2)
                .push(widget)
                .push(
                    horizontal_rule(1).class(crate::theme::Rule::Custom(Box::new(
                        |_: &crate::Theme| rule::Style {
                            color: Color::from_rgba8(224, 224, 224, 1.0),
                            width: 1,
                            radius: 0.0.into(),
                            fill_mode: rule::FillMode::Full,
                        },
                    ))),
                )
                .apply(widget::mouse_area)
        };

        let mut widget = widget;

        // SSD: show grab cursor over header (buttons override with pointer)
        if self.is_ssd {
            widget = widget.interaction(iced_core::mouse::Interaction::Grab);
        }

        // Assigns a message to emit when the headerbar is dragged.
        if let Some(message) = self.on_drag.clone() {
            widget = widget.on_drag(message);
        }

        // Assigns a message to emit when the headerbar is double-clicked.
        if let Some(message) = self.on_maximize.clone() {
            widget = widget.on_release(message);
        }
        if let Some(message) = self.on_double_click.clone() {
            widget = widget.on_double_press(message);
        }
        if let Some(message) = self.on_right_click.clone() {
            widget = widget.on_right_press(message);
        }

        widget.into()
    }

    /// Creates the widget for window controls.
    fn window_controls(&mut self) -> Element<'a, Message> {
        const ICON_MINIMIZE: &[u8] = include_bytes!("../../res/icons/window-minimize.svg");
        const ICON_MAXIMIZE: &[u8] = include_bytes!("../../res/icons/window-maximize.svg");
        const ICON_RESTORE: &[u8] = include_bytes!("../../res/icons/window-restore.svg");
        const ICON_CLOSE: &[u8] = include_bytes!("../../res/icons/window-close.svg");

        macro_rules! wc_icon {
            ($svg_bytes:expr, $size:expr, $on_press:expr, $is_close:expr) => {{
                let icon_w = widget::icon::icon(widget::icon::from_svg_bytes($svg_bytes))
                    .size($size)
                    .class(crate::theme::Svg::custom(|_| iced::widget::svg::Style {
                        color: Some(Color::from_rgb8(0x3D, 0x3D, 0x3D)),
                    }));
                let btn: Element<'a, Message> = widget::button::custom(icon_w)
                    .padding([4, 6])
                    .class(crate::theme::Button::Custom {
                        active: Box::new(move |_focused, _theme| {
                            crate::widget::button::Style {
                                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                                text_color: Some(Color::from_rgb8(0x3D, 0x3D, 0x3D)),
                                icon_color: Some(Color::from_rgb8(0x3D, 0x3D, 0x3D)),
                                border_radius: [6.0; 4].into(),
                                ..Default::default()
                            }
                        }),
                        disabled: Box::new(|_theme| crate::widget::button::Style::default()),
                        hovered: Box::new(move |_focused, _theme| {
                            let bg = if $is_close {
                                Color::from_rgba8(224, 64, 64, 0.20)
                            } else {
                                Color::from_rgb8(208, 208, 208)
                            };
                            let icon_c = if $is_close {
                                Color::from_rgb8(224, 64, 64)
                            } else {
                                Color::from_rgb8(0x3D, 0x3D, 0x3D)
                            };
                            crate::widget::button::Style {
                                background: Some(iced::Background::Color(bg)),
                                text_color: Some(icon_c),
                                icon_color: Some(icon_c),
                                border_radius: [6.0; 4].into(),
                                ..Default::default()
                            }
                        }),
                        pressed: Box::new(move |_focused, _theme| {
                            let bg = if $is_close {
                                Color::from_rgba8(200, 50, 50, 0.30)
                            } else {
                                Color::from_rgb8(190, 190, 190)
                            };
                            let icon_c = if $is_close {
                                Color::from_rgb8(200, 50, 50)
                            } else {
                                Color::from_rgb8(0x3D, 0x3D, 0x3D)
                            };
                            crate::widget::button::Style {
                                background: Some(iced::Background::Color(bg)),
                                text_color: Some(icon_c),
                                icon_color: Some(icon_c),
                                border_radius: [6.0; 4].into(),
                                ..Default::default()
                            }
                        }),
                    })
                    .on_press($on_press)
                    .into();
                btn
            }};
        }

        widget::row::with_capacity(3)
            .push_maybe(
                self.on_minimize
                    .take()
                    .map(|m: Message| wc_icon!(ICON_MINIMIZE, 14, m, false)),
            )
            .push_maybe(self.on_maximize.take().map(|m| {
                if self.maximized {
                    wc_icon!(ICON_RESTORE, 14, m, false)
                } else {
                    wc_icon!(ICON_MAXIMIZE, 14, m, false)
                }
            }))
            .push_maybe(self.on_close.take().map(|m| wc_icon!(ICON_CLOSE, 14, m, true)))
            .spacing(2)
            .apply(widget::container)
            .class(crate::theme::Container::custom(move |_theme| {
                iced_widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb8(232, 232, 232))),
                    border: Border {
                        radius: [8.0; 4].into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .padding([2, 4])
            .center_y(Length::Fill)
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBar<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBar<'a, Message>) -> Self {
        Element::new(headerbar.build())
    }
}

impl<'a, Message: Clone + 'static> From<HeaderBarWidget<'a, Message>> for Element<'a, Message> {
    fn from(headerbar: HeaderBarWidget<'a, Message>) -> Self {
        Element::new(headerbar)
    }
}
