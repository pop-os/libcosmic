// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use slotmap::{SecondaryMap, SparseSecondaryMap};

use super::{Entity, Model, Selectable};
use crate::widget::icon::Icon;
use std::borrow::Cow;

/// A builder for a [`Model`].
pub struct ModelBuilder<SelectionMode: Default, Message>(Model<SelectionMode, Message>);

//TODO: Default derive ends up requiring Message to implement Default
impl<SelectionMode: Default, Message> Default for ModelBuilder<SelectionMode, Message> {
    fn default() -> Self {
        Self(Model::default())
    }
}

/// Constructs a new item for the [`ModelBuilder`].
pub struct BuilderEntity<SelectionMode: Default, Message> {
    model: ModelBuilder<SelectionMode, Message>,
    id: Entity,
}

impl<SelectionMode: Default, Message> ModelBuilder<SelectionMode, Message>
where
    Model<SelectionMode, Message>: Selectable,
{
    /// Inserts a new item and its associated data into the model.
    #[must_use]
    pub fn insert(
        mut self,
        builder: impl Fn(BuilderEntity<SelectionMode, Message>) -> BuilderEntity<SelectionMode, Message>,
    ) -> Self {
        let id = self.0.insert().id();
        builder(BuilderEntity { model: self, id }).model
    }

    /// Consumes the builder and returns the model.
    pub fn build(self) -> Model<SelectionMode, Message> {
        self.0
    }
}

impl<SelectionMode: Default, Message> BuilderEntity<SelectionMode, Message>
where
    Model<SelectionMode, Message>: Selectable,
{
    /// Activates the newly-inserted item.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn activate(mut self) -> Self {
        self.model.0.activate(self.id);
        self
    }

    /// Defines that the close button should appear
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn closable(mut self) -> Self {
        self.model.0.closable_set(self.id, true);
        self
    }

    /// Associates extra data with an external secondary map.
    ///
    /// The secondary map internally uses a `Vec`, so should only be used for data that
    /// is commonly associated.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn secondary<Data>(self, map: &mut SecondaryMap<Entity, Data>, data: Data) -> Self {
        map.insert(self.id, data);
        self
    }

    /// Associates extra data with an external sparse secondary map.
    ///
    /// Sparse maps internally use a `HashMap`, for data that is sparsely associated.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn secondary_sparse<Data>(
        self,
        map: &mut SparseSecondaryMap<Entity, Data>,
        data: Data,
    ) -> Self {
        map.insert(self.id, data);
        self
    }

    /// Assigns extra data to the item.
    ///
    /// There can only be one data component per Rust type.
    ///
    /// ```ignore
    /// enum ViewItem { A }
    ///
    /// segmented_button::Model::builder()
    ///     .insert(|b| b.text("Item A").data(ViewItem::A))
    ///     .build()
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn data<Data: 'static>(mut self, data: Data) -> Self {
        self.model.0.data_set(self.id, data);
        self
    }

    /// Defines an icon for the item.
    ///
    /// ```ignore
    /// segmented_button::Model::builder()
    ///     .insert(|b| b.text("Item A").icon("custom-icon"))
    ///     .build()
    /// ```
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn icon(mut self, icon: Icon) -> Self {
        self.model.0.icon_set(self.id, icon);
        self
    }

    /// Define the position of the newly-inserted item.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn position(mut self, position: u16) -> Self {
        self.model.0.position_set(self.id, position);
        self
    }

    /// Swap the position with another item in the model.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn position_swap(mut self, other: Entity) -> Self {
        self.model.0.position_swap(self.id, other);
        self
    }

    /// Defines the text for the item.
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.model.0.text_set(self.id, text);
        self
    }

    /// Calls a function with the ID
    #[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
    pub fn with_id(self, func: impl FnOnce(Entity)) -> Self {
        func(self.id);
        self
    }
}
