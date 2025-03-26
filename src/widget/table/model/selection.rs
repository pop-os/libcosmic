// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Describes logic specific to the single-select and multi-select modes of a model.

use super::{
    Entity, Model,
    category::{ItemCategory, ItemInterface},
};
use std::collections::HashSet;

/// Describes a type that has selectable items.
pub trait Selectable {
    /// Activate an item.
    fn activate(&mut self, id: Entity);

    /// Deactivate an item.
    fn deactivate(&mut self, id: Entity);

    /// Checks if the item is active.
    fn is_active(&self, id: Entity) -> bool;
}

/// [`Model<SingleSelect>`] Ensures that only one key may be selected.
#[derive(Debug, Default)]
pub struct SingleSelect {
    pub active: Entity,
}

impl<Item: ItemInterface<Category>, Category: ItemCategory> Selectable
    for Model<SingleSelect, Item, Category>
{
    fn activate(&mut self, id: Entity) {
        if !self.items.contains_key(id) {
            return;
        }

        self.selection.active = id;
    }

    fn deactivate(&mut self, id: Entity) {
        if id == self.selection.active {
            self.selection.active = Entity::default();
        }
    }

    fn is_active(&self, id: Entity) -> bool {
        self.selection.active == id
    }
}

impl<Item: ItemInterface<Category>, Category: ItemCategory> Model<SingleSelect, Item, Category> {
    /// Get an immutable reference to the data associated with the active item.
    #[must_use]
    pub fn active_data<Data: 'static>(&self) -> Option<&Data> {
        self.data(self.active())
    }

    /// Get a mutable reference to the data associated with the active item.
    #[must_use]
    pub fn active_data_mut<Data: 'static>(&mut self) -> Option<&mut Data> {
        self.data_mut(self.active())
    }

    /// Deactivates the active item.
    pub fn deactivate(&mut self) {
        Selectable::deactivate(self, Entity::default());
    }

    /// The ID of the active item.
    #[must_use]
    pub fn active(&self) -> Entity {
        self.selection.active
    }
}

/// [`Model<MultiSelect>`] permits multiple keys to be active at a time.
#[derive(Debug, Default)]
pub struct MultiSelect {
    pub active: HashSet<Entity>,
}

impl<Item: ItemInterface<Category>, Category: ItemCategory> Selectable
    for Model<MultiSelect, Item, Category>
{
    fn activate(&mut self, id: Entity) {
        if !self.items.contains_key(id) {
            return;
        }

        if !self.selection.active.insert(id) {
            self.selection.active.remove(&id);
        }
    }

    fn deactivate(&mut self, id: Entity) {
        self.selection.active.remove(&id);
    }

    fn is_active(&self, id: Entity) -> bool {
        self.selection.active.contains(&id)
    }
}

impl<Item: ItemInterface<Category>, Category: ItemCategory> Model<MultiSelect, Item, Category> {
    /// Deactivates the item in the model.
    pub fn deactivate(&mut self, id: Entity) {
        Selectable::deactivate(self, id);
    }

    /// The IDs of the active items.
    pub fn active(&self) -> impl Iterator<Item = Entity> + '_ {
        self.selection.active.iter().copied()
    }
}
