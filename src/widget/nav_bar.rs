use iced::{
    Background,
    Color,
    Theme,
    widget::{
        container,
    },
};

pub fn nav_bar_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
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
