// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::iced::widget;
use crate::{theme, Renderer, Theme};

#[must_use]
pub fn horizontal_rule(size: u16) -> widget::Rule<Renderer> {
    widget::horizontal_rule(size).style(theme::Rule::Custom(separator_style))
}

#[must_use]
pub fn vertical_rule(size: u16) -> widget::Rule<Renderer> {
    widget::vertical_rule(size).style(theme::Rule::Custom(separator_style))
}

fn separator_style(theme: &Theme) -> widget::rule::Appearance {
    let cosmic = &theme.cosmic().primary;
    widget::rule::Appearance {
        color: cosmic.divider.into(),
        width: 1,
        radius: 0.0,
        fill_mode: widget::rule::FillMode::Padded(10),
    }
}