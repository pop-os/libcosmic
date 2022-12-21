// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod nav_bar {
    use crate::Theme;
    use iced::{widget, Background, Color};

    #[macro_export]
    macro_rules! nav_button {
        ($icon: expr, $title:expr, $active:expr) => {{
            $crate::iced::widget::Button::new(
                $crate::iced::widget::row!(
                    $crate::widget::icon($icon, 16)
                        .style(if $active {
                            $crate::theme::Svg::SymbolicLink
                        } else {
                            $crate::theme::Svg::Symbolic
                        }),
                    $crate::iced::widget::Text::new($title)
                        .vertical_alignment($crate::iced::alignment::Vertical::Center),
                    $crate::iced::widget::horizontal_space($crate::iced::Length::Fill),
                )
                .padding([4, 16])
                .spacing(8)
            )
            .style(if $active {
                $crate::theme::Button::LinkActive
            } else {
                $crate::theme::Button::Text
            })
        }};
    }

    pub fn nav_bar_sections_style(theme: &Theme) -> widget::container::Appearance {
        let cosmic = &theme.cosmic().primary;
        widget::container::Appearance {
            text_color: Some(cosmic.on.into()),
            background: Some(Background::Color(cosmic.base.into())),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    pub fn nav_bar_pages_style(theme: &Theme) -> widget::container::Appearance {
        let primary = &theme.cosmic().primary;
        let secondary = &theme.cosmic().secondary;
        widget::container::Appearance {
            text_color: Some(primary.on.into()),
            background: Some(Background::Color(secondary.component.base.into())),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    pub use nav_button;
}
