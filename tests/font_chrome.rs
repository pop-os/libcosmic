//! Headless regressions for font-sensitive window and menu chrome.
#[path = "support/process.rs"]
mod test_process;

use cosmic::config::{COSMIC_TK, CosmicTk};
use cosmic::iced::{
    Font, Pixels, Size,
    advanced::{layout, widget::Tree},
};
use cosmic::widget;

fn renderer(size: f32) -> cosmic::Renderer {
    let renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels(size));
    #[cfg(feature = "wgpu")]
    let renderer = cosmic::Renderer::Secondary(renderer);
    renderer
}

fn layout(element: &mut cosmic::Element<'_, ()>, tree: &mut Tree, size: f32) -> layout::Node {
    element.as_widget_mut().diff(tree);
    element.as_widget_mut().layout(
        tree,
        &renderer(size),
        &layout::Limits::new(Size::ZERO, Size::new(1000.0, 1000.0)),
    )
}

#[test]
fn chrome_fits_large_fonts_and_keeps_default_geometry() {
    for startup in [14_u16, 22, 32] {
        test_process::run_case(
            "chrome_fits_large_fonts_and_keeps_default_geometry",
            &startup.to_string(),
            || {
                *COSMIC_TK.write().unwrap() = CosmicTk {
                    font_size: startup,
                    ..CosmicTk::default()
                };
                assert_eq!(cosmic::config::font_size(), startup);
                let mut header_tree = Tree::empty();
                let mut menu_tree = Tree::empty();
                let mut tabs_header_tree = Tree::empty();
                let tabs: widget::segmented_button::SingleSelectModel =
                    widget::segmented_button::Model::builder()
                        .insert(|entry| entry.text("First"))
                        .insert(|entry| entry.text("Second"))
                        .build();
                let mut failures = Vec::new();
                for edited_size in [14_u16, 22, 32, 14] {
                    COSMIC_TK.write().unwrap().font_size = edited_size;
                    let size = startup;
                    let mut header: cosmic::Element<'_, ()> = widget::header_bar()
                        .title("Window title")
                        .maximized(true)
                        .into();
                    let header = layout(&mut header, &mut header_tree, f32::from(size));
                    if size == 14 {
                        assert_eq!(header.size().height, 48.0, "default header geometry");
                    }
                    let title_height = f32::from(size) * 1.5;
                    assert!(
                        header.size().height - 16.0 >= title_height,
                        "header must fit the title line"
                    );
                    let mut standalone: cosmic::Element<'_, ()> =
                        widget::segmented_button::horizontal(&tabs)
                            .width(cosmic::iced::Length::Shrink)
                            .into();
                    let mut standalone_tree = Tree::new(standalone.as_widget());
                    let tabs_height =
                        layout(&mut standalone, &mut standalone_tree, f32::from(size))
                            .size()
                            .height;
                    let mut tabs_header: cosmic::Element<'_, ()> = widget::header_bar()
                        .title("Window title")
                        .maximized(true)
                        .start(
                            widget::segmented_button::horizontal(&tabs)
                                .width(cosmic::iced::Length::Shrink),
                        )
                        .into();
                    let tabs_header =
                        layout(&mut tabs_header, &mut tabs_header_tree, f32::from(size));
                    if tabs_header.size().height - 16.0 < tabs_height {
                        failures.push(format!(
                            "header at {size}: content {} clips standard tabs {tabs_height}",
                            tabs_header.size().height - 16.0
                        ));
                    }
                    // Menu labels use the renderer default, including an explicit application override.
                    for renderer_size in [f32::from(size), 14.0, 32.0] {
                        let mut menu: cosmic::Element<'_, ()> =
                            widget::menu::menu_button(vec![widget::text("Menu label").into()])
                                .into();
                        let menu = layout(&mut menu, &mut menu_tree, renderer_size);
                        let expected_menu = (renderer_size * 1.4).max(28.0) + 8.0;
                        if (menu.size().height - expected_menu).abs() > 0.01 {
                            failures.push(format!(
                                "menu at {size}, renderer {renderer_size}: {} != {expected_menu}",
                                menu.size().height
                            ));
                        }
                    }
                }
                for size in [14.0_f32, 22.0, 32.0] {
                    let menu: cosmic::Element<'_, ()> =
                        widget::menu::menu_button(vec![widget::text("Item").into()]).into();
                    let root: cosmic::Element<'_, ()> = widget::button::text("Menu").into();
                    let mut bar: cosmic::Element<'_, ()> =
                        widget::menu::bar(vec![widget::menu::Tree::with_children(
                            root,
                            vec![widget::menu::Tree::from(menu)],
                        )])
                        .into();
                    #[cfg(feature = "surface-message")]
                    let mut responsive = widget::responsive_menu_bar().into_element(
                        &cosmic::Core::default(),
                        &std::collections::HashMap::new(),
                        widget::Id::unique(),
                        |_| (),
                        vec![(
                            "Menu",
                            vec![widget::menu::Item::Button("Item", None, MenuAction)],
                        )],
                    );
                    for (name, bar) in [
                        ("direct", &mut bar),
                        #[cfg(feature = "surface-message")]
                        ("responsive", &mut responsive),
                    ] {
                        let node = open_menu(bar, size);
                        let expected = (size * 1.4).ceil().max(22.0) + 8.0;

                        assert!(
                            !node.children().is_empty(),
                            "menu contains its opened items"
                        );
                        let mut leaf = &node;
                        while let [child] = leaf.children() {
                            leaf = child;
                        }
                        // The menu row contains its text and a minimum-height spacer.
                        if (leaf.size().height + 8.0 - expected).abs() > 0.01 {
                            failures.push(format!(
                                "opened {name} menu at {size}: row {} != {}",
                                leaf.size().height,
                                expected - 8.0
                            ));
                        }
                    }
                }
                let item: cosmic::Element<'_, ()> =
                    widget::menu::menu_button(vec![widget::text("Item").into()]).into();
                let root: cosmic::Element<'_, ()> = widget::button::text("Menu").into();
                let mut fixed: cosmic::Element<'_, ()> =
                    widget::menu::bar(vec![widget::menu::Tree::with_children(
                        root,
                        vec![widget::menu::Tree::from(item)],
                    )])
                    .item_height(widget::menu::ItemHeight::Uniform(60))
                    .into();
                assert_eq!(
                    open_menu(&mut fixed, 32.0).children()[0].size().height,
                    60.0,
                    "explicit menu item heights remain authoritative"
                );
                assert!(failures.is_empty(), "chrome clips text: {failures:#?}");
            },
        );
    }
}

fn open_menu(element: &mut cosmic::Element<'_, ()>, size: f32) -> layout::Node {
    use cosmic::iced::{
        Event, Point, Rectangle, Vector,
        advanced::{Layout, clipboard, mouse},
    };
    let renderer = renderer(size);
    let mut tree = Tree::new(element.as_widget());
    let node = layout(element, &mut tree, size);
    let viewport = Rectangle::with_size(Size::new(1000.0, 1000.0));
    element.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(5.0, 5.0)),
        &renderer,
        &mut clipboard::Null,
        &mut cosmic::iced::advanced::Shell::new(&mut Vec::new()),
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
    overlay.as_overlay_mut().update(
        &Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(5.0, 5.0),
        }),
        Layout::new(&initial),
        mouse::Cursor::Available(Point::new(5.0, 5.0)),
        &renderer,
        &mut clipboard::Null,
        &mut cosmic::iced::advanced::Shell::new(&mut Vec::new()),
    );
    overlay.as_overlay_mut().layout(&renderer, viewport.size())
}

#[cfg(feature = "surface-message")]
#[derive(Clone, Copy, PartialEq, Eq)]
struct MenuAction;
#[cfg(feature = "surface-message")]
impl widget::menu::Action for MenuAction {
    type Message = ();
    fn message(&self) {}
}
