use iced::{
    Background,
    Color,
    Theme,
    widget,
};

#[macro_export]
macro_rules! list_item {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::Row::with_children(vec![$($crate::iced::Element::from($x)),+])
        .padding([0, 8])
        .spacing(12)
    );
}
pub use list_item;

#[macro_export]
macro_rules! list_view {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::Container::new({
            let mut children = vec![$($crate::iced::Element::from($x)),+];

            //TODO: more efficient method for adding separators
            let mut i = 1;
            while i < children.len() {
                children.insert(i, $crate::iced::widget::horizontal_rule(12).into());
                i += 2;
            }

            $crate::iced::widget::Column::with_children(children)
            .spacing(12)
        })
        .padding([12, 16])
        .style(theme::Container::Custom(
            $crate::widget::list_view_style
        ))
    );
}
pub use list_view;

pub fn list_view_style(theme: &Theme) -> widget::container::Appearance {
    widget::container::Appearance {
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
