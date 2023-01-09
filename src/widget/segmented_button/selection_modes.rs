// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Describes logic specific to the single-select and multi-select modes of a model.

use super::Key;
use std::collections::HashSet;

/// Describes a type that has selectable components.
pub trait Selectable: Default {
    /// Activate a component.
    fn activate(&mut self, key: Key);

    /// Deactivate a component.
    fn deactivate(&mut self, key: Key);

    /// Checks if the component is active.
    fn is_active(&self, key: Key) -> bool;
}

/// Ensures that only one key may be selected.
#[derive(Default)]
pub struct SingleSelect {
    pub active: Key,
}

impl Selectable for SingleSelect {
    fn activate(&mut self, key: Key) {
        self.active = key;
    }

    fn deactivate(&mut self, _key: Key) {
        self.active = Key::default();
    }

    fn is_active(&self, key: Key) -> bool {
        self.active == key
    }
}

/// Permits multiple keys to be active at a time.
#[derive(Default)]
pub struct MultiSelect {
    pub active: HashSet<Key>,
}

impl Selectable for MultiSelect {
    fn activate(&mut self, key: Key) {
        self.active.insert(key);
    }

    fn deactivate(&mut self, key: Key) {
        self.active.remove(&key);
    }

    fn is_active(&self, key: Key) -> bool {
        self.active.contains(&key)
    }
}
