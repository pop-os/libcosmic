use iced::{widget, Background, Color, Theme};

#[macro_export]
macro_rules! nav_bar {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::Container::new(
            $crate::scrollable!(
                $crate::iced::widget::row![
                    $crate::iced::widget::Column::with_children(
                        vec![$($crate::iced::Element::from($x)),+]
                    )
                    .spacing(12)
                    .padding(12),
                ]
            )
        )
        .max_width(300)
        .padding(12)
        .height(Length::Fill)
        .style(theme::Container::Custom(
            $crate::widget::nav_bar_style
        ))
    );
}
pub use nav_bar;

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
pub use nav_button;
