#[macro_export]
macro_rules! button {
    ($($x:expr),+ $(,)?) => (
        $crate::iced::widget::Button::new(
            $crate::iced::widget::Row::with_children(
                vec![$($crate::iced::Element::from($x)),+]
            )
            .spacing(8)
        )
        .padding([8, 16])
    );
}
pub use button;

mod icon;
pub use self::icon::icon;

mod list_view;
pub use list_view::list_view_style;

mod nav_bar;
pub use nav_bar::{nav_bar, nav_bar_style};
