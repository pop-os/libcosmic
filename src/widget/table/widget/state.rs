use std::collections::HashMap;

use iced::Size;
use slotmap::SecondaryMap;

use crate::widget::table::{Entity, ItemCategory};

pub struct State<Category>
where
    Category: ItemCategory + 'static,
{
    pub(super) num_items: usize,
    pub(super) selected: SecondaryMap<Entity, bool>,
    pub(super) paragraphs: SecondaryMap<Entity, HashMap<Category, crate::Plain>>,
    pub(super) text_hashes: SecondaryMap<Entity, u64>,
    pub(super) item_layout: Vec<Size>,

    pub(super) sort_hash: HashMap<Category, u64>,
    pub(super) header_paragraphs: HashMap<Category, crate::Plain>,
    pub(super) category_layout: Vec<Size>,
}
