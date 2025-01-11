use derive_setters::Setters;

use crate::widget::table::model::{
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
    Entity, Model,
};
use crate::{
    theme,
    widget::{self, container, divider, menu},
    Apply, Element,
};
use iced::{Alignment, Border, Padding};

// THIS IS A PLACEHOLDER UNTIL A MORE SOPHISTICATED WIDGET CAN BE DEVELOPED

#[derive(Setters)]
#[must_use]
pub struct TableView<'a, SelectionMode, Item, Category, Message>
where
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    pub(super) model: &'a Model<SelectionMode, Item, Category>,

    #[setters(into)]
    pub(super) element_padding: Padding,

    #[setters(into)]
    pub(super) item_padding: Padding,
    pub(super) item_spacing: u16,
    pub(super) icon_size: u16,

    #[setters(into)]
    pub(super) divider_padding: Padding,

    #[setters(skip)]
    pub(super) item_context_builder: Box<dyn Fn(&Item) -> Option<Vec<menu::Tree<'a, Message>>>>,
    #[setters(skip)]
    pub(super) category_contexts: Box<dyn Fn(Category) -> Option<Vec<menu::Tree<'a, Message>>>>,

    #[setters(skip)]
    pub(super) on_item_select: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    #[setters(skip)]
    pub(super) on_item_context: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    #[setters(skip)]
    pub(super) on_category_select: Option<Box<dyn Fn(Category, bool) -> Message + 'a>>,
    #[setters(skip)]
    pub(super) on_category_context: Option<Box<dyn Fn(Category) -> Message + 'a>>,
}

impl<'a, SelectionMode, Item, Category, Message>
    From<TableView<'a, SelectionMode, Item, Category, Message>> for Element<'a, Message>
where
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    fn from(val: TableView<'a, SelectionMode, Item, Category, Message>) -> Self {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let header_row = val
            .model
            .categories
            .iter()
            .cloned()
            .map(|category| {
                let cat_context_tree = (val.category_contexts)(category);

                let mut sort_state = 0;

                if let Some(sort) = val.model.sort {
                    if sort.0 == category {
                        if sort.1 {
                            sort_state = 1;
                        } else {
                            sort_state = 2;
                        }
                    }
                };

                // Build the category header
                widget::row()
                    .spacing(space_xxxs)
                    .push(widget::text::heading(category.to_string()))
                    .push_maybe(match sort_state {
                        1 => Some(widget::icon::from_name("pan-up-symbolic").icon()),
                        2 => Some(widget::icon::from_name("pan-down-symbolic").icon()),
                        _ => None,
                    })
                    .apply(container)
                    .padding(
                        Padding::default()
                            .left(val.item_padding.left)
                            .right(val.item_padding.right),
                    )
                    .width(category.width())
                    .apply(widget::mouse_area)
                    .apply(|mouse_area| {
                        if let Some(ref on_category_select) = val.on_category_select {
                            mouse_area.on_press((on_category_select)(
                                category,
                                if let Some(sort) = val.model.sort {
                                    if sort.0 == category {
                                        !sort.1
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                },
                            ))
                        } else {
                            mouse_area
                        }
                    })
                    .apply(|mouse_area| widget::context_menu(mouse_area, cat_context_tree))
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::row::with_children)
            .apply(Element::from);
        // Build the items
        let items_full = if val.model.items.is_empty() {
            vec![divider::horizontal::default()
                .apply(container)
                .padding(val.divider_padding)
                .apply(Element::from)]
        } else {
            val.model
                .iter()
                .map(move |entity| {
                    let item = val.model.item(entity).unwrap();
                    let categories = &val.model.categories;
                    let selected = val.model.is_active(entity);
                    let item_context = (val.item_context_builder)(&item);

                    vec![
                        divider::horizontal::default()
                            .apply(container)
                            .padding(val.divider_padding)
                            .apply(Element::from),
                        categories
                            .iter()
                            .map(|category| {
                                widget::row()
                                    .spacing(space_xxxs)
                                    .push_maybe(
                                        item.get_icon(*category)
                                            .map(|icon| icon.size(val.icon_size)),
                                    )
                                    .push(widget::text::body(item.get_text(*category)))
                                    .align_y(Alignment::Center)
                                    .apply(container)
                                    .width(category.width())
                                    .align_y(Alignment::Center)
                                    .apply(Element::from)
                            })
                            .collect::<Vec<Element<'a, Message>>>()
                            .apply(widget::row::with_children)
                            .apply(container)
                            .padding(val.item_padding)
                            .class(theme::Container::custom(move |theme| {
                                widget::container::Style {
                                    icon_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    text_color: if selected {
                                        Some(theme.cosmic().on_accent_color().into())
                                    } else {
                                        None
                                    },
                                    background: if selected {
                                        Some(iced::Background::Color(
                                            theme.cosmic().accent_color().into(),
                                        ))
                                    } else {
                                        None
                                    },
                                    border: Border {
                                        radius: theme.cosmic().radius_xs().into(),
                                        ..Default::default()
                                    },
                                    shadow: Default::default(),
                                }
                            }))
                            .apply(widget::mouse_area)
                            .apply(|mouse_area| {
                                if let Some(ref on_item_select) = val.on_item_select {
                                    mouse_area.on_press((on_item_select)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            .apply(|mouse_area| widget::context_menu(mouse_area, item_context))
                            .apply(Element::from),
                    ]
                })
                .flatten()
                .collect::<Vec<Element<'a, Message>>>()
        };
        vec![vec![header_row], items_full]
            .into_iter()
            .flatten()
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::column::with_children)
            .spacing(val.item_spacing)
            .padding(val.element_padding)
            .apply(Element::from)
    }
}

impl<'a, SelectionMode, Item, Category, Message>
    TableView<'a, SelectionMode, Item, Category, Message>
where
    SelectionMode: Default,
    Model<SelectionMode, Item, Category>: Selectable,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Message: Clone + 'static,
{
    pub fn new(model: &'a Model<SelectionMode, Item, Category>) -> Self {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::active().cosmic().spacing;

        Self {
            model,
            element_padding: Padding::from(0),

            divider_padding: Padding::from(0).left(space_xxxs).right(space_xxxs),

            item_padding: Padding::from(space_xxs).into(),
            item_spacing: 0,
            icon_size: 24,

            on_item_select: None,
            on_item_context: None,
            item_context_builder: Box::new(|_| None),

            on_category_select: None,
            on_category_context: None,
            category_contexts: Box::new(|_| None),
        }
    }

    pub fn on_item_select<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_select = Some(Box::new(on_select));
        self
    }

    pub fn on_item_context<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Entity) -> Message + 'a,
    {
        self.on_item_context = Some(Box::new(on_select));
        self
    }

    pub fn item_context<F>(mut self, context_menu_builder: F) -> Self
    where
        F: Fn(&Item) -> Option<Vec<menu::Tree<'a, Message>>> + 'static,
        Message: 'static,
    {
        self.item_context_builder = Box::new(context_menu_builder);
        self
    }

    pub fn category_context<F>(mut self, context_menu_builder: F) -> Self
    where
        F: Fn(Category) -> Option<Vec<menu::Tree<'a, Message>>> + 'static,
        Message: 'static,
    {
        self.category_contexts = Box::new(context_menu_builder);
        self
    }

    pub fn on_category_select<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category, bool) -> Message + 'a,
    {
        self.on_category_select = Some(Box::new(on_select));
        self
    }

    pub fn on_category_context<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'a,
    {
        self.on_category_context = Some(Box::new(on_select));
        self
    }
}
