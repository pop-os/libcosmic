// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use slotmap::{SecondaryMap, SparseSecondaryMap};

use super::{
    Entity, Model, Selectable,
    category::{ItemCategory, ItemInterface},
};

/// A newly-inserted item which may have additional actions applied to it.
pub struct EntityMut<
    'a,
    SelectionMode: Default,
    Item: ItemInterface<Category>,
    Category: ItemCategory,
> {
    pub(super) id: Entity,
    pub(super) model: &'a mut Model<SelectionMode, Item, Category>,
}

impl<'a, SelectionMode: Default, Item: ItemInterface<Category>, Category: ItemCategory>
    EntityMut<'a, SelectionMode, Item, Category>
where
    Model<SelectionMode, Item, Category>: Selectable,
{
    /// Activates the newly-inserted item.
    ///
    /// ```ignore
    /// model.insert().text("Item A").activate();
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn activate(self) -> Self {
        self.model.activate(self.id);
        self
    }

    /// Associates extra data with an external secondary map.
    ///
    /// The secondary map internally uses a `Vec`, so should only be used for data that
    /// is commonly associated.
    ///
    /// ```ignore
    /// let mut secondary_data = segmented_button::SecondaryMap::default();
    /// model.insert().text("Item A").secondary(&mut secondary_data, String::new("custom data"));
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn secondary<Data>(self, map: &mut SecondaryMap<Entity, Data>, data: Data) -> Self {
        map.insert(self.id, data);
        self
    }

    /// Associates extra data with an external sparse secondary map.
    ///
    /// Sparse maps internally use a `HashMap`, for data that is sparsely associated.
    ///
    /// ```ignore
    /// let mut secondary_data = segmented_button::SparseSecondaryMap::default();
    /// model.insert().text("Item A").secondary(&mut secondary_data, String::new("custom data"));
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn secondary_sparse<Data>(
        self,
        map: &mut SparseSecondaryMap<Entity, Data>,
        data: Data,
    ) -> Self {
        map.insert(self.id, data);
        self
    }

    /// Associates data with the item.
    ///
    /// There may only be one data component per Rust type.
    ///
    /// ```ignore
    /// model.insert().text("Item A").data(String::from("custom string"));
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn data<Data: 'static>(self, data: Data) -> Self {
        self.model.data_set(self.id, data);
        self
    }

    /// Returns the ID of the item that was inserted.
    ///
    /// ```ignore
    /// let id = model.insert("Item A").id();
    /// ```
    #[must_use]
    pub fn id(self) -> Entity {
        self.id
    }

    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn indent(self, indent: u16) -> Self {
        self.model.indent_set(self.id, indent);
        self
    }

    /// Define the position of the item.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn position(self, position: u16) -> Self {
        self.model.position_set(self.id, position);
        self
    }

    /// Swap the position with another item in the model.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn position_swap(self, other: Entity) -> Self {
        self.model.position_swap(self.id, other);
        self
    }

    /// Defines the text for the item.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn item(self, item: Item) -> Self {
        self.model.item_set(self.id, item);
        self
    }

    /// Calls a function with the ID without consuming the wrapper.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn with_id(self, func: impl FnOnce(Entity)) -> Self {
        func(self.id);
        self
    }
}
