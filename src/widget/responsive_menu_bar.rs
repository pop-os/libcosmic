use std::collections::HashMap;

use apply::Apply;
use iced::window;

use crate::{
    Core, Element,
    widget::{button, icon, responsive_container},
};

use super::menu::{self, ItemHeight, ItemWidth};

pub fn responsive_menu_bar() -> ResponsiveMenuBar {
    ResponsiveMenuBar::default()
}

pub struct ResponsiveMenuBar {
    collapsed_item_width: ItemWidth,
    item_width: ItemWidth,
    item_height: ItemHeight,
    spacing: f32,
}

impl Default for ResponsiveMenuBar {
    fn default() -> ResponsiveMenuBar {
        ResponsiveMenuBar {
            collapsed_item_width: ItemWidth::Static(84),
            item_width: ItemWidth::Uniform(150),
            item_height: ItemHeight::Uniform(30),
            spacing: 0.,
        }
    }
}

impl ResponsiveMenuBar {
    /// Set the item width
    pub fn item_width(mut self, item_width: ItemWidth) -> Self {
        self.item_width = item_width;
        self
    }

    /// Set the item height
    pub fn item_height(mut self, item_height: ItemHeight) -> Self {
        self.item_height = item_height;
        self
    }

    /// Set the spacing
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// # Panics
    ///
    /// Will panic if the menu bar collapses without tracking the size
    pub fn into_element<
        'a,
        Message: std::fmt::Debug + Clone + 'static,
        A: menu::Action<Message = Message> + Clone,
        S: Into<std::borrow::Cow<'static, str>> + 'static,
    >(
        self,
        core: &Core,
        key_binds: &HashMap<menu::KeyBind, A>,
        id: crate::widget::Id,
        action_message: impl Fn(crate::surface::Action) -> Message + Clone + 'static,
        trees: Vec<(S, Vec<menu::Item<A, S>>)>,
    ) -> Element<'a, Message> {
        use crate::widget::id_container;

        let menu_bar_size = core.menu_bars.get(&id);

        #[allow(clippy::if_not_else)]
        if !menu_bar_size.is_some_and(|(limits, size)| {
            let max_size = limits.max();
            max_size.width < size.width
        }) {
            responsive_container::responsive_container(
                id_container(
                    menu::bar(
                        trees
                            .into_iter()
                            .map(|mt: (S, Vec<menu::Item<A, S>>)| {
                                menu::Tree::<_>::with_children(
                                    crate::widget::RcElementWrapper::new(Element::from(
                                        menu::root(mt.0),
                                    )),
                                    menu::items(key_binds, mt.1.into()),
                                )
                            })
                            .collect(),
                    )
                    .item_width(self.item_width)
                    .item_height(self.item_height)
                    .spacing(self.spacing)
                    .on_surface_action(action_message.clone())
                    .window_id_maybe(core.main_window_id()),
                    crate::widget::Id::new(format!("menu_bar_expanded_{id}")),
                ),
                id,
                action_message,
            )
            .apply(Element::from)
        } else {
            responsive_container::responsive_container(
                id_container(
                    menu::bar(vec![menu::Tree::<_>::with_children(
                        Element::from(
                            button::icon(icon::from_name("open-menu-symbolic"))
                                .padding([4, 12])
                                .class(crate::theme::Button::MenuRoot),
                        ),
                        menu::items(
                            key_binds,
                            trees
                                .into_iter()
                                .map(|mt| menu::Item::Folder(mt.0, mt.1.into()))
                                .collect(),
                        )
                        .into_iter()
                        .map(|t| {
                            t.width(match self.item_width {
                                ItemWidth::Uniform(w) | ItemWidth::Static(w) => w,
                            })
                        })
                        .collect(),
                    )])
                    .item_height(self.item_height)
                    .item_width(self.collapsed_item_width)
                    .spacing(self.spacing)
                    .on_surface_action(action_message.clone())
                    .window_id_maybe(core.main_window_id()),
                    crate::widget::Id::new(format!("menu_bar_collapsed_{id}")),
                ),
                id,
                action_message,
            )
            .size(menu_bar_size.unwrap().1)
            .apply(Element::from)
        }
    }
}
