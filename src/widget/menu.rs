// From iced_aw, license MIT

//! A [`MenuBar`] widget for displaying [`MenuTree`]s
//!
//! *This API requires the following crate features to be activated: `menu`*
//!
//! # Example
//!
//! ```ignore
//! use iced::widget::button;
//! use iced_aw::menu::{MenuTree, MenuBar};
//!
//! let sub_2 = MenuTree::with_children(
//!     button("Sub Menu 2"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let sub_1 = MenuTree::with_children(
//!     button("Sub Menu 1"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         sub_2,
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//!
//! let root_1 = MenuTree::with_children(
//!     button("Menu 1"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         sub_1,
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let root_2 = MenuTree::with_children(
//!     button("Menu 2"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let menu_bar = MenuBar::new(vec![root_1, root_2]);
//!
//! ```
//!

pub mod action;

pub use action::MenuAction as Action;

mod flex;
pub mod key_bind;
pub use key_bind::KeyBind;

mod menu_bar;
pub(crate) use menu_bar::MenuBarState;
pub use menu_bar::{MenuBar, menu_bar as bar};

mod menu_inner;
mod menu_tree;
pub use menu_tree::{
    MenuItem as Item, MenuTree as Tree, menu_button, menu_items as items, menu_root as root,
};

pub use crate::style::menu_bar::{Appearance, StyleSheet};
pub(crate) use menu_bar::{menu_roots_children, menu_roots_diff};
pub use menu_inner::{CloseCondition, ItemHeight, ItemWidth, PathHighlight};
pub(crate) use menu_inner::{Direction, Menu, init_root_menu};
