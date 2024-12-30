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
pub use widget::TableView;

pub type SingleSelectTableView<'a, Item, Category, Message> =
    TableView<'a, SingleSelect, Item, Category, Message>;
pub type SingleSelectModel<Item, Category> = Model<SingleSelect, Item, Category>;

pub type MultiSelectTableView<'a, Item, Category, Message> =
    TableView<'a, MultiSelect, Item, Category, Message>;
pub type MultiSelectModel<Item, Category> = Model<MultiSelect, Item, Category>;
