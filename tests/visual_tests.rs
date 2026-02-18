// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Visual snapshot tests for COSMIC widgets.
//!
//! These tests render widgets using the tiny-skia software renderer and compare
//! the output against stored reference snapshots. This catches unintended visual
//! regressions in widget rendering, theming, and layout.
//!
//! # Updating Snapshots
//!
//! When a visual change is intentional, delete the old snapshot PNG from
//! `tests/snapshots/` and re-run the tests. The new reference will be generated
//! automatically.

mod snapshot_harness;

use cosmic::iced_core::Length;
use cosmic::theme;
use snapshot_harness::SnapshotHarness;

/// Helper to render a cosmic Element to a snapshot.
fn render_element_snapshot(
    name: &str,
    width: u32,
    height: u32,
    element: cosmic::Element<'_, ()>,
) {
    let harness = SnapshotHarness::new(width, height);
    harness.snapshot(name, element);
}

// ---------------------------------------------------------------------------
// Theme rendering tests - verify that the cosmic theme system produces
// consistent visual output for core UI primitives.
// ---------------------------------------------------------------------------

#[test]
fn snapshot_container_with_background() {
    use cosmic::widget::{container, text};

    let content = text::body("Hello, COSMIC!")
        .apply(container)
        .padding(16)
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(60.0))
        .class(theme::Container::Primary);

    render_element_snapshot("container_with_background", 220, 80, content.into());
}

#[test]
fn snapshot_divider_horizontal() {
    use cosmic::widget::{column, container, divider};

    let content = column::with_capacity(3)
        .push(cosmic::widget::text::body("Above"))
        .push(divider::horizontal::default())
        .push(cosmic::widget::text::body("Below"))
        .width(Length::Fixed(200.0))
        .apply(container)
        .padding(8);

    render_element_snapshot("divider_horizontal", 220, 100, content.into());
}

#[test]
fn snapshot_divider_heavy() {
    use cosmic::widget::{column, container, divider};

    let content = column::with_capacity(3)
        .push(cosmic::widget::text::body("Above"))
        .push(divider::horizontal::heavy())
        .push(cosmic::widget::text::body("Below"))
        .width(Length::Fixed(200.0))
        .apply(container)
        .padding(8);

    render_element_snapshot("divider_heavy", 220, 100, content.into());
}

#[test]
fn snapshot_text_styles() {
    use cosmic::widget::{column, container, text};

    let content = column::with_capacity(4)
        .push(text::title3("Title Text"))
        .push(text::body("Body text content"))
        .push(text::caption("Caption text"))
        .push(text::monotext("Monospace text"))
        .spacing(8)
        .width(Length::Fixed(300.0))
        .apply(container)
        .padding(16);

    render_element_snapshot("text_styles", 340, 200, content.into());
}

#[test]
fn snapshot_progress_bar() {
    use cosmic::widget::{column, container, progress_bar};

    let content = column::with_capacity(3)
        .push(progress_bar(0.0..=100.0, 0.0).width(Length::Fixed(250.0)))
        .push(progress_bar(0.0..=100.0, 50.0).width(Length::Fixed(250.0)))
        .push(progress_bar(0.0..=100.0, 100.0).width(Length::Fixed(250.0)))
        .spacing(12)
        .apply(container)
        .padding(16);

    render_element_snapshot("progress_bar_states", 290, 120, content.into());
}

#[test]
fn snapshot_row_layout() {
    use cosmic::widget::{container, row, text};

    let content = row::with_capacity(3)
        .push(text::body("Left"))
        .push(text::body("Center"))
        .push(text::body("Right"))
        .spacing(16)
        .apply(container)
        .padding(12);

    render_element_snapshot("row_layout", 300, 60, content.into());
}

#[test]
fn snapshot_column_layout() {
    use cosmic::widget::{column, container, text};

    let content = column::with_capacity(3)
        .push(text::body("First"))
        .push(text::body("Second"))
        .push(text::body("Third"))
        .spacing(8)
        .apply(container)
        .padding(12);

    render_element_snapshot("column_layout", 200, 120, content.into());
}

#[test]
fn snapshot_nested_containers() {
    use cosmic::widget::{column, container, row, text};

    let inner = column::with_capacity(2)
        .push(text::body("Nested"))
        .push(text::caption("Content"))
        .spacing(4)
        .apply(container)
        .padding(8)
        .class(theme::Container::Primary);

    let outer = row::with_capacity(2)
        .push(inner)
        .push(text::body("Adjacent"))
        .spacing(16)
        .apply(container)
        .padding(12);

    render_element_snapshot("nested_containers", 300, 120, outer.into());
}

#[test]
fn snapshot_dark_theme_container() {
    use cosmic::widget::{container, text};

    // Set dark theme temporarily for this test
    let content = text::body("Dark Theme")
        .apply(container)
        .padding(16)
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(60.0))
        .class(theme::Container::Primary);

    // The default theme is dark, so this tests the default rendering
    render_element_snapshot("dark_theme_container", 220, 80, content.into());
}

// We use `apply` from the Apply trait for chaining
use apply::Apply;
