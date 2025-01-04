mod state;
use derive_setters::Setters;
use state::State;

use super::model::{
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
    Entity, Model,
};
use crate::{
    ext::CollectionWidget,
    theme,
    widget::{self, container, divider, menu},
    Apply, Element,
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
    pub(super) item_padding: Padding,
    pub(super) item_spacing: u16,

    #[setters(into)]
    pub(super) divider_padding: Padding,

    #[setters(skip)]
    pub(super) item_context_tree: Option<Vec<menu::Tree<'a, Message>>>,
    #[setters(skip)]
    pub(super) category_context_tree: Option<Vec<menu::Tree<'a, Message>>>,

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
            item_spacing: 0,
            element_padding: Padding::from(0),
            divider_padding: Padding::from(0).left(space_xxxs).right(space_xxxs),
            item_padding: Padding::from(space_xxs).into(),
            on_item_select: None,
            on_item_context: None,
            item_context_tree: None,
            on_category_select: None,
            on_category_context: None,
            category_context_tree: None,
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

    pub fn item_context(mut self, context_menu: Option<Vec<menu::Tree<'a, Message>>>) -> Self
    where
        Message: 'static,
    {
        self.item_context_tree =
            context_menu.map(|menus| vec![menu::Tree::with_children(widget::row(), menus)]);

        if let Some(ref mut context_menu) = self.item_context_tree {
            context_menu.iter_mut().for_each(menu::Tree::set_index);
        }

        self
    }

    pub fn category_context(mut self, context_menu: Option<Vec<menu::Tree<'a, Message>>>) -> Self
    where
        Message: 'static,
    {
        self.category_context_tree =
            context_menu.map(|menus| vec![menu::Tree::with_children(widget::row(), menus)]);

        if let Some(ref mut context_menu) = self.category_context_tree {
            context_menu.iter_mut().for_each(menu::Tree::set_index);
        }

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

    /*#[must_use]
    pub fn element_standard(&self) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let header_row = self
            .model
            .categories
            .iter()
            .map(|category| {
                widget::row()
                    .spacing(space_xxxs)
                    .push(widget::text::heading(category.to_string()))
                    .push_maybe(if let Some(sort) = self.model.sort {
                        if sort.0 == *category {
                            match sort.1 {
                                true => Some(widget::icon::from_name("pan-up-symbolic").icon()),
                                false => Some(widget::icon::from_name("pan-down-symbolic").icon()),
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    })
                    .apply(container)
                    .padding(
                        Padding::default()
                            .left(self.item_padding.left)
                            .right(self.item_padding.right),
                    )
                    .width(category.width())
                    .apply(widget::mouse_area)
                    .apply(|mouse_area| {
                        if let Some(ref on_category_select) = self.on_category_select {
                            mouse_area.on_press((on_category_select)(
                                *category,
                                if let Some(sort) = self.model.sort {
                                    if sort.0 == *category {
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
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::row::with_children)
            .apply(Element::from);
        let items_full = if self.model.items.is_empty() {
            vec![divider::horizontal::default()
                .apply(container)
                .padding(self.divider_padding)
                .apply(Element::from)]
        } else {
            self.model
                .iter()
                .map(|entity| {
                    let item = self.model.item(entity).unwrap();
                    let categories = &self.model.categories;
                    let selected = self.model.is_active(entity);

                    vec![
                        divider::horizontal::default()
                            .apply(container)
                            .padding(self.divider_padding)
                            .apply(Element::from),
                        categories
                            .iter()
                            .map(|category| {
                                widget::row()
                                    .spacing(space_xxxs)
                                    .push_maybe(item.get_icon(*category).map(|icon| icon.size(24)))
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
                            .padding(self.item_padding)
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
                                if let Some(ref on_item_select) = self.on_item_select {
                                    mouse_area.on_press((on_item_select)(entity))
                                } else {
                                    mouse_area
                                }
                            })
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
            .spacing(self.item_spacing)
            .padding(self.element_padding)
            .apply(Element::from)
    }

    #[must_use]
    pub fn element_compact(&self) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;
        self.model
            .iter()
            .map(|entity| {
                let item = self.model.item(entity).unwrap();
                let selected = self.model.is_active(entity);
                widget::column()
                    .spacing(self.item_spacing)
                    .push(
                        widget::divider::horizontal::default()
                            .apply(container)
                            .padding(self.divider_padding),
                    )
                    .push(
                        widget::row()
                            .spacing(space_xxxs)
                            .align_y(Alignment::Center)
                            .push_maybe(
                                item.get_icon(Category::default()).map(|icon| icon.size(48)),
                            )
                            .push(
                                widget::column()
                                    .push(widget::text::body(item.get_text(Category::default())))
                                    .push({
                                        let mut elements = self
                                            .model
                                            .categories
                                            .iter()
                                            .skip_while(|cat| **cat != Category::default())
                                            .map(|category| {
                                                vec![
                                                    widget::text::caption(item.get_text(*category))
                                                        .apply(Element::from),
                                                    widget::text::caption("-").apply(Element::from),
                                                ]
                                            })
                                            .flatten()
                                            .collect::<Vec<Element<'a, Message>>>();
                                        elements.pop();
                                        elements
                                            .apply(widget::row::with_children)
                                            .spacing(space_xxxs)
                                            .wrap()
                                    }),
                            )
                            .apply(container)
                            .padding(self.item_padding)
                            .width(iced::Length::Fill)
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
                            .apply(|ma| {
                                if let Some(on_item_select) = &self.on_item_select {
                                    ma.on_press((on_item_select)(entity))
                                } else {
                                    ma
                                }
                            }),
                    )
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::column::with_children)
            .spacing(self.item_spacing)
            .padding(self.element_padding)
            .apply(Element::from)
    }
    */
}
