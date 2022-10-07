#[macro_export]
macro_rules! scrollbar {
    ($x:expr) => (
        $crate::iced::widget::scrollable($x)
            .scrollbar_width(8)
            .scroller_width(8)
    );
}