use crate::widget::segmented_button::Entity;

/// `MenuAction` is a trait that represents an action in a menu.
///
/// It is used to define the behavior of menu items when they are activated.
/// Each menu item can have a unique action associated with it.
///
/// This trait is generic over a type `Message` which is the type of message
/// that will be produced when the action is triggered.
///
/// # Example
///
/// ```
/// use cosmic::widget::menu::action::MenuAction;
/// use cosmic::widget::segmented_button::Entity;
///
/// #[derive(Clone, Copy, Eq, PartialEq)]
/// enum MyMessage {
///     Open,
///     Save,
///     Quit,
/// }
///
/// #[derive(Clone, Copy, Eq, PartialEq)]
/// enum MyAction {
///     Open,
///     Save,
///     Quit,
/// }
///
/// impl MenuAction for MyAction {
///     type Message = MyMessage;
///
///     fn message(&self, entity: Option<Entity>) -> Self::Message {
///         match self {
///             MyAction::Open => MyMessage::Open,
///             MyAction::Save => MyMessage::Save,
///             MyAction::Quit => MyMessage::Quit,
///         }
///     }
/// }
/// ```
pub trait MenuAction: Clone + Copy + Eq + PartialEq {
    /// The type of message that will be produced when the action is triggered.
    type Message;

    /// Returns a message of type `Self::Message` when the action is triggered.
    ///
    /// # Arguments
    ///
    /// * `entity` - An optional `Entity` that may be associated with the action.
    ///
    /// # Returns
    ///
    /// * `Self::Message` - The message that is produced when the action is triggered.
    fn message(&self, entity: Option<Entity>) -> Self::Message;
}
