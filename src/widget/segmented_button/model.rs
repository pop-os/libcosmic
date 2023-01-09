// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::selection_modes::{MultiSelect, Selectable, SingleSelect};
use super::SegmentedItem;
use slotmap::{SecondaryMap, SlotMap};

slotmap::new_key_type! {
    /// A unique ID for a segmented button
    pub struct Key;
}

/// A model for single-select button selection.
pub type SingleSelectModel<Component> = Model<SingleSelect, Component>;

/// A model for multi-select button selection.
pub type MultiSelectModel<Component> = Model<MultiSelect, Component>;

/// The model held by the application, containing the unique IDs of each item and their respective contents.
#[derive(Default)]
pub struct Model<SelectionMode, Component> {
    pub(super) widget: WidgetModel<SelectionMode>,
    pub(super) app: AppModel<Component>,
}

/// The portion of the model used only by the application.
pub struct AppModel<Component>(SecondaryMap<Key, Component>);

impl<Component> Default for AppModel<Component> {
    fn default() -> Self {
        Self(SecondaryMap::default())
    }
}

/// The portion of the model useful to the widget.
#[derive(Default)]
pub struct WidgetModel<SelectionMode> {
    /// The content used for drawing segmented items.
    pub(super) items: SlotMap<Key, SegmentedItem>,

    /// Manages selections
    pub(super) selection: SelectionMode,
}

impl<Component> Model<SingleSelect, Component> {
    pub fn activate(&mut self, key: Key) {
        self.widget.selection.active = key;
    }

    #[must_use]
    pub fn active_component(&self) -> Option<&Component> {
        self.component(self.active())
    }

    #[must_use]
    pub fn active_component_mut(&mut self) -> Option<&mut Component> {
        self.component_mut(self.active())
    }

    pub fn deactivate(&mut self) {
        self.widget.selection.active = Key::default();
    }

    /// The ID of the active button.
    #[must_use]
    pub fn active(&self) -> Key {
        self.widget.selection.active
    }
}

impl<Component> Model<MultiSelect, Component> {
    pub fn activate(&mut self, key: Key) {
        if !self.widget.selection.active.insert(key) {
            self.widget.selection.active.remove(&key);
        }
    }

    pub fn deactivate(&mut self, key: Key) {
        self.widget.selection.active.remove(&key);
    }

    /// The IDs of the active items.
    pub fn active(&self) -> impl Iterator<Item = Key> + '_ {
        self.widget.selection.active.iter().copied()
    }
}

impl<SelectionMode, Component> Model<SelectionMode, Component>
where
    SelectionMode: Selectable,
{
    #[must_use]
    pub fn builder() -> ModelBuilder<SelectionMode, Component> {
        ModelBuilder(Self {
            widget: WidgetModel::default(),
            app: AppModel::default(),
        })
    }

    /// Convenience method for batching multiple operations
    #[must_use]
    pub fn batch(&mut self) -> Batch<SelectionMode, Component> {
        Batch(self)
    }

    /// Enables or disables a button
    #[must_use]
    pub fn content(&self, key: Key) -> Option<&SegmentedItem> {
        self.widget.items.get(key)
    }

    /// Enables or disables a button
    #[must_use]
    pub fn content_mut(&mut self, key: Key) -> Option<&mut SegmentedItem> {
        self.widget.items.get_mut(key)
    }

    pub fn component(&self, key: Key) -> Option<&Component> {
        self.app.0.get(key)
    }

    pub fn component_mut(&mut self, key: Key) -> Option<&mut Component> {
        self.app.0.get_mut(key)
    }

    /// Insert a new button.
    pub fn insert(&mut self, content: impl Into<SegmentedItem>, component: Component) -> Key {
        let key = self.widget.items.insert(content.into());
        self.app.0.insert(key, component);
        key
    }

    /// Inserts and activates a button.
    pub fn insert_active(
        &mut self,
        content: impl Into<SegmentedItem>,
        component: Component,
    ) -> Key {
        let key = self.insert(content, component);
        self.widget.selection.activate(key);
        key
    }

    #[must_use]
    pub fn is_active(&self, key: Key) -> bool {
        self.widget.selection.is_active(key)
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) {
        self.widget.items.remove(key);
        self.widget.selection.deactivate(key);
    }
}

pub struct ModelBuilder<SelectionMode, Component>(Model<SelectionMode, Component>);

impl<SelectionMode: Selectable, Component> ModelBuilder<SelectionMode, Component> {
    #[must_use]
    pub fn insert(mut self, content: impl Into<SegmentedItem>, component: Component) -> Self {
        self.0.insert(content, component);
        self
    }

    #[must_use]
    pub fn insert_active(
        mut self,
        content: impl Into<SegmentedItem>,
        component: Component,
    ) -> Self {
        self.0.insert_active(content, component);
        self
    }

    pub fn build(self) -> Model<SelectionMode, Component> {
        self.0
    }
}

/// Convenience type for batching multiple operations
pub struct Batch<'a, SelectionMode, Component>(&'a mut Model<SelectionMode, Component>);

impl<'a, SelectionMode: Selectable, Component> Batch<'a, SelectionMode, Component> {
    /// Insert a new button.
    #[must_use]
    pub fn insert(self, content: impl Into<SegmentedItem>, component: Component) -> Self {
        self.0.insert(content, component);
        self
    }

    /// Inserts and activates a button.
    #[must_use]
    pub fn insert_active(self, content: impl Into<SegmentedItem>, component: Component) -> Self {
        self.0.insert_active(content, component);
        self
    }

    /// Removes a button.
    pub fn remove(&mut self, key: Key) {
        self.0.remove(key);
    }
}
