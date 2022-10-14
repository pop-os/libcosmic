// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod nav_bar {
    use iced::{widget, Background, Color};
    use crate::Theme;

    #[macro_export]
    macro_rules! nav_button {
        ($title:expr, $icon: expr, $condensed:expr, $active: expr) => {
            if $condensed {
                $crate::iced::widget::Button::new(
                    $crate::iced::widget::Column::with_children(vec![
                        $crate::widget::icon($icon, 26).into(),
                        $crate::iced::widget::text($title).size(14).into(),
                    ])
                    .spacing(5)
                    .width($crate::iced::Length::Units(110))
                    .height($crate::iced::Length::Units(60))
                    .align_items($crate::iced::alignment::Alignment::Center),
                )
                .style(if $active {
                    $crate::iced::theme::Button::Primary.into()
                } else {
                    $crate::iced::theme::Button::Text.into()
                })
            } else {
                $crate::iced::widget::Button::new(
                    $crate::iced::widget::row![
                        $crate::widget::icon($icon, 20),
                        $crate::iced::widget::Text::new($title).size(16).width($crate::iced::Length::Fill)
                    ]
                    .spacing(10),
                )
                .padding(10)
                .style(if $active {
                    $crate::iced::theme::Button::Primary.into()
                } else {
                    $crate::iced::theme::Button::Text.into()
                })
            }
        };
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
