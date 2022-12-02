#[macro_export]
macro_rules! separator {
    ($size:expr) => {
        $crate::iced::widget::horizontal_rule($size)
            .style($crate::theme::Rule::Custom($crate::widget::separator_style))
    };
}
