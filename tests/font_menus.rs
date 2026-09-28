//! Opened menu geometry must follow the renderer and preserve explicit overrides.
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
fn direct() -> widget::menu::MenuBar<()> {
    widget::menu::bar(vec![widget::menu::Tree::with_children(
        cosmic::Element::from(widget::button::text("Menu")),
        vec![
            widget::menu::Tree::from(cosmic::Element::from(widget::menu::menu_button(vec![
                widget::text("Item").into(),
            ]))),
            widget::menu::Tree::from(cosmic::Element::from(widget::divider::horizontal::light())),
            widget::menu::Tree::from(cosmic::Element::from(widget::menu::menu_button(vec![
                widget::text("Other").into(),
            ]))),
        ],
    )])
}
#[test]
fn opened_menu_defaults_and_overrides_follow_renderer() {
    let mut failures = Vec::new();
    for size in [14.0_f32, 22.0, 32.0] {
        let mut direct: cosmic::Element<'_, ()> = direct().into();
        #[cfg(feature = "surface-message")]
        let mut responsive = widget::responsive_menu_bar().into_element(
            &cosmic::Core::default(),
            &std::collections::HashMap::new(),
            widget::Id::unique(),
            |_| (),
            vec![(
                "Menu",
                vec![
                    widget::menu::Item::Button("Item", None, MenuAction),
                    widget::menu::Item::Divider,
                    widget::menu::Item::Button("Other", None, MenuAction),
                ],
            )],
        );
        for (name, bar) in [
            ("direct", &mut direct),
            #[cfg(feature = "surface-message")]
            ("responsive", &mut responsive),
        ] {
            let node = open_menu(bar, size);
            let mut rows = &node;
            while let [child] = rows.children() {
                rows = child;
            }
            let height = (size * 1.4 + 8.0).ceil().max(30.0);
            let width = (150.0 * size / 14.0).ceil();
            assert_eq!(rows.children().len(), 3, "opened rows");
            for row in rows.children() {
                if row.size() != Size::new(width, height) {
                    failures.push(format!(
                        "{name} renderer {size}: {:?} != {width}x{height}",
                        row.size()
                    ));
                }
            }
        }
        let mut explicit: cosmic::Element<'_, ()> = self::direct()
            .item_height(widget::menu::ItemHeight::Uniform(60))
            .item_width(widget::menu::ItemWidth::Uniform(240))
            .into();
        #[cfg(feature = "surface-message")]
        let mut responsive_explicit = widget::responsive_menu_bar()
            .item_height(widget::menu::ItemHeight::Uniform(60))
            .item_width(widget::menu::ItemWidth::Uniform(240))
            .into_element(
                &cosmic::Core::default(),
                &std::collections::HashMap::new(),
                widget::Id::unique(),
                |_| (),
                vec![(
                    "Menu",
                    vec![
                        widget::menu::Item::Button("Item", None, MenuAction),
                        widget::menu::Item::Divider,
                        widget::menu::Item::Button("Other", None, MenuAction),
                    ],
                )],
            );
        for bar in [
            &mut explicit,
            #[cfg(feature = "surface-message")]
            &mut responsive_explicit,
        ] {
            let node = open_menu(bar, size);
            let mut rows = &node;
            while let [child] = rows.children() {
                rows = child;
            }
            for row in rows.children() {
                assert_eq!(row.size(), Size::new(240.0, 60.0));
            }
        }
    }
    assert!(failures.is_empty(), "menu geometry: {failures:#?}");
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
