pub mod category;
pub mod entity;
pub mod selection;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
};

use category::{ItemCategory, ItemInterface};
use entity::EntityMut;
use selection::Selectable;
use slotmap::{SecondaryMap, SlotMap};

slotmap::new_key_type! {
    /// Unique key type for items in the table
    pub struct Entity;
}

/// The portion of the model used only by the application.
#[derive(Debug, Default)]
pub(super) struct Storage(HashMap<TypeId, SecondaryMap<Entity, Box<dyn Any>>>);

pub struct Model<SelectionMode: Default, Item: ItemInterface<Category>, Category: ItemCategory>
where
    Category: ItemCategory,
{
    pub(super) categories: Vec<Category>,

    /// Stores the items
    pub(super) items: SlotMap<Entity, Item>,

    /// Whether the item is selected or not
    pub(super) active: SecondaryMap<Entity, bool>,

    /// Optional indents for the table items
    pub(super) indents: SecondaryMap<Entity, u16>,

    /// Order which the items will be displayed.
    pub(super) order: VecDeque<Entity>,

    /// Stores the current selection(s)
    pub(super) selection: SelectionMode,

    /// What category to sort by and whether it's ascending or not
    pub(super) sort: Option<(Category, bool)>,

    /// Application-managed data associated with each item
    pub(super) storage: Storage,
}

impl<SelectionMode: Default, Item: ItemInterface<Category>, Category: ItemCategory>
    Model<SelectionMode, Item, Category>
where
    Self: Selectable,
{
    pub fn new(categories: Vec<Category>) -> Self {
        Self {
            categories,
            items: SlotMap::default(),
            active: SecondaryMap::default(),
            indents: SecondaryMap::default(),
            order: VecDeque::new(),
            selection: SelectionMode::default(),
            sort: None,
            storage: Storage::default(),
        }
    }

    pub fn categories(&mut self, cats: Vec<Category>) {
        self.categories = cats;
    }

    /// Activates the item in the model.
    ///
    /// ```ignore
    /// model.activate(id);
    /// ```
    pub fn activate(&mut self, id: Entity) {
        Selectable::activate(self, id);
    }

    /// Activates the item at the given position, returning true if it was activated.
    pub fn activate_position(&mut self, position: u16) -> bool {
        if let Some(entity) = self.entity_at(position) {
            self.activate(entity);
            return true;
        }

        false
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
    pub fn item(&self, id: Entity) -> Option<&Item> {
        self.items.get(id)
    }

    /// Get a mutable reference to data associated with an item.
    pub fn item_mut(&mut self, id: Entity) -> Option<&mut Item> {
        self.items.get_mut(id)
    }

    /// Associates data with the item.
    ///
    /// There may only be one data component per Rust type.
    ///
    /// ```ignore
    /// model.data_set::<String>(id, String::from("custom string"));
    /// ```
    pub fn item_set(&mut self, id: Entity, data: Item) {
        if let Some(item) = self.items.get_mut(id) {
            *item = data;
        }
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

    /// Enable or disable an item.
    ///
    /// ```ignore
    /// model.enable(id, true);
    /// ```
    pub fn enable(&mut self, id: Entity, enable: bool) {
        if let Some(e) = self.active.get_mut(id) {
            *e = enable;
        }
    }

    /// Get the item that is located at a given position.
    #[must_use]
    pub fn entity_at(&mut self, position: u16) -> Option<Entity> {
        self.order.get(position as usize).copied()
    }

    /// Inserts a new item in the model.
    ///
    /// ```ignore
    /// let id = model.insert().text("Item A").icon("custom-icon").id();
    /// ```
    #[must_use]
    pub fn insert(&mut self, item: Item) -> EntityMut<'_, SelectionMode, Item, Category> {
        let id = self.items.insert(item);
        self.order.push_back(id);
        EntityMut { model: self, id }
    }

    /// Check if the given ID is the active ID.
    #[must_use]
    pub fn is_active(&self, id: Entity) -> bool {
        <Self as Selectable>::is_active(self, id)
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
    pub fn is_enabled(&self, id: Entity) -> bool {
        self.active.get(id).is_some_and(|e| *e)
    }

    /// Iterates across items in the model in the order that they are displayed.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.order.iter().copied()
    }

    pub fn indent(&self, id: Entity) -> Option<u16> {
        self.indents.get(id).copied()
    }

    pub fn indent_set(&mut self, id: Entity, indent: u16) -> Option<u16> {
        if !self.contains_item(id) {
            return None;
        }

        self.indents.insert(id, indent)
    }

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

    /// Get the sort data
    pub fn get_sort(&self) -> Option<(Category, bool)> {
        self.sort
    }

    /// Sorts items in the model, this should be called before it is drawn after all items have been added for the view
    pub fn sort(&mut self, category: Category, ascending: bool) {
        match self.sort {
            Some((cat, asc)) if cat == category && asc == ascending => return,
            Some((cat, _)) if cat == category => self.order.make_contiguous().reverse(),
            _ => {
                self.order.make_contiguous().sort_by(|entity_a, entity_b| {
                    let cmp = self
                        .items
                        .get(*entity_a)
                        .unwrap()
                        .compare(self.items.get(*entity_b).unwrap(), category);
                    if ascending { cmp } else { cmp.reverse() }
                });
            }
        }
        self.sort = Some((category, ascending));
    }
}
