use crate::widget::segmented_button::Entity;

pub trait MenuAction: Clone + Copy + Eq + PartialEq {
    type Message;

    fn message(&self, entity: Option<Entity>) -> Self::Message;
}
