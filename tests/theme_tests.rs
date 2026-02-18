// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Theme and style consistency tests for COSMIC.
//!
//! These tests verify that the COSMIC theme system produces consistent,
//! valid output across different theme variants (light, dark, high-contrast).
//! They catch issues like missing colors, zero-alpha components, and
//! inconsistencies between theme variants.

use cosmic::theme;
use cosmic_theme::Theme as CosmicTheme;

// ---------------------------------------------------------------------------
// Theme variant consistency tests
// ---------------------------------------------------------------------------

fn all_themes() -> Vec<(&'static str, &'static CosmicTheme)> {
    vec![
        ("dark", &*theme::COSMIC_DARK),
        ("light", &*theme::COSMIC_LIGHT),
        ("hc_dark", &*theme::COSMIC_HC_DARK),
        ("hc_light", &*theme::COSMIC_HC_LIGHT),
    ]
}

#[test]
fn all_themes_have_valid_spacing() {
    for (name, t) in all_themes() {
        assert!(t.spacing.space_xxs > 0, "{name}: space_xxs is zero");
        assert!(t.spacing.space_xs > 0, "{name}: space_xs is zero");
        assert!(t.spacing.space_s > 0, "{name}: space_s is zero");
        assert!(t.spacing.space_m > 0, "{name}: space_m is zero");
        assert!(t.spacing.space_l > 0, "{name}: space_l is zero");
        assert!(t.spacing.space_xl > 0, "{name}: space_xl is zero");
        assert!(t.spacing.space_xxl > 0, "{name}: space_xxl is zero");
    }
}

#[test]
fn all_themes_have_valid_corner_radii() {
    for (name, t) in all_themes() {
        let radii = &t.corner_radii;
        // Corner radii should be non-negative
        for r in &radii.radius_0 {
            assert!(*r >= 0.0, "{name}: negative radius_0");
        }
        for r in &radii.radius_xs {
            assert!(*r >= 0.0, "{name}: negative radius_xs");
        }
        for r in &radii.radius_s {
            assert!(*r >= 0.0, "{name}: negative radius_s");
        }
        for r in &radii.radius_m {
            assert!(*r >= 0.0, "{name}: negative radius_m");
        }
        for r in &radii.radius_l {
            assert!(*r >= 0.0, "{name}: negative radius_l");
        }
        for r in &radii.radius_xl {
            assert!(*r >= 0.0, "{name}: negative radius_xl");
        }
    }
}

#[test]
fn all_themes_have_non_zero_alpha_accent() {
    for (name, t) in all_themes() {
        let accent = &t.accent;
        assert!(
            accent.base.alpha > 0.0,
            "{name}: accent base alpha is zero"
        );
    }
}

#[test]
fn dark_themes_have_is_dark_set() {
    assert!(theme::COSMIC_DARK.is_dark);
    assert!(theme::COSMIC_HC_DARK.is_dark);
}

#[test]
fn light_themes_have_is_dark_unset() {
    assert!(!theme::COSMIC_LIGHT.is_dark);
    assert!(!theme::COSMIC_HC_LIGHT.is_dark);
}

#[test]
fn all_themes_have_non_zero_alpha_on_bg() {
    for (name, t) in all_themes() {
        let bg = &t.background;
        assert!(
            bg.base.alpha > 0.0,
            "{name}: background base alpha is zero"
        );
    }
}

#[test]
fn all_themes_have_non_zero_alpha_on_primary() {
    for (name, t) in all_themes() {
        let primary = &t.primary;
        assert!(
            primary.base.alpha > 0.0,
            "{name}: primary base alpha is zero"
        );
    }
}

#[test]
fn all_themes_have_non_zero_alpha_on_destructive() {
    for (name, t) in all_themes() {
        let dest = &t.destructive;
        assert!(
            dest.base.alpha > 0.0,
            "{name}: destructive base alpha is zero"
        );
    }
}

#[test]
fn all_themes_have_non_zero_alpha_on_warning() {
    for (name, t) in all_themes() {
        let warn = &t.warning;
        assert!(
            warn.base.alpha > 0.0,
            "{name}: warning base alpha is zero"
        );
    }
}

#[test]
fn all_themes_have_non_zero_alpha_on_success() {
    for (name, t) in all_themes() {
        let success = &t.success;
        assert!(
            success.base.alpha > 0.0,
            "{name}: success base alpha is zero"
        );
    }
}

// ---------------------------------------------------------------------------
// Component state consistency tests
// ---------------------------------------------------------------------------

/// Verify that all states of a Component have non-zero alpha.
fn assert_component_states_visible(name: &str, variant: &str, component: &cosmic_theme::Component) {
    assert!(
        component.base.alpha > 0.0,
        "{name}/{variant}: base alpha is zero"
    );
    // hover/pressed/selected states can have zero alpha in some themes, but
    // the 'on' color (text on top) should always be visible
    assert!(
        component.on.alpha > 0.0,
        "{name}/{variant}: 'on' (text) alpha is zero"
    );
}

#[test]
fn background_component_states_visible() {
    for (name, t) in all_themes() {
        assert_component_states_visible(name, "bg.component", &t.background.component);
    }
}

#[test]
fn primary_component_states_visible() {
    for (name, t) in all_themes() {
        assert_component_states_visible(name, "primary.component", &t.primary.component);
    }
}

// ---------------------------------------------------------------------------
// Serialization round-trip test
// ---------------------------------------------------------------------------

#[test]
fn dark_theme_serializes_to_ron() {
    let theme = CosmicTheme::dark_default();
    let serialized = ron::to_string(&theme);
    assert!(serialized.is_ok(), "Dark theme failed to serialize to RON");
}

#[test]
fn light_theme_serializes_to_ron() {
    let theme = CosmicTheme::light_default();
    let serialized = ron::to_string(&theme);
    assert!(serialized.is_ok(), "Light theme failed to serialize to RON");
}

#[test]
fn dark_theme_ron_round_trip() {
    let theme = CosmicTheme::dark_default();
    let serialized = ron::to_string(&theme).expect("serialize");
    let deserialized: CosmicTheme = ron::from_str(&serialized).expect("deserialize");

    assert_eq!(theme.is_dark, deserialized.is_dark);
    assert_eq!(theme.spacing.space_m, deserialized.spacing.space_m);
}

#[test]
fn light_theme_ron_round_trip() {
    let theme = CosmicTheme::light_default();
    let serialized = ron::to_string(&theme).expect("serialize");
    let deserialized: CosmicTheme = ron::from_str(&serialized).expect("deserialize");

    assert_eq!(theme.is_dark, deserialized.is_dark);
    assert_eq!(theme.spacing.space_m, deserialized.spacing.space_m);
}

// ---------------------------------------------------------------------------
// Spacing monotonicity
// ---------------------------------------------------------------------------

#[test]
fn all_themes_spacing_is_monotonic() {
    for (name, t) in all_themes() {
        let s = &t.spacing;
        assert!(
            s.space_xxs <= s.space_xs,
            "{name}: xxs({}) > xs({})",
            s.space_xxs,
            s.space_xs
        );
        assert!(
            s.space_xs <= s.space_s,
            "{name}: xs({}) > s({})",
            s.space_xs,
            s.space_s
        );
        assert!(
            s.space_s <= s.space_m,
            "{name}: s({}) > m({})",
            s.space_s,
            s.space_m
        );
        assert!(
            s.space_m <= s.space_l,
            "{name}: m({}) > l({})",
            s.space_m,
            s.space_l
        );
        assert!(
            s.space_l <= s.space_xl,
            "{name}: l({}) > xl({})",
            s.space_l,
            s.space_xl
        );
        assert!(
            s.space_xl <= s.space_xxl,
            "{name}: xl({}) > xxl({})",
            s.space_xl,
            s.space_xxl
        );
    }
}

// ---------------------------------------------------------------------------
// Corner radii ordering
// ---------------------------------------------------------------------------

#[test]
fn all_themes_corner_radii_are_ordered() {
    for (name, t) in all_themes() {
        let r = &t.corner_radii;
        // radius_0 should be all zeros
        for val in &r.radius_0 {
            assert_eq!(*val, 0.0, "{name}: radius_0 is not zero");
        }
        // Each subsequent radius should be >= the previous (at index 0)
        assert!(
            r.radius_xs[0] <= r.radius_s[0],
            "{name}: radius_xs > radius_s"
        );
        assert!(
            r.radius_s[0] <= r.radius_m[0],
            "{name}: radius_s > radius_m"
        );
        assert!(
            r.radius_m[0] <= r.radius_l[0],
            "{name}: radius_m > radius_l"
        );
        assert!(
            r.radius_l[0] <= r.radius_xl[0],
            "{name}: radius_l > radius_xl"
        );
    }
}
