// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod builder;
pub use self::builder::{BuilderEntity, ModelBuilder};

mod entity;
pub use self::entity::EntityMut;

mod selection;
pub use self::selection::{MultiSelect, Selectable, SingleSelect};

use crate::widget::IconSource;
use slotmap::{SecondaryMap, SlotMap};
use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

slotmap::new_key_type! {
    /// A unique ID for an item in the [`Model`].
    pub struct Entity;
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// A model for single-select button selection.
pub type SingleSelectModel = Model<SingleSelect>;

/// Single-select variant of an [`EntityMut`].
pub type SingleSelectEntityMut<'a> = EntityMut<'a, SingleSelect>;

/// A model for multi-select button selection.
pub type MultiSelectModel = Model<MultiSelect>;

/// Multi-select variant of an [`EntityMut`].
pub type MultiSelectEntityMut<'a> = EntityMut<'a, MultiSelect>;

/// The portion of the model used only by the application.
#[derive(Debug, Default)]
pub(super) struct Storage(HashMap<TypeId, SecondaryMap<Entity, Box<dyn Any>>>);

/// The model held by the application, containing the unique IDs and data of each inserted item.
#[derive(Default, Debug)]
pub struct Model<SelectionMode: Default> {
    /// The content used for drawing segmented items.
    pub(super) items: SlotMap<Entity, Settings>,

    /// Icons optionally-defined for each item.
    pub(super) icons: SecondaryMap<Entity, IconSource<'static>>,

    /// Text optionally-defined for each item.
    pub(super) text: SecondaryMap<Entity, Cow<'static, str>>,

    /// Order which the items will be displayed.
    pub(super) order: VecDeque<Entity>,

    /// Manages selections
    pub(super) selection: SelectionMode,

    /// Data managed by the application.
    pub(super) storage: Storage,
}

impl<SelectionMode: Default> Model<SelectionMode>
where
    Self: Selectable,
{
    /// Activates the item in the model.
    ///
    /// ```ignore
    /// model.activate(id);
    /// ```
    pub fn activate(&mut self, id: Entity) {
        Selectable::activate(self, id);
    }

    /// Creates a builder for initializing a model.
    ///
    /// ```ignore
    /// let model = segmented_button::Model::builder()
    ///     .insert(|b| b.text("Item A").activate())
    ///     .insert(|b| b.text("Item B"))
    ///     .insert(|b| b.text("Item C"))
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> ModelBuilder<SelectionMode> {
        ModelBuilder::default()
    }

    /// Removes all items from the model.
    ///
    /// Any IDs held elsewhere by the application will no longer be usable with the map.
    /// The generation is incremented on removal, so the stale IDs will return `None` for
    /// any attempt to get values from the map.
    ///
    /// ```ignore
    /// model.clear();
    /// ```
    pub fn clear(&mut self) {
        for entity in self.order.clone() {
            self.remove(entity);
        }
    }

    /// Check if an item exists in the map.
    ///
    /// ```ignore
    /// if model.contains_item(id) {
    ///     println!("ID is still valid");
    /// }
    /// ```
    pub fn contains_item(&self, id: Entity) -> bool {
        self.items.contains_key(id)
    }

    /// Get an immutable reference to data associated with an item.
    ///
    /// ```ignore
    /// if let Some(data) = model.data::<String>(id) {
    ///     println!("found string on {:?}: {}", id, data);
    /// }
    /// ```
    pub fn data<Data: 'static>(&self, id: Entity) -> Option<&Data> {
        self.storage
            .0
            .get(&TypeId::of::<Data>())
            .and_then(|storage| storage.get(id))
            .and_then(|data| data.downcast_ref())
    }

    /// Get a mutable reference to data associated with an item.
    pub fn data_mut<Data: 'static>(&mut self, id: Entity) -> Option<&mut Data> {
        self.storage
            .0
            .get_mut(&TypeId::of::<Data>())
            .and_then(|storage| storage.get_mut(id))
            .and_then(|data| data.downcast_mut())
    }

    /// Associates data with the item.
    ///
    /// There may only be one data component per Rust type.
    ///
    /// ```ignore
    /// model.data_set::<String>(id, String::from("custom string"));
    /// ```
    pub fn data_set<Data: 'static>(&mut self, id: Entity, data: Data) {
        if self.contains_item(id) {
            self.storage
                .0
                .entry(TypeId::of::<Data>())
                .or_insert_with(SecondaryMap::new)
                .insert(id, Box::new(data));
        }
    }

    /// Removes a specific data type from the item.
    ///
    /// ```ignore
    /// model.data.remove::<String>(id);
    /// ```
    pub fn data_remove<Data: 'static>(&mut self, id: Entity) {
        self.storage
            .0
            .get_mut(&TypeId::of::<Data>())
            .and_then(|storage| storage.remove(id));
    }

    /// Enable or disable an item.
    ///
    /// ```ignore
    /// model.enable(id, true);
    /// ```
    pub fn enable(&mut self, id: Entity, enable: bool) {
        if let Some(e) = self.items.get_mut(id) {
            e.enabled = enable;
        }
    }

    /// Immutable reference to the icon associated with the item.
    ///
    /// ```ignore
    /// if let Some(icon) = model.icon(id) {
    ///     println!("has icon: {:?}", icon);
    /// }
    /// ```
    pub fn icon(&self, id: Entity) -> Option<&IconSource<'static>> {
        self.icons.get(id)
    }

    /// Sets a new icon for an item.
    ///
    /// ```ignore
    /// if let Some(old_icon) = model.icon_set(IconSource::Name("new-icon".into())) {
    ///     println!("previously had icon: {:?}", old_icon);
    /// }
    /// ```
    pub fn icon_set(
        &mut self,
        id: Entity,
        icon: impl Into<IconSource<'static>>,
    ) -> Option<IconSource<'static>> {
        if !self.contains_item(id) {
            return None;
        }

        self.icons.insert(id, icon.into())
    }

    /// Removes the icon from an item.
    ///
    /// ```ignore
    /// if let Some(old_icon) = model.icon_remove(id) {
    ///     println!("previously had icon: {:?}", old_icon);
    /// }
    pub fn icon_remove(&mut self, id: Entity) -> Option<IconSource<'static>> {
        self.icons.remove(id)
    }

    /// Inserts a new item in the model.
    ///
    /// ```ignore
    /// let id = model.insert().text("Item A").icon("custom-icon").id();
    /// ```
    #[must_use]
    pub fn insert(&mut self) -> EntityMut<SelectionMode> {
        let id = self.items.insert(Settings::default());
        self.order.push_back(id);
        EntityMut { model: self, id }
    }

    /// Check if the item is enabled.
    ///
    /// ```ignore
    ///
    /// if model.is_enabled(id) {
    ///     if let Some(text) = model.text(id) {
    ///         println!("{text} is enabled");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn is_enabled(&self, id: Entity) -> bool {
        self.items.get(id).map_or(false, |e| e.enabled)
    }

    /// The position of the item in the model.
    ///
    /// ```ignore
    /// if let Some(position) = model.position(id) {
    ///     println!("found item at {}", position);
    /// }
    pub fn position(&self, id: Entity) -> Option<usize> {
        self.order.iter().position(|k| *k == id)
    }

    /// Change the position of an item in the model.
    ///
    /// ```ignore
    /// if let Some(new_position) = model.position_set(id, 0) {
    ///     println!("placed item at {}", new_position);
    /// }
    /// ```
    pub fn position_set(&mut self, id: Entity, position: u16) -> Option<usize> {
        let Some(index) = self.position(id) else {
            return None
        };

        let position = self.order.len().min(position as usize);

        self.order.remove(index);
        self.order.insert(position, id);
        Some(position)
    }

    /// Swap the position of two items in the model.
    ///
    /// Returns false if the swap cannot be performed.
    ///
    /// ```ignore
    /// if model.position_swap(first_id, second_id) {
    ///     println!("positions swapped");
    /// }
    /// ```
    pub fn position_swap(&mut self, first: Entity, second: Entity) -> bool {
        let Some(first_index) = self.position(first) else {
            return false
        };

        let Some(second_index) = self.position(second) else {
            return false
        };

        self.order.swap(first_index, second_index);
        true
    }

    /// Removes an item from the model.
    ///
    /// The generation of the slot for the ID will be incremented, so this ID will no
    /// longer be usable with the map. Subsequent attempts to get values from the map
    /// with this ID will return `None` and failed to assign values.
    pub fn remove(&mut self, id: Entity) {
        self.items.remove(id);
        self.deactivate(id);

        for storage in self.storage.0.values_mut() {
            storage.remove(id);
        }

        if let Some(index) = self.position(id) {
            self.order.remove(index);
        }
    }

    /// Immutable reference to the text assigned to the item.
    ///
    /// ```ignore
    /// if let Some(text) = model.text(id) {
    ///     println!("{:?} has text {text}", id);
    /// }
    /// ```
    pub fn text(&self, id: Entity) -> Option<&str> {
        self.text.get(id).map(Cow::as_ref)
    }

    /// Sets new text for an item.
    ///
    /// ```ignore
    /// if let Some(old_text) = model.text_set(id, "Item B") {
    ///     println!("{:?} had text {}", id, old_text)
    /// }
    /// ```
    pub fn text_set(&mut self, id: Entity, text: impl Into<Cow<'static, str>>) -> Option<Cow<str>> {
        if !self.contains_item(id) {
            return None;
        }

        self.text.insert(id, text.into())
    }

    /// Removes text from an item.
    /// ```ignore
    /// if let Some(old_text) = model.text_remove(id) {
    ///     println!("{:?} had text {}", id, old_text);
    /// }
    pub fn text_remove(&mut self, id: Entity) -> Option<Cow<'static, str>> {
        self.text.remove(id)
    }
}
