//! A widget allowing the user to display tables of information with optional sorting by category
//!

pub mod model;
pub use model::{
    category::ItemCategory,
    category::ItemInterface,
    selection::{MultiSelect, SingleSelect},
    Entity, Model,
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

pub fn table<'a, SelectionMode, Item, Category, Message>(
    model: &'a Model<SelectionMode, Item, Category>,
) -> TableView<'a, SelectionMode, Item, Category, Message>
where
    Message: Clone,
    SelectionMode: Default,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: model::selection::Selectable,
{
    TableView::new(model)
}

pub fn compact_table<'a, SelectionMode, Item, Category, Message>(
    model: &'a Model<SelectionMode, Item, Category>,
) -> CompactTableView<'a, SelectionMode, Item, Category, Message>
where
    Message: Clone,
    SelectionMode: Default,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: model::selection::Selectable,
{
    CompactTableView::new(model)
}
