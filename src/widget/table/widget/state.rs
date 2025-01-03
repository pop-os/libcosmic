use slotmap::SecondaryMap;

use crate::widget::table::Entity;

pub struct State {
    pub(super) num_items: usize,
    pub(super) selected: SecondaryMap<Entity, bool>,
}
