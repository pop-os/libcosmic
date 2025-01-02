use std::borrow::Cow;

use crate::widget::Icon;

/// Implementation of std::fmt::Display allows user to customize the header
/// Ideally, this is implemented on an enum.
pub trait ItemCategory: Default + std::fmt::Display + Clone + Copy + PartialEq + Eq {
    /// Function that gets the width of the data
    fn width(&self) -> u16;
}

pub trait ItemInterface<Category: ItemCategory>: Default {
    fn get_icon(&self, category: Category) -> Option<Icon>;
    fn get_text(&self, category: Category) -> Option<String>;

    fn compare(&self, other: &Self, category: Category) -> std::cmp::Ordering;
}
