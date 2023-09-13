// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{button, Builder, Style};
use crate::widget::{
    icon::{self, Handle},
    tooltip,
};
use crate::Element;
use apply::Apply;
use iced_core::{font::Weight, text::LineHeight, widget::Id, Alignment, Length, Padding};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Icon>;

pub struct Icon {
    handle: Handle,
    vertical: bool,
}

pub fn icon<'a, Message>(handle: impl Into<Handle>) -> Button<'a, Message> {
    Button::new(Icon {
        handle: handle.into(),
        vertical: false,
    })
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(icon: Icon) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();
            let padding = theme.space_xxs();

            Self {
                id: Id::unique(),
                label: Cow::Borrowed(""),
                tooltip: Cow::Borrowed(""),
                on_press: None,
                width: Length::Shrink,
                height: Length::Fixed(46.0),
                padding: Padding::from(padding),
                spacing: theme.space_xxxs(),
                icon_size: if icon.handle.symbolic { 16 } else { 24 },
                line_height: 20,
                font_size: 14,
                font_weight: Weight::Normal,
                style: Style::Icon,
                variant: icon,
            }
        })
    }

    /// Applies the **Extra Small** button size preset.
    pub fn extra_small(mut self) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();

            self.font_size = 14;
            self.font_weight = Weight::Normal;
            self.icon_size = 16;
            self.line_height = 20;
            self.height = Length::Fixed(36.0);
            self.padding = Padding::from(theme.space_xxs());
            self.spacing = theme.space_xxxs();
        });

        self
    }

    /// Applies the **Medium** button size preset.
    pub fn medium(mut self) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();

            self.font_size = 24;
            self.font_weight = Weight::Normal;
            self.icon_size = 32;
            self.line_height = 32;
            self.height = Length::Fixed(56.0);
            self.padding = Padding::from(theme.space_xs());
            self.spacing = theme.space_xxs();
        });

        self
    }

    /// Applies the **Large** button size preset.
    pub fn large(mut self) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();

            self.font_size = 28;
            self.font_weight = Weight::Normal;
            self.icon_size = 40;
            self.line_height = 36;
            self.height = Length::Fixed(64.0);
            self.padding = Padding::from(theme.space_xs());
            self.spacing = theme.space_xxs();
        });

        self
    }

    /// Applies the **Extra Large** button size preset.
    pub fn extra_large(mut self) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();
            let padding = theme.space_xs();

            self.font_size = 32;
            self.font_weight = Weight::Light;
            self.icon_size = 56;
            self.line_height = 44;
            self.height = Length::Fixed(80.0);
            self.padding = Padding::from(padding);
            self.spacing = theme.space_xxs();
        });

        self
    }

    pub fn inherit_colors(mut self) -> Self {
        self.style = Style::IconInheritColors;
        self
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.variant.vertical = vertical;
        self.style = Style::IconVertical;
        self
    }
}

impl<'a, Message: Clone + 'static> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(mut builder: Button<'a, Message>) -> Element<'a, Message> {
        let mut content = Vec::with_capacity(2);

        if let icon::Data::Name(ref mut named) = builder.variant.handle.data {
            named.size = Some(builder.icon_size);
        }

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
                    .font({
                        let mut font = crate::font::DEFAULT;
                        font.weight = builder.font_weight;
                        font
                    })
                    .into(),
            );
        }

        let button = if builder.variant.vertical {
            crate::widget::column::with_children(content)
                .padding(builder.padding)
                // .width(builder.width)
                // .height(builder.height)
                .spacing(builder.spacing)
                .align_items(Alignment::Center)
                .apply(button)
        } else {
            crate::widget::row::with_children(content)
                .padding(builder.padding)
                .width(builder.width)
                .height(builder.height)
                .spacing(builder.spacing)
                .align_items(Alignment::Center)
                .apply(button)
        };

        let button = button
            .padding(0)
            .id(builder.id)
            .on_press_maybe(builder.on_press)
            .style(builder.style);

        if builder.tooltip.is_empty() {
            button.into()
        } else {
            tooltip(button, builder.tooltip, tooltip::Position::Top)
                .size(builder.font_size)
                .font({
                    let mut font = crate::font::DEFAULT;
                    font.weight = builder.font_weight;
                    font
                })
                .into()
        }
    }
}
