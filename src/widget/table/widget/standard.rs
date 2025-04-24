use derive_setters::Setters;

use crate::widget::table::model::{
    Entity, Model,
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
};
use crate::{
    Apply, Element, theme,
    widget::{self, container, divider, menu},
};
use iced::{Alignment, Border, Length, Padding};

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
    pub(super) width: Length,
    #[setters(into)]
    pub(super) height: Length,

    #[setters(into)]
    pub(super) item_padding: Padding,
    pub(super) item_spacing: u16,
    pub(super) icon_spacing: u16,
    pub(super) icon_size: u16,

    #[setters(into)]
    pub(super) divider_padding: Padding,

    // === Item Interaction ===
    #[setters(skip)]
    pub(super) on_item_mb_left: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_item_mb_double: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_item_mb_mid: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_item_mb_right: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) item_context_builder: Box<dyn Fn(&Item) -> Option<Vec<menu::Tree<Message>>>>,
    // Item DND

    // === Category Interaction ===
    #[setters(skip)]
    pub(super) on_category_mb_left: Option<Box<dyn Fn(Category) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_category_mb_double: Option<Box<dyn Fn(Category) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_category_mb_mid: Option<Box<dyn Fn(Category) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_category_mb_right: Option<Box<dyn Fn(Category) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) category_context_builder: Box<dyn Fn(Category) -> Option<Vec<menu::Tree<Message>>>>,
}

impl<SelectionMode, Item, Category, Message>
    From<TableView<'static, SelectionMode, Item, Category, Message>> for Element<'static, Message>
where
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    fn from(val: TableView<'static, SelectionMode, Item, Category, Message>) -> Self {
        // Header row
        let header_row = val
            .model
            .categories
            .iter()
            .copied()
            .map(|category| {
                let cat_context_tree = (val.category_context_builder)(category);

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
                    .spacing(val.icon_spacing)
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
                        if let Some(ref on_category_select) = val.on_category_mb_left {
                            mouse_area.on_press((on_category_select)(category))
                        } else {
                            mouse_area
                        }
                    })
                    .apply(|mouse_area| widget::context_menu(mouse_area, cat_context_tree))
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'static, Message>>>()
            .apply(widget::row::with_children)
            .apply(Element::from);
        // Build the items
        let items_full = if val.model.items.is_empty() {
            vec![
                divider::horizontal::default()
                    .apply(container)
                    .padding(val.divider_padding)
                    .apply(Element::from),
            ]
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
                                    .spacing(val.icon_spacing)
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
                            .collect::<Vec<Element<'static, Message>>>()
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
                            // Left click
                            .apply(|mouse_area| {
                                if let Some(ref on_item_mb) = val.on_item_mb_left {
                                    mouse_area.on_press((on_item_mb)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            // Double click
                            .apply(|mouse_area| {
                                if let Some(ref on_item_mb) = val.on_item_mb_left {
                                    mouse_area.on_double_click((on_item_mb)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            // Middle click
                            .apply(|mouse_area| {
                                if let Some(ref on_item_mb) = val.on_item_mb_mid {
                                    mouse_area.on_middle_press((on_item_mb)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            // Right click
                            .apply(|mouse_area| {
                                if let Some(ref on_item_mb) = val.on_item_mb_right {
                                    mouse_area.on_right_press((on_item_mb)(entity))
                                } else {
                                    mouse_area
                                }
                            })
                            .apply(|mouse_area| widget::context_menu(mouse_area, item_context))
                            .apply(Element::from),
                    ]
                })
                .flatten()
                .collect::<Vec<Element<'static, Message>>>()
        };
        vec![vec![header_row], items_full]
            .into_iter()
            .flatten()
            .collect::<Vec<Element<'static, Message>>>()
            .apply(widget::column::with_children)
            .width(val.width)
            .height(val.height)
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
        } = theme::spacing();

        Self {
            model,

            element_padding: Padding::from(0),
            width: Length::Fill,
            height: Length::Shrink,

            item_padding: Padding::from(space_xxs).into(),
            item_spacing: 0,
            icon_spacing: space_xxxs,
            icon_size: 24,

            divider_padding: Padding::from(0).left(space_xxxs).right(space_xxxs),

            on_item_mb_left: None,
            on_item_mb_double: None,
            on_item_mb_mid: None,
            on_item_mb_right: None,
            item_context_builder: Box::new(|_| None),

            on_category_mb_left: None,
            on_category_mb_double: None,
            on_category_mb_mid: None,
            on_category_mb_right: None,
            category_context_builder: Box::new(|_| None),
        }
    }

    pub fn on_item_left_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn(Entity) -> Message + 'static,
    {
        self.on_item_mb_left = Some(Box::new(on_click));
        self
    }

    pub fn on_item_double_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn(Entity) -> Message + 'static,
    {
        self.on_item_mb_double = Some(Box::new(on_click));
        self
    }

    pub fn on_item_middle_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn(Entity) -> Message + 'static,
    {
        self.on_item_mb_mid = Some(Box::new(on_click));
        self
    }

    pub fn on_item_right_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn(Entity) -> Message + 'static,
    {
        self.on_item_mb_right = Some(Box::new(on_click));
        self
    }

    pub fn item_context<F>(mut self, context_menu_builder: F) -> Self
    where
        F: Fn(&Item) -> Option<Vec<menu::Tree<Message>>> + 'static,
        Message: 'static,
    {
        self.item_context_builder = Box::new(context_menu_builder);
        self
    }

    pub fn on_category_left_click<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'static,
    {
        self.on_category_mb_left = Some(Box::new(on_select));
        self
    }
    pub fn on_category_double_click<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'static,
    {
        self.on_category_mb_double = Some(Box::new(on_select));
        self
    }
    pub fn on_category_middle_click<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'static,
    {
        self.on_category_mb_mid = Some(Box::new(on_select));
        self
    }

    pub fn on_category_right_click<F>(mut self, on_select: F) -> Self
    where
        F: Fn(Category) -> Message + 'static,
    {
        self.on_category_mb_right = Some(Box::new(on_select));
        self
    }

    pub fn category_context<F>(mut self, context_menu_builder: F) -> Self
    where
        F: Fn(Category) -> Option<Vec<menu::Tree<Message>>> + 'static,
        Message: 'static,
    {
        self.category_context_builder = Box::new(context_menu_builder);
        self
    }
}
