// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Describes logic specific to the single-select and multi-select modes of a model.

use super::{Entity, Model};
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

impl Selectable for Model<SingleSelect> {
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

    #[inline]
    fn is_active(&self, id: Entity) -> bool {
        self.selection.active == id
    }
}

impl Model<SingleSelect> {
    /// Get an immutable reference to the data associated with the active item.
    #[must_use]
    #[inline]
    pub fn active_data<Data: 'static>(&self) -> Option<&Data> {
        self.data(self.active())
    }

    /// Get a mutable reference to the data associated with the active item.
    #[must_use]
    #[inline]
    pub fn active_data_mut<Data: 'static>(&mut self) -> Option<&mut Data> {
        self.data_mut(self.active())
    }

    /// Deactivates the active item.
    #[inline]
    pub fn deactivate(&mut self) {
        Selectable::deactivate(self, Entity::default());
    }

    /// The ID of the active item.
    #[must_use]
    #[inline]
    pub fn active(&self) -> Entity {
        self.selection.active
    }
}

/// [`Model<MultiSelect>`] permits multiple keys to be active at a time.
#[derive(Debug, Default)]
pub struct MultiSelect {
    pub active: HashSet<Entity>,
}

impl Selectable for Model<MultiSelect> {
    fn activate(&mut self, id: Entity) {
        if !self.items.contains_key(id) {
            return;
        }

        if !self.selection.active.insert(id) {
            self.selection.active.remove(&id);
        }
    }

    #[inline]
    fn deactivate(&mut self, id: Entity) {
        self.selection.active.remove(&id);
    }

    #[inline]
    fn is_active(&self, id: Entity) -> bool {
        self.selection.active.contains(&id)
    }
}

impl Model<MultiSelect> {
    /// Deactivates the item in the model.
    #[inline]
    pub fn deactivate(&mut self, id: Entity) {
        Selectable::deactivate(self, id);
    }

    /// The IDs of the active items.
    #[inline]
    pub fn active(&self) -> impl Iterator<Item = Entity> + '_ {
        self.selection.active.iter().copied()
    }
}
