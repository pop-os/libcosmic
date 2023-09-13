// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{App, Message};
use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::theme::ThemeType;
use cosmic::widget::{
    column, container, divider, list, pick_list, radio, row, settings, spin_button, text,
};
use cosmic::Element;
use cosmic_time::{anim, id, once_cell::sync::Lazy};

pub static DEBUG_TOGGLER: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ThemeVariant {
    Custom,
    Dark,
    HighContrastDark,
    HighContrastLight,
    Light,
    System,
}

impl ThemeVariant {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Custom => "Custom",
            Self::Dark => "Dark",
            Self::HighContrastDark => "HighContrastDark",
            Self::HighContrastLight => "HighContrastLight",
            Self::Light => "Light",
            Self::System => "System",
        }
    }
}

impl From<&ThemeType> for ThemeVariant {
    fn from(theme: &ThemeType) -> Self {
        match theme {
            ThemeType::Light => ThemeVariant::Light,
            ThemeType::Dark => ThemeVariant::Dark,
            ThemeType::HighContrastDark => ThemeVariant::HighContrastDark,
            ThemeType::HighContrastLight => ThemeVariant::HighContrastLight,
            ThemeType::Custom(_) => ThemeVariant::Custom,
            ThemeType::System(_) => ThemeVariant::System,
        }
    }
}

impl From<ThemeType> for ThemeVariant {
    fn from(theme: ThemeType) -> Self {
        ThemeVariant::from(&theme)
    }
}

const THEME_CHOICES: &[ThemeVariant] = &[
    ThemeVariant::Light,
    ThemeVariant::Dark,
    ThemeVariant::System,
    ThemeVariant::HighContrastLight,
    ThemeVariant::HighContrastDark,
    ThemeVariant::Custom,
];

impl App
where
    Self: cosmic::Application,
{
    pub fn view_debug(&self) -> Element<Message> {
        let mut theme_choices = THEME_CHOICES.iter().cloned().map(|theme| {
            radio(
                theme.as_str(),
                theme,
                if ThemeVariant::from(cosmic::theme::active_type()) == theme {
                    Some(theme)
                } else {
                    None
                },
                Message::ThemeChanged,
            )
            .width(200)
        });

        column()
            .spacing(24)
            .push(
                column()
                    .spacing(8)
                    .push(text::heading("Change Theme"))
                    .push(list::container(
                        column()
                            .spacing(12)
                            .padding([0, 18])
                            .push(row().extend(theme_choices.by_ref().take(3)))
                            .push(row().extend(theme_choices))
                            .apply(container)
                            .center_x()
                            .width(Length::Fill),
                    )),
            )
            .push(
                column()
                    .spacing(8)
                    .push(text::heading("Debug Options"))
                    .push(list::container(
                        column()
                            .spacing(12)
                            .push(
                                container(anim!(
                                    DEBUG_TOGGLER,
                                    &self.timeline,
                                    String::from("Debug layout"),
                                    self.core.debug,
                                    |_chain, enable| { Message::DebugToggled(enable) },
                                ))
                                .padding([0, 18]),
                            )
                            .push(divider::horizontal::light())
                            .push(settings::item(
                                "Scaling Factor",
                                spin_button(&self.scale_factor_str, Message::ScalingFactorChanged),
                            ))
                            .push(divider::horizontal::light())
                            .push(settings::item(
                                "Layer",
                                pick_list(
                                    &["Default", "Primary", "Secondary"][..],
                                    Some(self.layer_selection),
                                    Message::LayerSelect,
                                ),
                            )),
                    )),
            )
            .into()
    }
}
