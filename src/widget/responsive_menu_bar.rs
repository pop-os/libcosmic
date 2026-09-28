use std::collections::HashMap;

use apply::Apply;

use crate::widget::{button, icon, responsive_container};
use crate::{Core, Element};

use super::menu::{self, ItemHeight, ItemWidth};

#[must_use]
pub fn responsive_menu_bar() -> ResponsiveMenuBar {
    ResponsiveMenuBar::default()
}

pub struct ResponsiveMenuBar {
    item_width: Option<ItemWidth>,
    item_height: Option<ItemHeight>,
    spacing: f32,
}

impl Default for ResponsiveMenuBar {
    fn default() -> ResponsiveMenuBar {
        ResponsiveMenuBar {
            item_width: None,
            item_height: None,
            spacing: 0.,
        }
    }
}

impl ResponsiveMenuBar {
    /// Set the item width
    #[must_use]
    pub fn item_width(mut self, item_width: ItemWidth) -> Self {
        self.item_width = Some(item_width);
        self
    }

    /// Set the item height
    #[must_use]
    pub fn item_height(mut self, item_height: ItemHeight) -> Self {
        self.item_height = Some(item_height);
        self
    }

    /// Set the spacing
    #[must_use]
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// # Panics
    ///
    /// Will panic if the menu bar collapses without tracking the size
    pub fn into_element<
        'a,
        Message: Clone + 'static,
        A: menu::Action<Message = Message> + Clone,
        S: Into<std::borrow::Cow<'static, str>> + 'static,
    >(
        self,
        core: &Core,
        key_binds: &HashMap<menu::KeyBind, A>,
        id: crate::widget::Id,
        action_message: impl Fn(crate::surface::Action<Message>) -> Message
        + Send
        + Sync
        + Clone
        + 'static,
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
                                    menu::items(key_binds, mt.1),
                                )
                            })
                            .collect(),
                    )
                    .apply(|bar| match self.item_width {
                        Some(width) => bar.item_width(width),
                        None => bar,
                    })
                    .apply(|bar| match self.item_height {
                        Some(height) => bar.item_height(height),
                        None => bar,
                    })
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
                                .map(|mt| menu::Item::Folder(mt.0, mt.1))
                                .collect(),
                        )
                        .into_iter()
                        .map(|t| match self.item_width {
                            Some(ItemWidth::Uniform(w) | ItemWidth::Static(w)) => t.width(w),
                            None => t,
                        })
                        .collect(),
                    )])
                    .apply(|mut bar| {
                        #[cfg(wayland_platform)]
                        let width = if matches!(
                            crate::app::cosmic::WINDOWING_SYSTEM.get(),
                            Some(crate::app::cosmic::WindowingSystem::Wayland)
                        ) {
                            150
                        } else {
                            84
                        };
                        #[cfg(not(wayland_platform))]
                        let width = 84;
                        bar.default_root_width = Some(width);
                        bar
                    })
                    .apply(|bar| match self.item_height {
                        Some(height) => bar.item_height(height),
                        None => bar,
                    })
                    .apply(|bar| match self.item_width {
                        Some(width) => bar.item_width(width),
                        None => bar,
                    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iced::{
        Font, Pixels, Size,
        advanced::{layout, widget::Tree},
    };
    fn renderer(size: f32) -> crate::Renderer {
        let renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels(size));
        #[cfg(feature = "wgpu")]
        let renderer = crate::Renderer::Secondary(renderer);
        renderer
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Action;
    impl menu::Action for Action {
        type Message = ();
        fn message(&self) {}
    }

    #[test]
    fn collapsed_menu_preserves_root_and_submenu_widths() {
        for size in [14.0_f32, 32.0] {
            let mut core = Core::default();
            let id = crate::widget::Id::unique();
            core.menu_bars.insert(
                id.clone(),
                (
                    layout::Limits::new(Size::ZERO, Size::new(10.0, 1000.0)),
                    Size::new(100.0, 30.0),
                ),
            );
            for explicit in [false, true] {
                let builder = responsive_menu_bar();
                let builder = if explicit {
                    builder
                        .item_width(ItemWidth::Uniform(240))
                        .item_height(ItemHeight::Uniform(60))
                } else {
                    builder
                };
                let mut bar = builder.into_element(
                    &core,
                    &HashMap::new(),
                    id.clone(),
                    |_| (),
                    vec![(
                        "Menu",
                        vec![
                            menu::Item::Button("Item", None, Action),
                            menu::Item::Divider,
                            menu::Item::Button("Other", None, Action),
                        ],
                    )],
                );
                let node = open_menu(&mut bar, size);
                assert_eq!(
                    node.children().len(),
                    2,
                    "root and hovered submenu must open"
                );
                let height = if explicit {
                    60.0
                } else {
                    (size * 1.4 + 8.0).ceil().max(30.0)
                };
                for (index, menu) in node.children().iter().enumerate() {
                    let width = if explicit {
                        240.0
                    } else {
                        ((if index == 0 { 84.0 } else { 150.0 }) * size / 14.0).ceil()
                    };
                    assert_eq!(menu.size().width, width);
                    for row in menu.children() {
                        assert_eq!(row.size(), Size::new(width, height));
                    }
                }
            }
        }
    }
    fn open_menu(element: &mut crate::Element<'_, ()>, size: f32) -> layout::Node {
        use crate::iced::{
            Event, Point, Rectangle, Vector,
            advanced::{Layout, clipboard, mouse},
        };
        let renderer = renderer(size);
        let mut tree = Tree::new(element.as_widget());
        element.as_widget_mut().diff(&mut tree);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(1000.0, 1000.0)),
        );
        let viewport = Rectangle::with_size(Size::new(1000.0, 1000.0));
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(5.0, 5.0)),
            &renderer,
            &mut clipboard::Null,
            &mut crate::iced::advanced::Shell::new(&mut Vec::new()),
            &viewport,
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &viewport,
                Vector::ZERO,
            )
            .expect("menu opens");
        let initial = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        let cursor = Point::new(5.0, 5.0);
        overlay.as_overlay_mut().update(
            &Event::Mouse(mouse::Event::CursorMoved { position: cursor }),
            Layout::new(&initial),
            mouse::Cursor::Available(cursor),
            &renderer,
            &mut clipboard::Null,
            &mut crate::iced::advanced::Shell::new(&mut Vec::new()),
        );
        let opened = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        let cursor = Layout::new(&opened)
            .children()
            .next()
            .unwrap()
            .children()
            .next()
            .unwrap()
            .bounds()
            .center();
        overlay.as_overlay_mut().update(
            &Event::Mouse(mouse::Event::CursorMoved { position: cursor }),
            Layout::new(&opened),
            mouse::Cursor::Available(cursor),
            &renderer,
            &mut clipboard::Null,
            &mut crate::iced::advanced::Shell::new(&mut Vec::new()),
        );
        overlay.as_overlay_mut().layout(&renderer, viewport.size())
    }
}
