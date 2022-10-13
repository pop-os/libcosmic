pub mod nav_bar {
    use iced::{widget, Background, Color, Theme};

    #[macro_export]
    macro_rules! nav_button {
        ($icon: expr, $title:expr, $condensed:expr) => {{
            if $condensed {
                $crate::iced::widget::Button::new($crate::widget::icon($icon, 22)).padding(8)
            } else {
                $crate::widget::button!(
                    $crate::widget::icon($icon, 22),
                    $crate::iced::widget::Text::new($title),
                    $crate::iced::widget::horizontal_space($crate::iced::Length::Fill),
                )
            }
        }};
    }

    pub fn nav_bar_style(theme: &Theme) -> widget::container::Appearance {
        let cosmic = &theme.cosmic().primary;
        widget::container::Appearance {
            text_color: Some(cosmic.on.into()),
            background: Some(Background::Color(cosmic.base.into())),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    pub use nav_button;
}
