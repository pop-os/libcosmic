use iced::{
    Background,
    Color,
    Theme,
    widget::{
        container,
    },
};

pub fn list_view_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
        text_color: None,
        background: Some(Background::Color(
            match theme {
                Theme::Dark => Color::from_rgb8(0x27, 0x27, 0x27),
                Theme::Light => Color::from_rgb8(0xf7, 0xf7, 0xf7),
            }
        )),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
