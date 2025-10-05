//! A [`Model`] for a multi menu dropdown widget.

/// Create a [`Model`] for a multi-list dropdown.
pub fn model<S, Item>() -> Model<S, Item> {
    Model {
        lists: Vec::new(),
        selected: None,
    }
}

/// Create a [`List`] for a multi-list dropdown widget.
pub fn list<S, Item>(description: Option<S>, options: Vec<(S, Item)>) -> List<S, Item> {
    List {
        description,
        options,
    }
}

/// A model for managing the options in a multi-list dropdown.
///
/// ```no_run
/// #[derive(Copy, Clone, Eq, PartialEq)]
/// enum MenuOption {
///     Option1,
///     Option2,
///     Option3,
///     Option4,
///     Option5,
///     Option6
/// }
/// use cosmic::widget::dropdown;
///
/// let mut model = dropdown::multi::model();
///
/// model.insert(dropdown::multi::list(Some("List A"), vec![
///     ("Option 1", MenuOption::Option1),
///     ("Option 2", MenuOption::Option2),
///     ("Option 3", MenuOption::Option3)
/// ]));
///
/// model.insert(dropdown::multi::list(Some("List B"), vec![
///     ("Option 4", MenuOption::Option4),
///     ("Option 5", MenuOption::Option5),
///     ("Option 6", MenuOption::Option6)
/// ]));
///
/// model.clear();
/// ```
#[must_use]
pub struct Model<S, Item> {
    pub lists: Vec<List<S, Item>>,
    pub selected: Option<Item>,
}

impl<S, Item: PartialEq> Model<S, Item> {
    pub(super) fn get(&self, item: &Item) -> Option<&S> {
        for list in &self.lists {
            for option in &list.options {
                if &option.1 == item {
                    return Some(&option.0);
                }
            }
        }

        None
    }

    pub(super) fn next(&self) -> Option<&(S, Item)> {
        let item = self.selected.as_ref()?;

        let mut next = false;
        for list in &self.lists {
            for option in &list.options {
                if next {
                    return Some(option);
                }

                if &option.1 == item {
                    next = true;
                }
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.lists.clear();
    }

    pub fn insert(&mut self, list: List<S, Item>) {
        self.lists.push(list);
    }
}

/// A list for a multi-list dropdown widget.
pub struct List<S, Item> {
    pub description: Option<S>,
    pub options: Vec<(S, Item)>,
}
