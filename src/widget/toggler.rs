use iced::{
    Length,
    widget,
};

pub fn toggler<'a, Message, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> widget::Toggler<'a, Message, Renderer>
where
    Renderer: iced_native::text::Renderer,
    Renderer::Theme: widget::toggler::StyleSheet,
{
    widget::Toggler::new(is_checked, label, f)
        .size(24)
        .spacing(12)
        .width(Length::Shrink)
}
