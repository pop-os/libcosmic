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
