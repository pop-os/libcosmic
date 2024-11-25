#[doc(inline)]
pub use iced::widget::{image, image::*, Image};

#[cfg(feature = "winit")]
use {
    crate::app::Task,
    iced_accessibility::Id,
    iced_core::widget::operation,
    iced_runtime::{task, Action},
};

#[cfg(feature = "winit")]
/// Produces a [`Task`] that sets a new [`Handle`] to the [`Image`] with the given [`Id`].
pub fn set_handle<Message: 'static>(id: Id, handle: impl Into<image::Handle>) -> Task<Message> {
    task::effect(Action::widget(operation::image::set_handle(
        id,
        handle.into(),
    )))
}
