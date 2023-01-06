// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use derive_setters::Setters;
use slotmap::{SecondaryMap, SlotMap};
use std::borrow::Cow;

use crate::widget::IconSource;

slotmap::new_key_type! {
    /// An ID for a segmented button
    pub struct Key;
}

/// Contains all state for interacting with a segmented button.
pub struct State<Data> {
    /// State that is shared with widget drawing.
    pub inner: SharedWidgetState,

    /// State unique to the application.
    pub data: SecondaryState<Data>,
}

impl<Data> Default for State<Data> {
    fn default() -> Self {
        Self {
            inner: SharedWidgetState::default(),
            data: SecondaryState::default(),
        }
    }
}

/// State which is most useful to the widget.
#[derive(Default)]
pub struct SharedWidgetState {
    /// The content used for drawing segmented buttons.
    pub buttons: SlotMap<Key, Content>,

    /// The actively-selected segmented button.
    pub active: Key,
}

/// State which is most useful to the application.
pub type SecondaryState<Data> = SecondaryMap<Key, Data>;

impl<Data> State<Data> {
    #[must_use]
    pub fn builder() -> Builder<Data> {
        Builder(Self::default())
    }

    /// Activates this button.
    pub fn activate(&mut self, key: Key) {
        self.inner.active = key;
    }

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

    /// Convenience method for batching multiple operations
    #[must_use]
    pub fn batch(&mut self) -> Batch<Data> {
        Batch(self)
    }

    /// Enables or disables a button
    #[must_use]
    pub fn content(&self, key: Key) -> Option<&Content> {
        self.inner.buttons.get(key)
    }

    /// Enables or disables a button
    #[must_use]
    pub fn content_mut(&mut self, key: Key) -> Option<&mut Content> {
        self.inner.buttons.get_mut(key)
    }

    /// Get the application data for a button.
    #[must_use]
    pub fn data(&self, key: Key) -> Option<&Data> {
        self.data.get(key)
    }

    /// Get mutable application data for a button.
    #[must_use]
    pub fn data_mut(&mut self, key: Key) -> Option<&mut Data> {
        self.data.get_mut(key)
    }

    /// Insert a new button.
    pub fn insert(&mut self, content: impl Into<Content>, data: Data) -> Key {
        let key = self.inner.buttons.insert(content.into());
        self.data.insert(key, data);
        key
    }

    /// Inserts and activates a button.
    pub fn insert_active(&mut self, content: impl Into<Content>, data: Data) -> Key {
        let key = self.insert(content, data);
        self.activate(key);
        key
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) -> Option<Data> {
        self.inner.buttons.remove(key);
        self.data.remove(key)
    }
}

/// Data to be drawn in a segmented button.
#[derive(Default, Setters)]
pub struct Content {
    #[setters(strip_option, into)]
    /// The label to display in this button.
    pub text: Option<Cow<'static, str>>,

    #[setters(strip_option, into)]
    /// An optionally-displayed icon beside the label.
    pub icon: Option<IconSource<'static>>,

    /// Whether the button is clickable or not.
    pub enabled: bool,
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Self::from(Cow::Owned(text))
    }
}

impl From<&'static str> for Content {
    fn from(text: &'static str) -> Self {
        Self::from(Cow::Borrowed(text))
    }
}

impl From<Cow<'static, str>> for Content {
    fn from(text: Cow<'static, str>) -> Self {
        Content::default().text(text)
    }
}

pub struct Builder<Data>(State<Data>);

impl<Data> Builder<Data> {
    pub fn insert(mut self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert(content, data);
        self
    }

    pub fn insert_active(mut self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert_active(content, data);
        self
    }

    pub fn build(self) -> State<Data> {
        self.0
    }
}

/// Convenience type for batching multiple operations
pub struct Batch<'a, Data>(&'a mut State<Data>);

impl<'a, Data> Batch<'a, Data> {
    /// Insert a new button.
    pub fn insert(self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert(content, data);
        self
    }

    /// Inserts and activates a button.
    pub fn insert_active(self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert_active(content, data);
        self
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) {
        self.0.remove(key);
    }
}
