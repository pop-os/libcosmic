// From iced_aw, license MIT

//! A tree structure for constructing a hierarchical menu

use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use iced::advanced::widget::text::Style as TextStyle;
use iced_widget::core::{Element, renderer};

use crate::iced_core::{Alignment, Length};
use crate::widget::menu::action::MenuAction;
use crate::widget::menu::key_bind::KeyBind;
use crate::widget::{Button, RcElementWrapper, icon};
use crate::{theme, widget};

/// Nested menu is essentially a tree of items, a menu is a collection of items
/// a menu itself can also be an item of another menu.
///
/// A `MenuTree` represents a node in the tree, it holds a widget as a menu item
/// for its parent, and a list of menu tree as child nodes.
/// Conceptually a node is either a menu(inner node) or an item(leaf node),
/// but there's no need to explicitly distinguish them here, if a menu tree
/// has children, it's a menu, otherwise it's an item
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct MenuTree<Message> {
    /// The menu tree will be flatten into a vector to build a linear widget tree,
    /// the `index` field is the index of the item in that vector
    pub(crate) index: usize,

    /// The item of the menu tree
    pub(crate) item: RcElementWrapper<Message>,
    /// The children of the menu tree
    pub(crate) children: Vec<MenuTree<Message>>,
    /// The width of the menu tree
    pub(crate) width: Option<u16>,
    /// The min width of the menu tree
    pub(crate) min_width: Option<u16>,
    /// The max width of the menu tree
    pub(crate) max_width: Option<u16>,
    /// The height of the menu tree
    pub(crate) height: Option<u16>,
}

impl<Message: Clone + 'static> MenuTree<Message> {
    /// Create a new menu tree from a widget
    pub fn new(item: impl Into<RcElementWrapper<Message>>) -> Self {
        Self {
            index: 0,
            item: item.into(),
            children: Vec::new(),
            width: None,
            min_width: None,
            max_width: None,
            height: None,
        }
    }

    /// Create a menu tree from a widget and a vector of sub trees
    pub fn with_children(
        item: impl Into<RcElementWrapper<Message>>,
        children: Vec<impl Into<MenuTree<Message>>>,
    ) -> Self {
        Self {
            index: 0,
            item: item.into(),
            children: children.into_iter().map(Into::into).collect(),
            width: None,
            min_width: None,
            max_width: None,
            height: None,
        }
    }

    /// Sets the width of the menu tree.
    /// See [`ItemWidth`]
    ///
    /// [`ItemWidth`]:`super::ItemWidth`
    #[must_use]
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the min width of the menu tree.
    pub fn min_width(mut self, min: u16) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Sets the max width of the menu tree.
    pub fn max_width(mut self, max: u16) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Sets the height of the menu tree.
    /// See [`ItemHeight`]
    ///
    /// [`ItemHeight`]: `super::ItemHeight`
    #[must_use]
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /* Keep `set_index()` and `flattern()` recurse in the same order */

    /// Set the index of each item
    pub(crate) fn set_index(&mut self) {
        /// inner counting function.
        fn rec<Message: Clone + 'static>(mt: &mut MenuTree<Message>, count: &mut usize) {
            // keep items under the same menu line up
            mt.children.iter_mut().for_each(|c| {
                c.index = *count;
                *count += 1;
            });

            mt.children.iter_mut().for_each(|c| rec(c, count));
        }

        let mut count = 0;
        self.index = count;
        count += 1;
        rec(self, &mut count);
    }

    /// Flatten the menu tree
    pub(crate) fn flattern(&self) -> Vec<&Self> {
        /// Inner flattening function
        fn rec<'a, Message: Clone + 'static>(
            mt: &'a MenuTree<Message>,
            flat: &mut Vec<&'a MenuTree<Message>>,
        ) {
            mt.children.iter().for_each(|c| {
                flat.push(c);
            });

            mt.children.iter().for_each(|c| {
                rec(c, flat);
            });
        }

        let mut flat = Vec::new();
        flat.push(self);
        rec(self, &mut flat);

        flat
    }
}

impl<Message: Clone + 'static> From<crate::Element<'static, Message>> for MenuTree<Message> {
    fn from(value: crate::Element<'static, Message>) -> Self {
        Self::new(RcElementWrapper::new(value))
    }
}

pub fn menu_button<'a, Message>(
    children: Vec<crate::Element<'a, Message>>,
) -> crate::widget::Button<'a, Message>
where
    Message: std::clone::Clone + 'a,
{
    widget::button::custom(
        widget::Row::from_vec(children)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill),
    )
    .height(Length::Fixed(36.0))
    .padding([4, 16])
    .width(Length::Fill)
    .class(theme::Button::MenuItem)
}

/// The type of menu item
#[derive(Clone)]
pub enum MenuItemKind<A: MenuAction, L: Into<Cow<'static, str>>> {
    /// Represents a button menu item.
    Button(L, Option<icon::Handle>, A),
    /// Represents a button menu item that's disabled.
    ButtonDisabled(L, Option<icon::Handle>, A),
    /// Represents a checkbox menu item.
    CheckBox(L, Option<icon::Handle>, bool, A),
    /// Represents a folder menu item.
    Folder(L, Vec<MenuItem<A, L>>),
    /// Represents a divider between menu items.
    Divider,
}

/// A menu item with optional width configuration.
///
/// # Examples
///
/// ```ignore
/// use cosmic::widget::menu;
///
/// // Simple button
/// menu::Item::button("Save", None, Action::Save);
///
/// // Button with icon
/// menu::Item::button(
///     "Open",
///     Some(cosmic::widget::icon::from_name("document-open-symbolic").into()),
///     Action::Open,
/// );
///
/// // Checkbox
/// menu::Item::checkbox("Show Hidden", None, true, Action::ToggleHidden);
///
/// // Folder with custom width
/// menu::Item::folder("Recent", vec![
///     menu::Item::button("file1.txt", None, Action::OpenRecent(0)),
/// ]).width(300);
///
/// // Divider
/// menu::Item::divider();
///
/// // Folder with custom width constraints
/// menu::Item::folder("Recent", vec![
///     menu::Item::button("file1.txt", None, Action::OpenRecent(0)),
/// ]).width(300).min_width(200).max_width(400);
///
/// // Using min_width to ensure a minimum size
/// menu::Item::button("Short", None, Action::Short).min_width(150);
///
/// // Using max_width to cap the size
/// menu::Item::button("Very Long Label Here", None, Action::Long).max_width(200);
/// ```
#[derive(Clone)]
pub struct MenuItem<A: MenuAction, L: Into<Cow<'static, str>>> {
    /// Kind of menu item.
    kind: MenuItemKind<A, L>,
    /// Optional width override for this item.
    width: Option<u16>,
    /// Optional min width for this item.
    min_width: Option<u16>,
    /// Optional max width for this item.
    max_width: Option<u16>,
}

impl<A: MenuAction, L: Into<Cow<'static, str>>> MenuItem<A, L> {
    /// Create from a kind with no width set
    pub fn new(kind: MenuItemKind<A, L>) -> Self {
        Self {
            kind,
            width: None,
            min_width: None,
            max_width: None,
        }
    }

    /// Builder method to set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder method to set minimum width
    pub fn min_width(mut self, min: u16) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Builder method to set max width
    pub fn max_width(mut self, max: u16) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Create a button menu item.
    pub fn button(label: L, icon: Option<icon::Handle>, action: A) -> Self {
        Self::new(MenuItemKind::Button(label, icon, action))
    }

    /// Create a disabled button menu item.
    pub fn button_disabled(label: L, icon: Option<icon::Handle>, action: A) -> Self {
        Self::new(MenuItemKind::ButtonDisabled(label, icon, action))
    }

    /// Create a checkbox menu item.
    pub fn checkbox(label: L, icon: Option<icon::Handle>, checked: bool, action: A) -> Self {
        Self::new(MenuItemKind::CheckBox(label, icon, checked, action))
    }

    /// Create a folder (submenu) menu item.
    pub fn folder(label: L, children: Vec<MenuItem<A, L>>) -> Self {
        Self::new(MenuItemKind::Folder(label, children))
    }

    /// Create a divider between menu items.
    pub fn divider() -> Self {
        Self::new(MenuItemKind::Divider)
    }
}

impl<A: MenuAction, L: Into<Cow<'static, str>>> From<MenuItemKind<A, L>> for MenuItem<A, L> {
    fn from(kind: MenuItemKind<A, L>) -> Self {
        Self::new(kind)
    }
}

/// Create a root menu item.
///
/// # Arguments
/// - `label` - The label of the menu item.
///
/// # Returns
/// - A button for the root menu item.
pub fn menu_root<'a, Message, Renderer: renderer::Renderer>(
    label: impl Into<Cow<'a, str>> + 'a,
) -> Button<'a, Message>
where
    Element<'a, Message, crate::Theme, Renderer>: From<widget::Button<'a, Message>>,
    Message: std::clone::Clone + 'a,
{
    widget::button::custom(widget::text(label))
        .padding([4, 12])
        .class(theme::Button::MenuRoot)
}

/// Create a list of menu items from a vector of `MenuItem`.
///
/// The `MenuItem` can be either an action or a separator.
///
/// # Arguments
/// - `key_binds` - A reference to a `HashMap` that maps `KeyBind` to `A`.
/// - `children` - A vector of `MenuItem`.
///
/// # Returns
/// - A vector of `MenuTree`.
#[must_use]
pub fn menu_items<
    A: MenuAction<Message = Message>,
    L: Into<Cow<'static, str>> + 'static,
    Message: 'static + std::clone::Clone,
>(
    key_binds: &HashMap<KeyBind, A>,
    children: Vec<MenuItem<A, L>>,
) -> Vec<MenuTree<Message>> {
    fn find_key<A: MenuAction>(action: &A, key_binds: &HashMap<KeyBind, A>) -> String {
        for (key_bind, key_action) in key_binds {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    }

    fn key_style(theme: &crate::Theme) -> TextStyle {
        let mut color = theme.cosmic().background.component.on;
        color.alpha *= 0.75;
        TextStyle {
            color: Some(color.into()),
        }
    }
    let key_class = theme::Text::Custom(key_style);

    let size = children.len();

    children
        .into_iter()
        .enumerate()
        .flat_map(|(i, item)| {
            let mut trees = vec![];
            let spacing = crate::theme::spacing();
            let item_width = item.width;
            let item_min_width = item.min_width;
            let item_max_width = item.max_width;

            match item.kind {
                MenuItemKind::Button(label, icon, action) => {
                    let l: Cow<'static, str> = label.into();
                    let key = find_key(&action, key_binds);
                    let mut items = vec![
                        widget::text(l).into(),
                        widget::horizontal_space().into(),
                        widget::text(key).class(key_class).into(),
                    ];

                    if let Some(icon) = icon {
                        items.insert(0, widget::icon::icon(icon).size(14).into());
                        items.insert(1, widget::Space::with_width(spacing.space_xxs).into());
                    }

                    let menu_button = menu_button(items).on_press(action.message());

                    // Add a user designated width
                    let mut tree = MenuTree::<Message>::from(Element::from(menu_button));

                    if let Some(width) = item_width {
                        tree = tree.width(width);
                    }

                    if let Some(min_width) = item_min_width {
                        tree = tree.min_width(min_width);
                    }

                    if let Some(max_width) = item_max_width {
                        tree = tree.max_width(max_width);
                    }

                    trees.push(tree);
                }
                MenuItemKind::ButtonDisabled(label, icon, action) => {
                    let l: Cow<'static, str> = label.into();

                    let key = find_key(&action, key_binds);

                    let mut items = vec![
                        widget::text(l).into(),
                        widget::horizontal_space().into(),
                        widget::text(key).class(key_class).into(),
                    ];

                    if let Some(icon) = icon {
                        items.insert(0, widget::icon::icon(icon).size(14).into());
                        items.insert(1, widget::Space::with_width(spacing.space_xxs).into());
                    }

                    let menu_button = menu_button(items);

                    let mut tree = MenuTree::<Message>::from(Element::from(menu_button));

                    if let Some(width) = item_width {
                        tree = tree.width(width);
                    }

                    if let Some(min_width) = item_min_width {
                        tree = tree.min_width(min_width);
                    }

                    if let Some(max_width) = item_max_width {
                        tree = tree.max_width(max_width);
                    }

                    trees.push(tree);
                }
                MenuItemKind::CheckBox(label, icon, value, action) => {
                    let key = find_key(&action, key_binds);
                    let mut items = vec![
                        if value {
                            widget::icon::from_name("object-select-symbolic")
                                .size(16)
                                .icon()
                                .class(theme::Svg::Custom(Rc::new(|theme| {
                                    iced_widget::svg::Style {
                                        color: Some(theme.cosmic().accent_text_color().into()),
                                    }
                                })))
                                .width(Length::Fixed(16.0))
                                .into()
                        } else {
                            widget::Space::with_width(Length::Fixed(16.0)).into()
                        },
                        widget::Space::with_width(spacing.space_xxs).into(),
                        widget::text(label).align_x(iced::Alignment::Start).into(),
                        widget::horizontal_space().into(),
                        widget::text(key).class(key_class).into(),
                    ];

                    if let Some(icon) = icon {
                        items.insert(1, widget::Space::with_width(spacing.space_xxs).into());
                        items.insert(2, widget::icon::icon(icon).size(14).into());
                    }

                    let mut tree = MenuTree::from(Element::from(
                        menu_button(items).on_press(action.message()),
                    ));

                    if let Some(width) = item_width {
                        tree = tree.width(width);
                    }

                    if let Some(min_width) = item_min_width {
                        tree = tree.min_width(min_width);
                    }

                    if let Some(max_width) = item_max_width {
                        tree = tree.max_width(max_width);
                    }

                    trees.push(tree);
                }
                MenuItemKind::Folder(label, children) => {
                    let l: Cow<'static, str> = label.into();

                    let mut tree = MenuTree::<Message>::with_children(
                        RcElementWrapper::new(crate::Element::from(
                            menu_button::<'static, _>(vec![
                                widget::text(l.clone()).into(),
                                widget::horizontal_space().into(),
                                widget::icon::from_name("pan-end-symbolic")
                                    .size(16)
                                    .icon()
                                    .into(),
                            ])
                            .class(
                                // Menu folders have no on_press so they take on the disabled style by default
                                if children.is_empty() {
                                    // This will make the folder use the disabled style if it has no children
                                    theme::Button::MenuItem
                                } else {
                                    // This will make the folder use the enabled style if it has children
                                    theme::Button::MenuFolder
                                },
                            ),
                        )),
                        menu_items(key_binds, children),
                    );

                    if let Some(width) = item_width {
                        tree = tree.width(width);
                    }

                    if let Some(min_width) = item_min_width {
                        tree = tree.min_width(min_width);
                    }

                    if let Some(max_width) = item_max_width {
                        tree = tree.max_width(max_width);
                    }

                    trees.push(tree);
                }
                MenuItemKind::Divider => {
                    if i != size - 1 {
                        let mut tree = MenuTree::<Message>::from(Element::from(
                            widget::divider::horizontal::light(),
                        ));

                        if let Some(width) = item_width {
                            tree = tree.width(width);
                        }

                        if let Some(min_width) = item_min_width {
                            tree = tree.min_width(min_width);
                        }

                        if let Some(max_width) = item_max_width {
                            tree = tree.max_width(max_width);
                        }

                        trees.push(tree);
                    }
                }
            }
            trees
        })
        .collect()
}
