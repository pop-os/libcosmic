// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Builder, ButtonClass};
use crate::Element;
use crate::widget::{
    icon::{self, Handle},
    tooltip,
};
use apply::Apply;
use iced_core::{Alignment, Length, Padding, font::Weight, text::LineHeight, widget::Id};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Icon>;

/// The icon variant of a button.
pub struct Icon {
    handle: Handle,
    vertical: bool,
    selected: bool,
}

/// A button constructed from an icon handle, using icon button styling.
pub fn icon<'a, Message>(handle: impl Into<Handle>) -> Button<'a, Message> {
    Button::new(Icon {
        handle: handle.into(),
        vertical: false,
        selected: false,
    })
}

impl<Message> Button<'_, Message> {
    pub fn new(icon: Icon) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();
        let padding = theme.space_xxs();

        Self {
            id: Id::unique(),
            label: Cow::Borrowed(""),
            tooltip: Cow::Borrowed(""),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::from(padding),
            spacing: theme.space_xxxs(),
            icon_size: if icon.handle.symbolic { 16 } else { 24 },
            line_height: 20,
            font_size: 14,
            font_weight: Weight::Normal,
            class: ButtonClass::Icon,
            variant: icon,
        }
    }

    /// Applies the **Extra Small** button size preset.
    pub fn extra_small(mut self) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();

        self.font_size = 14;
        self.font_weight = Weight::Normal;
        self.icon_size = 16;
        self.line_height = 20;
        self.padding = Padding::from(theme.space_xxs());
        self.spacing = theme.space_xxxs();

        self
    }

    /// Applies the **Medium** button size preset.
    pub fn medium(mut self) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();

        self.font_size = 24;
        self.font_weight = Weight::Normal;
        self.icon_size = 32;
        self.line_height = 32;
        self.padding = Padding::from(theme.space_xs());
        self.spacing = theme.space_xxs();

        self
    }

    /// Applies the **Large** button size preset.
    pub fn large(mut self) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();

        self.font_size = 28;
        self.font_weight = Weight::Normal;
        self.icon_size = 40;
        self.line_height = 36;
        self.padding = Padding::from(theme.space_xs());
        self.spacing = theme.space_xxs();

        self
    }

    /// Applies the **Extra Large** button size preset.
    pub fn extra_large(mut self) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();
        let padding = theme.space_xs();

        self.font_size = 32;
        self.font_weight = Weight::Light;
        self.icon_size = 56;
        self.line_height = 44;
        self.padding = Padding::from(padding);
        self.spacing = theme.space_xxs();

        self
    }

    #[inline]
    pub fn selected(mut self, selected: bool) -> Self {
        self.variant.selected = selected;
        self
    }

    #[inline]
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.variant.vertical = vertical;
        self.class = ButtonClass::IconVertical;
        self
    }
}

impl<'a, Message: Clone + 'static> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(mut builder: Button<'a, Message>) -> Element<'a, Message> {
        let mut content = Vec::with_capacity(2);

        content.push(
            crate::widget::icon(builder.variant.handle.clone())
                .size(builder.icon_size)
                .into(),
        );

        if !builder.label.is_empty() {
            content.push(
                crate::widget::text(builder.label)
                    .size(builder.font_size)
                    .line_height(LineHeight::Absolute(builder.line_height.into()))
                    .font(crate::font::Font {
                        weight: builder.font_weight,
                        ..crate::font::default()
                    })
                    .into(),
            );
        }

        let button = if builder.variant.vertical {
            crate::widget::column::with_children(content)
                .padding(builder.padding)
                .spacing(builder.spacing)
                .align_x(Alignment::Center)
                .apply(super::custom)
        } else {
            crate::widget::row::with_children(content)
                .padding(builder.padding)
                .width(builder.width)
                .height(builder.height)
                .spacing(builder.spacing)
                .align_y(Alignment::Center)
                .apply(super::custom)
        };

        let button = button
            .padding(0)
            .id(builder.id)
            .on_press_maybe(builder.on_press)
            .selected(builder.variant.selected)
            .class(builder.class);

        if builder.tooltip.is_empty() {
            button.into()
        } else {
            tooltip(
                button,
                crate::widget::text(builder.tooltip)
                    .size(builder.font_size)
                    .font(crate::font::Font {
                        weight: builder.font_weight,
                        ..crate::font::default()
                    }),
                tooltip::Position::Top,
            )
            .into()
        }
    }
}
