//! Exercise the RON configuration through real text layout without a display server.

#[path = "support/process.rs"]
mod test_process;

use cosmic::config::{COSMIC_TK, CosmicTk};
use cosmic::iced::{Font, Pixels, Size, advanced::layout};
use cosmic::widget::text;
use cosmic_config::{Config, ConfigSet, CosmicConfigEntry};

fn renderer(size: f32) -> cosmic::Renderer {
    let renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels(size));
    #[cfg(feature = "wgpu")]
    let renderer = cosmic::Renderer::Secondary(renderer);
    renderer
}

fn dimensions<'a>(widget: impl Into<cosmic::Element<'a, ()>>) -> Size {
    layout_node(widget).size()
}

fn layout_node<'a>(widget: impl Into<cosmic::Element<'a, ()>>) -> layout::Node {
    // Match a fresh application's renderer; startup settings are tested in app::settings.
    let mut renderer = renderer(f32::from(cosmic::config::font_size()));
    let mut element = widget.into();
    let mut tree = cosmic::iced::advanced::widget::Tree::new(element.as_widget());
    element.as_widget_mut().diff(&mut tree);
    element.as_widget_mut().layout(
        &mut tree,
        &mut renderer,
        &layout::Limits::new(Size::ZERO, Size::new(2000.0, 2000.0)),
    )
}

fn draw_metrics(
    element: &mut cosmic::Element<'_, ()>,
    tree: &mut cosmic::iced::advanced::widget::Tree,
) -> Vec<(f32, f32)> {
    use cosmic::iced::advanced::{graphics::text::Text, mouse, renderer};
    use cosmic::iced::{Rectangle, advanced::Layout};

    // Keep the old renderer default to exercise a config reload in an existing app.
    let mut renderer = renderer(14.0);
    element.as_widget_mut().diff(tree);
    let node = element.as_widget_mut().layout(
        tree,
        &mut renderer,
        &layout::Limits::new(Size::ZERO, Size::new(2000.0, 2000.0)),
    );
    element.as_widget().draw(
        tree,
        &mut renderer,
        &cosmic::Theme::default(),
        &renderer::Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(node.size()),
    );
    #[cfg(feature = "wgpu")]
    let cosmic::Renderer::Secondary(renderer) = &mut renderer else {
        unreachable!("tests use the software renderer");
    };
    renderer
        .layers()
        .iter()
        .flat_map(|layer| layer.text.iter())
        .flat_map(|item| item.as_slice())
        .filter_map(|item| match item {
            Text::Cached {
                size, line_height, ..
            } => Some((size.0, line_height.0)),
            Text::Paragraph { paragraph, .. } => paragraph.upgrade().map(|paragraph| {
                let metrics = paragraph.buffer().metrics();
                (metrics.font_size, metrics.line_height)
            }),
            _ => None,
        })
        .collect()
}

fn drawn_sizes<'a>(widget: impl Into<cosmic::Element<'a, ()>>) -> Vec<f32> {
    let mut element = widget.into();
    let mut tree = cosmic::iced::advanced::widget::Tree::new(element.as_widget());
    draw_metrics(&mut element, &mut tree)
        .into_iter()
        .map(|(size, _)| size)
        .collect()
}

fn menu_size(
    element: &mut cosmic::Element<'_, ()>,
    tree: &mut cosmic::iced::advanced::widget::Tree,
    open: bool,
) -> Size {
    use cosmic::iced::{
        Event, Point, Rectangle, Vector,
        advanced::{Layout, clipboard, mouse},
    };
    let mut renderer = renderer(14.0);
    element.as_widget_mut().diff(tree);
    let node = element.as_widget_mut().layout(
        tree,
        &mut renderer,
        &layout::Limits::new(Size::ZERO, Size::new(2000.0, 2000.0)),
    );
    let viewport = Rectangle::with_size(Size::new(2000.0, 2000.0));
    if open {
        element.as_widget_mut().update(
            tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(5.0, 5.0)),
            &renderer,
            &mut clipboard::Null,
            &mut cosmic::iced::advanced::Shell::new(&mut Vec::new()),
            &viewport,
        );
    }
    element
        .as_widget_mut()
        .overlay(tree, Layout::new(&node), &renderer, &viewport, Vector::ZERO)
        .expect("dropdown menu is open")
        .as_overlay_mut()
        .layout(&renderer, viewport.size())
        .size()
}

fn menu_width(
    element: &mut cosmic::Element<'_, ()>,
    tree: &mut cosmic::iced::advanced::widget::Tree,
    open: bool,
) -> f32 {
    menu_size(element, tree, open).width
}

struct TestConfig {
    config: Config,
    directory: tempfile::TempDir,
}

impl TestConfig {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let config = Config::with_custom_path(
            cosmic::config::ID,
            CosmicTk::VERSION,
            directory.path().to_owned(),
        )
        .unwrap();
        CosmicTk::default().write_entry(&config).unwrap();
        let fixture = Self { config, directory };
        fixture.load();
        fixture
    }

    fn load(&self) {
        *COSMIC_TK.write().unwrap() =
            CosmicTk::get_entry(&self.config).unwrap_or_else(|(_, entry)| entry);
    }

    fn set(&self, size: u16) {
        self.config.set("font_size", size).unwrap();
        self.load();
    }
}

#[test]
fn ron_font_size_scales_typography() {
    test_process::run("ron_font_size_scales_typography", || {
        let fixture = TestConfig::new();
        fixture.set(21);
        let original = dimensions(text::body("Font size").size(14).line_height(
            cosmic::iced::advanced::text::LineHeight::Absolute(21.0.into()),
        ));
        assert_eq!(original.height, 21.0);

        fixture.set(21);
        let enlarged = dimensions(text::body("Font size"));
        assert_eq!(
            enlarged.height, 31.5,
            "RON font size must reach text layout"
        );
        assert!(enlarged.width > original.width);

        for (widget, height) in [
            (text::title1("Title"), 78.0),
            (text::title2("Title"), 64.5),
            (text::title3("Title"), 54.0),
            (text::title4("Title"), 45.0),
            (text::heading("Heading"), 31.5),
            (text::caption("Caption"), 25.5),
            (text::caption_heading("Caption"), 25.5),
            (text::monotext("Monospace"), 30.0),
        ] {
            assert_eq!(dimensions(widget).height, height);
        }
    });
}

#[test]
fn invalid_and_missing_font_sizes_are_safe() {
    for (value, expected_height) in [
        ("0", 12.0),
        ("7", 12.0),
        ("33", 48.0),
        ("65535", 48.0),
        ("-1", 21.0),
        ("NaN", 21.0),
        ("\"invalid\"", 21.0),
        ("missing", 21.0),
        ("8", 12.0),
        ("32", 48.0),
        ("14", 21.0),
    ] {
        test_process::run_case("invalid_and_missing_font_sizes_are_safe", value, || {
            let fixture = TestConfig::new();
            let path = fixture
                .directory
                .path()
                .join("cosmic/com.system76.CosmicTk/v1/font_size");
            if value == "missing" {
                std::fs::remove_file(path).unwrap();
            } else {
                std::fs::write(path, value).unwrap();
            }
            fixture.load();
            assert_eq!(
                dimensions(text::body("Font size")).height,
                expected_height,
                "{value}"
            );
        });
    }
}

#[test]
fn controls_fit_large_text_and_preserve_explicit_sizes() {
    for size in [14_u16, 32] {
        test_process::run_case(
            "controls_fit_large_text_and_preserve_explicit_sizes",
            &size.to_string(),
            || {
                let fixture = TestConfig::new();
                fixture.set(size);
                // At the largest supported size, controls must grow enough to fit their text.
                use cosmic::widget::{
                    button, dropdown, icon, nav_bar, segmented_button, text_input, toggler,
                };
                let model: segmented_button::SingleSelectModel = segmented_button::Model::builder()
                    .insert(|entry| entry.text("Font size"))
                    .build();
                let controls = || -> Vec<(&str, cosmic::Element<'_, ()>)> {
                    vec![
                        ("button", button::text("Font size").into()),
                        ("link", button::link("Font size").into()),
                        (
                            "icon button",
                            button::icon(icon::from_name("edit-copy-symbolic"))
                                .label("Font size")
                                .into(),
                        ),
                        (
                            "dropdown",
                            dropdown(vec!["Font size"], Some(0), |_| ()).into(),
                        ),
                        ("text input", text_input("", "Font size").into()),
                        (
                            "toggler",
                            toggler(false).label(String::from("Font size")).into(),
                        ),
                        (
                            "segmented button",
                            segmented_button::horizontal(&model).into(),
                        ),
                    ]
                };
                let baseline = [32.0_f32, 28.0, 36.0, 32.8, 35.6, 24.0, 32.0];
                for ((name, widget), normal) in controls().into_iter().zip(baseline) {
                    let measured = dimensions(widget);
                    if size == 14 {
                        assert!(
                            (measured.height - normal).abs() < 0.001,
                            "{name}: default height {} != {normal}",
                            measured.height
                        );
                    } else {
                        assert!(
                            measured.height > normal && measured.height >= 32.0,
                            "{name}: {normal} -> {measured:?}"
                        );
                    }
                }
                // The navigation viewport fills the window; its scroll content must fit the label.
                let navigation = layout_node(nav_bar(&model, |_| ()));
                let mut content = &navigation;
                while let [child] = content.children() {
                    content = child;
                }
                assert!(
                    content.size().height >= f32::from(size) * 1.4,
                    "navigation content: {content:?}"
                );
                // Explicit application sizes still take precedence over the preference.
                assert_eq!(
                    dimensions(text::text("Override").size(14).line_height(1.5)).height,
                    21.0
                );
            },
        );
    }
}

#[test]
fn dropdown_explicit_size_updates_drawing_and_menu_measurements() {
    test_process::run(
        "dropdown_explicit_size_updates_drawing_and_menu_measurements",
        || {
            let fixture = TestConfig::new();
            use cosmic::widget::dropdown;
            let mut renderer = renderer(14.0);
            let mut dropdown_element: cosmic::Element<'_, ()> = dropdown(
                vec!["Font size", "Longer option text for measuring menu"],
                Some(0),
                |_| (),
            )
            .into();
            let mut dropdown_tree =
                cosmic::iced::advanced::widget::Tree::new(dropdown_element.as_widget());
            dropdown_element.as_widget_mut().diff(&mut dropdown_tree);
            let original_menu_width = menu_width(&mut dropdown_element, &mut dropdown_tree, true);
            fixture.set(32);
            dropdown_element = dropdown(
                vec!["Font size", "Longer option text for measuring menu"],
                Some(0),
                |_| (),
            )
            .text_size(32.0)
            .into();
            dropdown_element.as_widget_mut().diff(&mut dropdown_tree);
            let updated = dropdown_element.as_widget_mut().layout(
                &mut dropdown_tree,
                &mut renderer,
                &layout::Limits::new(Size::ZERO, Size::new(2000.0, 2000.0)),
            );
            assert_eq!(
                updated.size(),
                dimensions(dropdown(vec!["Font size"], Some(0), |_| ()).text_size(32.0)),
                "reload must invalidate cached dropdown measurements"
            );
            assert!(
                menu_width(&mut dropdown_element, &mut dropdown_tree, false)
                    > original_menu_width * 1.5,
                "reload must resize the menu for unselected labels too"
            );
            assert_eq!(
                drawn_sizes(dropdown(vec!["Font size"], Some(0), |_| ()).text_size(32.0)),
                vec![32.0]
            );
            let mut multi = dropdown::multi::model();
            multi.insert(dropdown::multi::list(None, vec![("Font size", 0)]));
            multi.selected = Some(0);
            assert_eq!(
                drawn_sizes(dropdown::multi::dropdown(&multi, |_| ()).text_size(32.0)),
                vec![32.0]
            );
            assert_eq!(
                drawn_sizes(dropdown(vec!["Font size"], Some(0), |_| ()).text_size(16.0)),
                vec![16.0]
            );
        },
    );
}

#[test]
fn segmented_buttons_refresh_explicit_metrics() {
    test_process::run("segmented_buttons_refresh_explicit_metrics", || {
        use cosmic::iced::advanced::widget::Tree;
        use cosmic::widget::segmented_button;
        let fixture = TestConfig::new();
        let model: segmented_button::SingleSelectModel = segmented_button::Model::builder()
            .insert(|entry| entry.text("First"))
            .insert(|entry| entry.text("Second"))
            .build();
        let mut element: cosmic::Element<'_, ()> = segmented_button::horizontal(&model).into();
        let mut tree = Tree::new(element.as_widget());
        assert_eq!(
            draw_metrics(&mut element, &mut tree)
                .iter()
                .map(|m| m.0)
                .collect::<Vec<_>>(),
            vec![14.0, 14.0]
        );
        fixture.set(32);
        let mut element: cosmic::Element<'_, ()> =
            segmented_button::horizontal(&model).font_size(32.0).into();
        let updated = draw_metrics(&mut element, &mut tree);
        assert_eq!(
            updated.iter().map(|m| m.0).collect::<Vec<_>>(),
            vec![32.0, 32.0],
            "existing paragraph cache must reload font size"
        );
        let mut element: cosmic::Element<'_, ()> = segmented_button::horizontal(&model)
            .font_size(32.0)
            .line_height(cosmic::iced::advanced::text::LineHeight::Relative(2.0))
            .into();
        assert_eq!(
            draw_metrics(&mut element, &mut tree),
            vec![(32.0, 64.0), (32.0, 64.0)],
            "line height must also invalidate the paragraph cache"
        );
    });
}

#[cfg(all(feature = "winit", feature = "tokio"))]
struct FontApp {
    core: cosmic::Core,
    initial_text_height: f32,
}

#[cfg(all(feature = "winit", feature = "tokio"))]
impl cosmic::Application for FontApp {
    type Executor = cosmic::iced::executor::Default;
    type Flags = ();
    type Message = ();
    const APP_ID: &'static str = "com.system76.FontSizeTest";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }
    fn init(core: cosmic::Core, _: ()) -> (Self, cosmic::app::Task<()>) {
        (
            Self {
                core,
                initial_text_height: dimensions(text::body("Font size")).height,
            },
            cosmic::app::Task::none(),
        )
    }
    fn view(&self) -> cosmic::Element<'_, ()> {
        text::body("Font size").into()
    }
}

#[cfg(all(feature = "winit", feature = "tokio"))]
#[test]
fn applets_keep_panel_sizing_on_startup_and_reload() {
    test_process::run("applets_keep_panel_sizing_on_startup_and_reload", || {
        use cosmic::{Application, app::cosmic::Cosmic, core::AppType};
        use cosmic_config::ConfigGet;
        let fixture = TestConfig::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _entered = runtime.enter();
        for kind in [AppType::Window, AppType::System, AppType::Applet] {
            fixture.set(21);
            let mut core = cosmic::Core::default();
            core.set_app_type(kind);
            let (mut application, _) = Cosmic::<FontApp>::init((core, ()));
            assert_eq!(
                COSMIC_TK.read().unwrap().font_size,
                21,
                "runtime policy must preserve the raw config on initialization"
            );
            let expected = if kind == AppType::Applet { 21.0 } else { 31.5 };
            assert_eq!(
                application.app.initial_text_height, expected,
                "{kind:?} initialization"
            );
            fixture.config.set("font_size", 28_u16).unwrap();
            let incoming = CosmicTk::get_entry(&fixture.config).unwrap();
            let _ = application.update(cosmic::Action::Cosmic(cosmic::app::Action::ToolkitConfig(
                incoming,
            )));
            assert_eq!(
                COSMIC_TK.read().unwrap().font_size,
                28,
                "runtime policy must preserve the raw config on reload"
            );
            let expected = if kind == AppType::Applet { 21.0 } else { 31.5 };
            assert_eq!(
                dimensions(application.app.view()).height,
                expected,
                "{kind:?} reload"
            );
            assert_eq!(
                fixture.config.get::<u16>("font_size").unwrap(),
                28,
                "applet opt-out must not rewrite the stored preference"
            );
            if kind == AppType::Applet {
                COSMIC_TK.write().unwrap().font_size = 32;
                assert_eq!(
                    dimensions(application.app.view()).height,
                    21.0,
                    "direct configuration writes must not bypass applet sizing"
                );
            }
        }
    });
}

#[test]
fn multi_dropdown_row_height_follows_explicit_text_size() {
    use cosmic::iced::advanced::widget::Tree;
    use cosmic::widget::dropdown::multi;
    for (global, explicit, expected) in [
        (14, 14.0, 16.0),
        (32, 14.0, 16.0),
        (32, 12.0, 96.0 / 7.0),
        (14, 28.0, 32.0),
    ] {
        test_process::run_case(
            "multi_dropdown_row_height_follows_explicit_text_size",
            &format!("{global}-{explicit}"),
            || {
                let fixture = TestConfig::new();
                let mut model = multi::model();
                model.insert(multi::list(None, vec![("Font size", 0)]));
                model.selected = Some(0);
                fixture.set(global);
                let mut element: cosmic::Element<'_, ()> = multi::dropdown(&model, |_| ())
                    .text_size(explicit)
                    .padding(0)
                    .into();
                let mut tree = Tree::new(element.as_widget());
                let size = menu_size(&mut element, &mut tree, true);
                assert!(
                    (size.height - expected).abs() < 0.001,
                    "global {global}, explicit {explicit}: menu height {} != {expected}",
                    size.height
                );
            },
        );
    }
}

#[cfg(all(feature = "winit", feature = "tokio"))]
#[test]
fn font_preference_is_stable_until_restart() {
    use cosmic::{Application, app::cosmic::Cosmic};
    for size in [21_u16, 28] {
        test_process::run_case(
            "font_preference_is_stable_until_restart",
            &size.to_string(),
            || {
                let fixture = TestConfig::new();
                fixture.set(size);
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _entered = runtime.enter();
                let (mut application, _) = Cosmic::<FontApp>::init((cosmic::Core::default(), ()));
                let expected = f32::from(size) * 1.5;
                assert_eq!(application.app.initial_text_height, expected);
                let mut tree = {
                    let mut view = application.app.view();
                    let mut tree = cosmic::iced::advanced::widget::Tree::new(view.as_widget());
                    assert_eq!(
                        draw_metrics(&mut view, &mut tree),
                        vec![(f32::from(size), expected)]
                    );
                    tree
                };
                fixture.config.set("font_size", size + 3).unwrap();
                let incoming = CosmicTk::get_entry(&fixture.config).unwrap();
                let _ = application.update(cosmic::Action::Cosmic(
                    cosmic::app::Action::ToolkitConfig(incoming),
                ));
                assert_eq!(COSMIC_TK.read().unwrap().font_size, size + 3);
                assert_eq!(
                    dimensions(application.app.view()).height,
                    expected,
                    "font preference changes must not partially resize an existing session"
                );
                assert_eq!(
                    draw_metrics(&mut application.app.view(), &mut tree),
                    vec![(f32::from(size), expected)],
                    "existing widget trees must retain the startup size too"
                );
            },
        );
    }
}

#[test]
fn icon_button_presets_scale_in_order() {
    use cosmic::widget::{button, icon};
    for (size, expected) in [
        (
            14_u16,
            vec![(14.0, 20.0), (24.0, 32.0), (28.0, 36.0), (32.0, 44.0)],
        ),
        (
            32,
            vec![(32.0, 46.0), (55.0, 74.0), (64.0, 83.0), (74.0, 101.0)],
        ),
    ] {
        test_process::run_case(
            "icon_button_presets_scale_in_order",
            &size.to_string(),
            || {
                let fixture = TestConfig::new();
                fixture.set(size);
                let make = || button::icon(icon::from_name("edit-copy-symbolic")).label("Label");
                let buttons: Vec<cosmic::Element<'_, ()>> = vec![
                    make().extra_small().into(),
                    make().medium().into(),
                    make().large().into(),
                    make().extra_large().into(),
                ];
                let actual: Vec<_> = buttons
                    .into_iter()
                    .flat_map(|mut element| {
                        let mut tree =
                            cosmic::iced::advanced::widget::Tree::new(element.as_widget());
                        draw_metrics(&mut element, &mut tree)
                    })
                    .collect();
                assert_eq!(
                    actual, expected,
                    "all icon-button typography presets must scale together"
                );
            },
        );
    }
}
