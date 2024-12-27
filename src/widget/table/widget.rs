use super::model::{
    category::{ItemCategory, ItemInterface},
    selection::Selectable,
    Entity, Model,
};
use crate::{
    ext::CollectionWidget,
    theme,
    widget::{self, container, divider},
    Apply, Element,
};
use iced::{Alignment, Padding};
use iced_widget::container::Catalog;

// THIS IS A PLACEHOLDER UNTIL A MORE SOPHISTICATED WIDGET CAN BE DEVELOPED

#[must_use]
pub struct TableView<'a, SelectionMode, Item, Category, Message>
where
    Category: ItemCategory,
    Item: ItemInterface<Category>,
    Model<SelectionMode, Item, Category>: Selectable,
    SelectionMode: Default,
{
    pub(super) model: &'a Model<SelectionMode, Item, Category>,

    pub(super) spacing: u16,
    pub(super) padding: Padding,
    pub(super) list_item_padding: Padding,
    pub(super) divider_padding: Padding,
    pub(super) style: theme::Container<'a>,

    pub(super) on_selected: Box<dyn Fn(Entity) -> Message + 'a>,
    pub(super) on_category_select: Box<dyn Fn(Category, bool) -> Message + 'a>,
    pub(super) on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
}

impl<'a, SelectionMode, Item, Category, Message>
    TableView<'a, SelectionMode, Item, Category, Message>
where
    SelectionMode: Default,
    Model<SelectionMode, Item, Category>: Selectable,
    Category: ItemCategory,
    Item: ItemInterface<Category>,
{
    pub fn new(
        model: &'a Model<SelectionMode, Item, Category>,
        on_selected: impl Fn(Entity) -> Message + 'a,
        on_category_select: impl Fn(Category, bool) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(usize) -> Message>,
    ) -> Self {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            ..
        } = theme::active().cosmic().spacing;
        Self {
            model,
            spacing: 0,
            padding: Padding::from(0),
            divider_padding: Padding::from(0).left(space_xxxs).right(space_xxxs),
            list_item_padding: Padding::from(space_xxs).into(),
            style: theme::Container::Background,

            on_selected: Box::new(on_selected),
            on_category_select: Box::new(on_category_select),
            on_option_hovered,
        }
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the style variant of this [`Circular`].
    pub fn style(mut self, style: <crate::Theme as Catalog>::Class<'a>) -> Self {
        self.style = style;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn divider_padding(mut self, padding: u16) -> Self {
        self.divider_padding = Padding::from(0).left(padding).right(padding);
        self
    }

    pub fn list_item_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.list_item_padding = padding.into();
        self
    }
    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        crate::widget::column()
            .push(widget::row::with_children(
                self.model
                    .categories
                    .iter()
                    .map(|category| {
                        container(
                            widget::row()
                                .spacing(space_xxxs)
                                .push(widget::text::heading(category.to_string()))
                                .push_maybe(if self.model.sort.0 == *category {
                                    match self.model.sort.1 {
                                        true => {
                                            Some(widget::icon::from_name("pan-up-symbolic").icon())
                                        }
                                        false => Some(
                                            widget::icon::from_name("pan-down-symbolic").icon(),
                                        ),
                                    }
                                } else {
                                    None
                                }),
                        )
                        .padding(
                            Padding::from(0)
                                .left(self.list_item_padding.left)
                                .right(self.list_item_padding.right),
                        )
                        .width(category.width())
                        .into()
                    })
                    .collect(),
            ))
            .append(&mut if self.model.items.is_empty() {
                vec![container(divider::horizontal::default()).padding(self.divider_padding)]
            } else {
                self.model
                    .order
                    .iter()
                    .map(|entity| {
                        let item = self.model.item(*entity).unwrap();
                        let categories = &self.model.categories;

                        vec![
                            container(divider::horizontal::default()).padding(self.divider_padding),
                            container(widget::row::with_children(
                                categories
                                    .iter()
                                    .map(|category| {
                                        container(
                                            widget::row()
                                                .push_maybe(item.get_icon(*category))
                                                .push(widget::text::body(item.get_text(*category))),
                                        )
                                        .width(category.width())
                                        .align_y(Alignment::Center)
                                        .padding(self.list_item_padding)
                                        .apply(Element::from)
                                    })
                                    .collect(),
                            )),
                        ]
                    })
                    .flatten()
                    .collect()
            })
            .spacing(self.spacing)
            .padding(self.padding)
            .apply(container)
            .padding([self.spacing, 0])
            .class(self.style)
            .width(iced::Length::Fill)
            .into()
    }
}
