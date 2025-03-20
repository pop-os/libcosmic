use std::collections::HashMap;

use apply::Apply;

use crate::{
    widget::{button, icon, responsive_container},
    Core, Element,
};

use super::menu;

/// # Panics
///
/// Will panic if the menu bar collapses without tracking the size
pub fn responsive_menu_bar<'a, Message: Clone + 'static, A: menu::Action<Message = Message>>(
    core: &Core,
    key_binds: &HashMap<menu::KeyBind, A>,
    id: crate::widget::Id,
    action_message: impl Fn(crate::surface::Action) -> Message + 'static,
    trees: Vec<(
        std::borrow::Cow<'static, str>,
        Vec<menu::Item<A, std::borrow::Cow<'static, str>>>,
    )>,
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
                        .map(|mt| {
                            menu::Tree::<_>::with_children(
                                menu::root(mt.0),
                                menu::items(key_binds, mt.1),
                            )
                        })
                        .collect(),
                ),
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
                            .map(|mt| menu::Item::Folder(mt.0, mt.1))
                            .collect(),
                    ),
                )]),
                crate::widget::Id::new(format!("menu_bar_collapsed_{id}")),
            ),
            id,
            action_message,
        )
        .size(menu_bar_size.unwrap().1)
        .apply(Element::from)
    }
}
