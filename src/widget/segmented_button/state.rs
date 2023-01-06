// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use derive_setters::Setters;
use slotmap::{SecondaryMap, SlotMap};
use std::{borrow::Cow, collections::HashSet};

use crate::widget::IconSource;

slotmap::new_key_type! {
    /// An ID for a segmented button
    pub struct Key;
}

/// Contains all state for interacting with a segmented button.
pub struct State<Selection, Data> {
    /// State that is shared with widget drawing.
    pub inner: SharedWidgetState<Selection>,

    /// State unique to the application.
    pub data: SecondaryState<Data>,
}

impl<Selection: Default, Data> Default for State<Selection, Data> {
    fn default() -> Self {
        Self {
            inner: SharedWidgetState::default(),
            data: SecondaryState::default(),
        }
    }
}

/// State which is most useful to the widget.
#[derive(Default)]
pub struct SharedWidgetState<Variant> {
    /// The content used for drawing segmented buttons.
    pub buttons: SlotMap<Key, Content>,

    /// Manages selections
    pub selection: Variant,
}

impl<Data> State<SingleSelect, Data> {
    pub fn activate(&mut self, key: Key) {
        self.inner.selection.activate(key);
    }

    pub fn deactivate(&mut self, key: Key) {
        self.inner.selection.deactivate(key);
    }

    #[must_use]
    pub fn is_active(&self, key: Key) -> bool {
        self.inner.selection.is_active(key)
    }

    /// The ID of the active button.
    #[must_use]
    pub fn active(&self) -> Key {
        self.inner.selection.active
    }

    /// Get the application data for the active button.
    #[must_use]
    pub fn active_data(&self) -> Option<&Data> {
        self.data.get(self.inner.selection.active)
    }

    /// Mutable application data for the active button.
    #[must_use]
    pub fn active_data_mut(&mut self) -> Option<&mut Data> {
        self.data.get_mut(self.inner.selection.active)
    }
}

impl<Data> State<MultiSelect, Data> {
    pub fn activate(&mut self, key: Key) {
        if self.inner.selection.is_active(key) {
            self.inner.selection.deactivate(key);
        } else {
            self.inner.selection.activate(key);
        }
    }

    pub fn deactivate(&mut self, key: Key) {
        self.inner.selection.deactivate(key);
    }

    #[must_use]
    pub fn is_active(&self, key: Key) -> bool {
        self.inner.selection.is_active(key)
    }

    /// The IDs of the active buttons.
    pub fn active(&self) -> impl Iterator<Item = Key> + '_ {
        self.inner.selection.active.iter().copied()
    }

    /// Get the application data for the active buttons.
    pub fn active_data(&self) -> impl Iterator<Item = (Key, &Data)> {
        self.inner.buttons.keys().filter_map(|key| {
            if self.inner.selection.is_active(key) {
                self.data.get(key).map(|data| (key, data))
            } else {
                None
            }
        })
    }
}

/// State which is most useful to the application.
pub type SecondaryState<Data> = SecondaryMap<Key, Data>;

impl<Selection, Data> State<Selection, Data>
where
    Selection: Selectable,
{
    #[must_use]
    pub fn builder() -> Builder<Selection, Data> {
        Builder(Self::default())
    }

    /// Convenience method for batching multiple operations
    #[must_use]
    pub fn batch(&mut self) -> Batch<Selection, Data> {
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
        self.inner.selection.activate(key);
        key
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) -> Option<Data> {
        self.inner.buttons.remove(key);
        self.inner.selection.deactivate(key);
        self.data.remove(key)
    }
}

pub trait Selectable: Default {
    fn activate(&mut self, key: Key);

    fn deactivate(&mut self, key: Key);

    fn is_active(&self, key: Key) -> bool;
}

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

pub struct Builder<Selection, Data>(State<Selection, Data>);

impl<Selection: Selectable, Data> Builder<Selection, Data> {
    pub fn insert(mut self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert(content, data);
        self
    }

    pub fn insert_active(mut self, content: impl Into<Content>, data: Data) -> Self {
        self.0.insert_active(content, data);
        self
    }

    pub fn build(self) -> State<Selection, Data> {
        self.0
    }
}

/// Convenience type for batching multiple operations
pub struct Batch<'a, Selection, Data>(&'a mut State<Selection, Data>);

impl<'a, Selection: Selectable, Data> Batch<'a, Selection, Data> {
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
