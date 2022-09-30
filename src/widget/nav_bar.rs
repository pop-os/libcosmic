use iced::{
    Background,
    Color,
    Theme,
    widget
};

#[macro_export]
macro_rules! nav_bar {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::Container::new(
            $crate::iced::widget::Column::with_children(
                vec![$($crate::iced::Element::from($x)),+]
            )
            .spacing(12)
        )
        .max_width(300)
        .padding(12)
        .style(theme::Container::Custom(
            $crate::widget::nav_bar_style
        ))
    );
}
pub use nav_bar;

pub fn nav_bar_style(theme: &Theme) -> widget::container::Appearance {
    widget::container::Appearance {
        text_color: None,
        background: Some(Background::Color(
            match theme {
                Theme::Dark => Color::from_rgb8(0x29, 0x29, 0x29),
                Theme::Light => Color::from_rgb8(0xfd, 0xfd, 0xfd),
            }
        )),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
