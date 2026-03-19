// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Theme;
use crate::theme::{STATE_DEFAULT_BG, STATE_DEFAULT_COLOR};
use crate::widget::dropdown;
use iced::Background;

impl dropdown::menu::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> dropdown::menu::Appearance {
        let cosmic = self.cosmic();

        dropdown::menu::Appearance {
            text_color: cosmic.on_bg_color().into(),
            background: Background::Color(cosmic.background.component.base.into()),
            border_width: 0.0,
            border_radius: cosmic.corner_radii.radius_m.into(),
            border_color: iced::Color::TRANSPARENT,

            hovered_text_color: cosmic.on_bg_color().into(),
            hovered_background: Background::Color(STATE_DEFAULT_BG),

            selected_text_color: STATE_DEFAULT_COLOR,
            selected_background: Background::Color(STATE_DEFAULT_BG),

            description_color: cosmic.primary.component.on_disabled.into(),
        }
    }
}
