// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod builder;
pub use self::builder::{BuilderEntity, ModelBuilder};

mod entity;
pub use self::entity::EntityMut;

mod selection;
pub use self::selection::{MultiSelect, Selectable, SingleSelect};

use crate::widget::Icon;
use crate::widget::segmented_button::InsertPosition;
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
    pub closable: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            closable: false,
        }
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
#[derive(Default)]
pub struct Model<SelectionMode: Default> {
    /// The content used for drawing segmented items.
    pub(super) items: SlotMap<Entity, Settings>,

    /// Divider optionally-defined for each item.
    pub(super) divider_aboves: SecondaryMap<Entity, bool>,

    /// Icons optionally-defined for each item.
    pub(super) icons: SecondaryMap<Entity, Icon>,

    /// Indent optionally-defined for each item.
    pub(super) indents: SecondaryMap<Entity, u16>,

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
    #[inline]
    pub fn activate(&mut self, id: Entity) {
        Selectable::activate(self, id);
    }

    /// Activates the item at the given position, returning true if it was activated.
    #[inline]
    pub fn activate_position(&mut self, position: u16) -> bool {
        if let Some(entity) = self.entity_at(position) {
            self.activate(entity);
            return true;
        }

        false
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
    #[inline]
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
    #[inline]
    pub fn clear(&mut self) {
        for entity in self.order.clone() {
            self.remove(entity);
        }
    }

    /// Shows or hides the item's close button.
    #[inline]
    pub fn closable_set(&mut self, id: Entity, closable: bool) {
        if let Some(settings) = self.items.get_mut(id) {
            settings.closable = closable;
        }
    }

    /// Check if an item exists in the map.
    ///
    /// ```ignore
    /// if model.contains_item(id) {
    ///     println!("ID is still valid");
    /// }
    /// ```
    #[inline]
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
                .or_default()
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

    #[inline]
    pub fn divider_above(&self, id: Entity) -> Option<bool> {
        self.divider_aboves.get(id).copied()
    }

    pub fn divider_above_set(&mut self, id: Entity, divider_above: bool) -> Option<bool> {
        if !self.contains_item(id) {
            return None;
        }

        self.divider_aboves.insert(id, divider_above)
    }

    #[inline]
    pub fn divider_above_remove(&mut self, id: Entity) -> Option<bool> {
        self.divider_aboves.remove(id)
    }

    /// Enable or disable an item.
    ///
    /// ```ignore
    /// model.enable(id, true);
    /// ```
    #[inline]
    pub fn enable(&mut self, id: Entity, enable: bool) {
        if let Some(e) = self.items.get_mut(id) {
            e.enabled = enable;
        }
    }

    /// Get the item that is located at a given position.
    #[must_use]
    #[inline]
    pub fn entity_at(&mut self, position: u16) -> Option<Entity> {
        self.order.get(position as usize).copied()
    }

    /// Immutable reference to the icon associated with the item.
    ///
    /// ```ignore
    /// if let Some(icon) = model.icon(id) {
    ///     println!("has icon: {:?}", icon);
    /// }
    /// ```
    #[inline]
    pub fn icon(&self, id: Entity) -> Option<&Icon> {
        self.icons.get(id)
    }

    /// Sets a new icon for an item.
    ///
    /// ```ignore
    /// if let Some(old_icon) = model.icon_set(IconSource::from("new-icon")) {
    ///     println!("previously had icon: {:?}", old_icon);
    /// }
    /// ```
    #[inline]
    pub fn icon_set(&mut self, id: Entity, icon: Icon) -> Option<Icon> {
        if !self.contains_item(id) {
            return None;
        }

        self.icons.insert(id, icon)
    }

    /// Removes the icon from an item.
    ///
    /// ```ignore
    /// if let Some(old_icon) = model.icon_remove(id) {
    ///     println!("previously had icon: {:?}", old_icon);
    /// }
    #[inline]
    pub fn icon_remove(&mut self, id: Entity) -> Option<Icon> {
        self.icons.remove(id)
    }

    /// Inserts a new item in the model.
    ///
    /// ```ignore
    /// let id = model.insert().text("Item A").icon("custom-icon").id();
    /// ```
    #[must_use]
    #[inline]
    pub fn insert(&mut self) -> EntityMut<'_, SelectionMode> {
        let id = self.items.insert(Settings::default());
        self.order.push_back(id);
        EntityMut { model: self, id }
    }

    /// Check if the given ID is the active ID.
    #[must_use]
    #[inline]
    pub fn is_active(&self, id: Entity) -> bool {
        <Self as Selectable>::is_active(self, id)
    }

    /// Whether the item should contain a close button.
    #[must_use]
    #[inline]
    pub fn is_closable(&self, id: Entity) -> bool {
        self.items.get(id).map(|e| e.closable).unwrap_or_default()
    }

    /// Check if the item is enabled.
    ///
    /// ```ignore
    /// if model.is_enabled(id) {
    ///     if let Some(text) = model.text(id) {
    ///         println!("{text} is enabled");
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub fn is_enabled(&self, id: Entity) -> bool {
        self.items.get(id).map(|e| e.enabled).unwrap_or_default()
    }

    /// Get number of items in the model.
    #[inline]
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Iterates across items in the model in the order that they are displayed.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.order.iter().copied()
    }

    #[inline]
    pub fn indent(&self, id: Entity) -> Option<u16> {
        self.indents.get(id).copied()
    }

    #[inline]
    pub fn indent_set(&mut self, id: Entity, indent: u16) -> Option<u16> {
        if !self.contains_item(id) {
            return None;
        }

        self.indents.insert(id, indent)
    }

    #[inline]
    pub fn indent_remove(&mut self, id: Entity) -> Option<u16> {
        self.indents.remove(id)
    }

    /// The position of the item in the model.
    ///
    /// ```ignore
    /// if let Some(position) = model.position(id) {
    ///     println!("found item at {}", position);
    /// }
    #[must_use]
    #[inline]
    pub fn position(&self, id: Entity) -> Option<u16> {
        #[allow(clippy::cast_possible_truncation)]
        self.order.iter().position(|k| *k == id).map(|v| v as u16)
    }

    /// Change the position of an item in the model.
    ///
    /// ```ignore
    /// if let Some(new_position) = model.position_set(id, 0) {
    ///     println!("placed item at {}", new_position);
    /// }
    /// ```
    pub fn position_set(&mut self, id: Entity, position: u16) -> Option<usize> {
        let index = self.position(id)?;

        self.order.remove(index as usize);

        let position = self.order.len().min(position as usize);

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
            return false;
        };

        let Some(second_index) = self.position(second) else {
            return false;
        };

        self.order.swap(first_index as usize, second_index as usize);
        true
    }

    /// Reorder `dragged` relative to `target` based on the provided position.
    ///
    /// Returns `true` if the model changed, or `false` if the move was invalid.
    pub fn reorder(&mut self, dragged: Entity, target: Entity, position: InsertPosition) -> bool {
        if !self.contains_item(dragged) || !self.contains_item(target) || dragged == target {
            return false;
        }

        let len = self.iter().count();
        let target_pos = self.position(target).map(|pos| pos as usize).unwrap_or(len);
        let from_pos = self
            .position(dragged)
            .map(|pos| pos as usize)
            .unwrap_or(target_pos);
        let mut insert_pos = match position {
            InsertPosition::Before => target_pos,
            InsertPosition::After => target_pos.saturating_add(1),
        };
        if from_pos < insert_pos {
            insert_pos = insert_pos.saturating_sub(1);
        }
        if len > 0 {
            insert_pos = insert_pos.min(len.saturating_sub(1));
        }

        self.position_set(dragged, insert_pos as u16);
        self.activate(dragged);
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
            self.order.remove(index as usize);
        }
    }

    /// Immutable reference to the text assigned to the item.
    ///
    /// ```ignore
    /// if let Some(text) = model.text(id) {
    ///     println!("{:?} has text {text}", id);
    /// }
    /// ```
    #[inline]
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
    pub fn text_set(
        &mut self,
        id: Entity,
        text: impl Into<Cow<'static, str>>,
    ) -> Option<Cow<'_, str>> {
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
    #[inline]
    pub fn text_remove(&mut self, id: Entity) -> Option<Cow<'static, str>> {
        self.text.remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model() -> (Model<SingleSelect>, Vec<Entity>) {
        let mut ids = Vec::new();
        let model = Model::builder()
            .insert(|b| b.text("Tab1").with_id(|id| ids.push(id)))
            .insert(|b| b.text("Tab2").with_id(|id| ids.push(id)))
            .insert(|b| b.text("Tab3").with_id(|id| ids.push(id)))
            .insert(|b| b.text("Tab4").with_id(|id| ids.push(id)))
            .build();
        (model, ids)
    }

    fn order_of(model: &Model<SingleSelect>) -> Vec<Entity> {
        model.iter().collect()
    }

    #[test]
    fn reorder_inserts_before_target() {
        let (mut model, ids) = sample_model();
        assert!(model.reorder(ids[3], ids[1], InsertPosition::Before));
        assert_eq!(order_of(&model), vec![ids[0], ids[3], ids[1], ids[2]]);
    }

    #[test]
    fn reorder_inserts_after_target() {
        let (mut model, ids) = sample_model();
        assert!(model.reorder(ids[0], ids[2], InsertPosition::After));
        assert_eq!(order_of(&model), vec![ids[1], ids[2], ids[0], ids[3]]);
    }

    #[test]
    fn reorder_rejects_invalid_entities() {
        let (mut model, ids) = sample_model();
        assert!(!model.reorder(ids[0], ids[0], InsertPosition::After));
    }
}
