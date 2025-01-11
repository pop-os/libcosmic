use derive_setters::Setters;

use crate::widget::table::model::{
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
    Entity, Model,
};
use crate::{
    theme,
    widget::{self, container, menu},
    Apply, Element,
};
use iced::{Alignment, Border, Padding};

#[derive(Setters)]
#[must_use]
pub struct CompactTableView<'a, SelectionMode, Item, Category, Message>
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
    pub(super) on_item_select: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
    #[setters(skip)]
    pub(super) on_item_context: Option<Box<dyn Fn(Entity) -> Message + 'a>>,
}

impl<'a, SelectionMode, Item, Category, Message>
    From<CompactTableView<'a, SelectionMode, Item, Category, Message>> for Element<'a, Message>
where
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
    Message: Clone + 'static,
{
    fn from(val: CompactTableView<'a, SelectionMode, Item, Category, Message>) -> Self {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;
        val.model
            .iter()
            .map(|entity| {
                let item = val.model.item(entity).unwrap();
                let selected = val.model.is_active(entity);
                let context_menu = (val.item_context_builder)(&item);

                widget::column()
                    .spacing(val.item_spacing)
                    .push(
                        widget::divider::horizontal::default()
                            .apply(container)
                            .padding(val.divider_padding),
                    )
                    .push(
                        widget::row()
                            .spacing(space_xxxs)
                            .align_y(Alignment::Center)
                            .push_maybe(
                                item.get_icon(Category::default())
                                    .map(|icon| icon.size(val.icon_size)),
                            )
                            .push(
                                widget::column()
                                    .push(widget::text::body(item.get_text(Category::default())))
                                    .push({
                                        let mut elements = val
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
                            .padding(val.item_padding)
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
                                if let Some(on_item_select) = &val.on_item_select {
                                    ma.on_press((on_item_select)(entity))
                                } else {
                                    ma
                                }
                            })
                            .apply(|ma| widget::context_menu(ma, context_menu)),
                    )
                    .apply(Element::from)
            })
            .collect::<Vec<Element<'a, Message>>>()
            .apply(widget::column::with_children)
            .spacing(val.item_spacing)
            .padding(val.element_padding)
            .apply(Element::from)
    }
}

impl<'a, SelectionMode, Item, Category, Message>
    CompactTableView<'a, SelectionMode, Item, Category, Message>
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
            icon_size: 48,

            item_context_builder: Box::new(|_| None),
            on_item_select: None,
            on_item_context: None,
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
}
