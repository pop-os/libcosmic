//! A widget allowing the user to display tables of information with optional sorting by category
//!

pub mod model;
pub use model::{
    Entity, Model,
    category::ItemCategory,
    category::ItemInterface,
    selection::{MultiSelect, SingleSelect},
};
pub mod widget;
pub use widget::compact::CompactTableView;
pub use widget::standard::TableView;

pub type SingleSelectTableView<'a, Item, Category, Message> =
    TableView<'a, SingleSelect, Item, Category, Message>;
pub type SingleSelectModel<Item, Category> = Model<SingleSelect, Item, Category>;

pub type MultiSelectTableView<'a, Item, Category, Message> =
    TableView<'a, MultiSelect, Item, Category, Message>;
pub type MultiSelectModel<Item, Category> = Model<MultiSelect, Item, Category>;

pub fn table<SelectionMode, Item, Category, Message>(
    model: &Model<SelectionMode, Item, Category>,
) -> TableView<'_, SelectionMode, Item, Category, Message>
where
    Message: Clone,
    SelectionMode: Default,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: model::selection::Selectable,
{
    TableView::new(model)
}

pub fn compact_table<SelectionMode, Item, Category, Message>(
    model: &Model<SelectionMode, Item, Category>,
) -> CompactTableView<'_, SelectionMode, Item, Category, Message>
where
    Message: Clone,
    SelectionMode: Default,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: model::selection::Selectable,
{
    CompactTableView::new(model)
}
