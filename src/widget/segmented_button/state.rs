/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use slotmap::{SecondaryMap, SlotMap};

slotmap::new_key_type! {
    pub struct Key;
}

/// Contains all state for interacting with a [`SegmentedButton`].
pub struct State<Data> {
    pub inner: WidgetState,
    pub data: SecondaryState<Data>,
}

impl<Data> Default for State<Data> {
    fn default() -> Self {
        Self {
            inner: WidgetState::default(),
            data: SecondaryState::default(),
        }
    }
}

/// State which is most useful to the widget.
#[derive(Default)]
pub struct WidgetState {
    pub buttons: SlotMap<Key, ButtonContent>,
    pub active: Key,
}

/// State which is most useful to the application.
pub type SecondaryState<Data> = SecondaryMap<Key, Data>;

impl<Data> State<Data> {
    /// The ID of the active button.
    #[must_use]
    pub fn active(&self) -> Key {
        self.inner.active
    }

    /// Get the application data for the active button.
    #[must_use]
    pub fn active_data(&self) -> Option<&Data> {
        self.data(self.active())
    }

    /// Get the application data for a button.
    #[must_use]
    pub fn data(&self, key: Key) -> Option<&Data> {
        self.data.get(key)
    }

    /// Insert a new button.
    pub fn insert(&mut self, content: impl Into<ButtonContent>, data: Data) -> Key {
        let key = self.inner.buttons.insert(content.into());
        self.data.insert(key, data);
        key
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) -> Option<Data> {
        self.inner.buttons.remove(key);
        self.data.remove(key)
    }

    /// Activates this button.
    pub fn activate(&mut self, key: Key) {
        self.inner.active = key;
    }
}

/// Data to be drawn in a [`SegmentedButton`] button.
pub struct ButtonContent {
    pub text: String,
}

impl From<String> for ButtonContent {
    fn from(text: String) -> Self {
        ButtonContent { text }
    }
}
