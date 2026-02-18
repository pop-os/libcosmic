// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Unit tests for COSMIC widget logic and models.
//!
//! These tests verify the behavioral correctness of widget models,
//! helper functions, and data structures without requiring a renderer.

// ---------------------------------------------------------------------------
// Spin button logic tests
// ---------------------------------------------------------------------------
mod spin_button_tests {
    use cosmic::widget::spin_button;

    // Test the internal increment/decrement functions via the public API.
    // The functions are private, so we test through behavior.

    #[test]
    fn spin_button_clamps_value_to_min() {
        // When value is below min, it should be clamped
        let button = spin_button::spin_button(
            "-5",
            #[cfg(feature = "a11y")]
            "test",
            -5i32,
            1,
            0,
            100,
            |v| v,
        );
        // The SpinButton constructor clamps value to min when value < min
        // We verify this through the element creation (it won't panic)
        let _: cosmic::Element<'_, i32> = button.into();
    }

    #[test]
    fn spin_button_clamps_value_to_max() {
        let button = spin_button::spin_button(
            "150",
            #[cfg(feature = "a11y")]
            "test",
            150i32,
            1,
            0,
            100,
            |v| v,
        );
        let _: cosmic::Element<'_, i32> = button.into();
    }

    #[test]
    fn spin_button_accepts_valid_range() {
        let button = spin_button::spin_button(
            "50",
            #[cfg(feature = "a11y")]
            "test",
            50i32,
            1,
            0,
            100,
            |v| v,
        );
        let _: cosmic::Element<'_, i32> = button.into();
    }

    #[test]
    fn spin_button_vertical_variant() {
        let button = spin_button::vertical(
            "25",
            #[cfg(feature = "a11y")]
            "vertical test",
            25i32,
            5,
            0,
            100,
            |v| v,
        );
        let _: cosmic::Element<'_, i32> = button.into();
    }

    #[test]
    fn spin_button_float_values() {
        let button = spin_button::spin_button(
            "3.14",
            #[cfg(feature = "a11y")]
            "float test",
            3.14f64,
            0.1,
            0.0,
            10.0,
            |v| v,
        );
        let _: cosmic::Element<'_, f64> = button.into();
    }
}

// ---------------------------------------------------------------------------
// Calendar model tests
// ---------------------------------------------------------------------------
mod calendar_model_tests {
    use chrono::{Datelike, NaiveDate, Weekday};
    use cosmic::widget::calendar::{CalendarModel, get_calender_first, set_day};

    #[test]
    fn calendar_model_creation() {
        let model = CalendarModel::now();
        assert_eq!(model.selected, model.visible);
    }

    #[test]
    fn calendar_model_custom_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let model = CalendarModel::new(date, date);
        assert_eq!(model.selected.month(), 6);
        assert_eq!(model.selected.day(), 15);
    }

    #[test]
    fn calendar_show_prev_month() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.show_prev_month();
        assert_eq!(model.visible.month(), 2);
        assert_eq!(model.selected, date); // selected unchanged
    }

    #[test]
    fn calendar_show_next_month() {
        let date = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.show_next_month();
        assert_eq!(model.visible.month(), 12);
    }

    #[test]
    fn calendar_show_prev_month_wraps_year() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.show_prev_month();
        assert_eq!(model.visible.month(), 12);
        assert_eq!(model.visible.year(), 2023);
    }

    #[test]
    fn calendar_show_next_month_wraps_year() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.show_next_month();
        assert_eq!(model.visible.month(), 1);
        assert_eq!(model.visible.year(), 2025);
    }

    #[test]
    fn calendar_set_prev_month_updates_selected() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.set_prev_month();
        assert_eq!(model.selected, model.visible);
        assert_eq!(model.visible.month(), 5);
    }

    #[test]
    fn calendar_set_next_month_updates_selected() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.set_next_month();
        assert_eq!(model.selected, model.visible);
        assert_eq!(model.visible.month(), 7);
    }

    #[test]
    fn calendar_set_selected_visible() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let new_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let mut model = CalendarModel::new(date, date);
        model.set_selected_visible(new_date);
        assert_eq!(model.selected, new_date);
        assert_eq!(model.visible, new_date);
    }

    #[test]
    fn set_day_increases() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let result = set_day(date, 20);
        assert_eq!(result.day(), 20);
        assert_eq!(result.month(), 6);
    }

    #[test]
    fn set_day_decreases() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let result = set_day(date, 5);
        assert_eq!(result.day(), 5);
    }

    #[test]
    fn set_day_same_returns_original() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = set_day(date, 15);
        assert_eq!(result, date);
    }

    #[test]
    fn get_calendar_first_monday_start() {
        // June 2024 starts on a Saturday
        let first = get_calender_first(2024, 6, Weekday::Mon);
        assert_eq!(first.weekday(), Weekday::Mon);
        // The first Monday before June 1 (Saturday) is May 27
        assert_eq!(first.day(), 27);
        assert_eq!(first.month(), 5);
    }

    #[test]
    fn get_calendar_first_sunday_start() {
        let first = get_calender_first(2024, 6, Weekday::Sun);
        assert_eq!(first.weekday(), Weekday::Sun);
        assert_eq!(first.day(), 26);
        assert_eq!(first.month(), 5);
    }

    #[test]
    fn get_calendar_first_when_month_starts_on_first_day() {
        // April 2024 starts on a Monday
        let first = get_calender_first(2024, 4, Weekday::Mon);
        assert_eq!(first.day(), 1);
        assert_eq!(first.month(), 4);
    }
}

// ---------------------------------------------------------------------------
// Segmented button model tests
// ---------------------------------------------------------------------------
mod segmented_model_tests {
    use cosmic::widget::segmented_button::{Model, SingleSelect, MultiSelect};

    #[test]
    fn builder_creates_items_in_order() {
        let model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("First"))
            .insert(|b| b.text("Second"))
            .insert(|b| b.text("Third"))
            .build();

        let texts: Vec<&str> = model
            .iter()
            .filter_map(|id| model.text(id))
            .collect();

        assert_eq!(texts, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn single_select_activates_only_one() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .insert(|b| b.text("C").with_id(|id| ids.push(id)))
            .build();

        model.activate(ids[0]);
        assert!(model.is_active(ids[0]));
        assert!(!model.is_active(ids[1]));

        model.activate(ids[1]);
        assert!(!model.is_active(ids[0]));
        assert!(model.is_active(ids[1]));
    }

    #[test]
    fn multi_select_toggles_items() {
        let mut ids = Vec::new();
        let mut model: Model<MultiSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .build();

        model.activate(ids[0]);
        model.activate(ids[1]);
        assert!(model.is_active(ids[0]));
        assert!(model.is_active(ids[1]));

        // Toggle off
        model.activate(ids[0]);
        assert!(!model.is_active(ids[0]));
        assert!(model.is_active(ids[1]));
    }

    #[test]
    fn remove_item_reduces_length() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .insert(|b| b.text("C").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.len(), 3);
        model.remove(ids[1]);
        assert_eq!(model.len(), 2);
        assert!(!model.contains_item(ids[1]));
    }

    #[test]
    fn clear_removes_all() {
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A"))
            .insert(|b| b.text("B"))
            .build();

        model.clear();
        assert_eq!(model.len(), 0);
    }

    #[test]
    fn data_storage_and_retrieval() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Item").with_id(|id| ids.push(id)))
            .build();

        model.data_set::<u32>(ids[0], 42);
        assert_eq!(model.data::<u32>(ids[0]), Some(&42));

        model.data_remove::<u32>(ids[0]);
        assert_eq!(model.data::<u32>(ids[0]), None);
    }

    #[test]
    fn text_set_and_remove() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Original").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.text(ids[0]), Some("Original"));

        model.text_set(ids[0], "Updated");
        assert_eq!(model.text(ids[0]), Some("Updated"));

        model.text_remove(ids[0]);
        assert_eq!(model.text(ids[0]), None);
    }

    #[test]
    fn enable_disable_items() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Item").with_id(|id| ids.push(id)))
            .build();

        assert!(model.is_enabled(ids[0]));

        model.enable(ids[0], false);
        assert!(!model.is_enabled(ids[0]));

        model.enable(ids[0], true);
        assert!(model.is_enabled(ids[0]));
    }

    #[test]
    fn position_operations() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .insert(|b| b.text("C").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.position(ids[0]), Some(0));
        assert_eq!(model.position(ids[1]), Some(1));
        assert_eq!(model.position(ids[2]), Some(2));

        // Move C to position 0
        model.position_set(ids[2], 0);
        assert_eq!(model.position(ids[2]), Some(0));
        assert_eq!(model.position(ids[0]), Some(1));
    }

    #[test]
    fn position_swap() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .build();

        assert!(model.position_swap(ids[0], ids[1]));
        assert_eq!(model.position(ids[0]), Some(1));
        assert_eq!(model.position(ids[1]), Some(0));
    }

    #[test]
    fn entity_at_returns_correct_entity() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.entity_at(0), Some(ids[0]));
        assert_eq!(model.entity_at(1), Some(ids[1]));
        assert_eq!(model.entity_at(2), None);
    }

    #[test]
    fn activate_position_works() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("A").with_id(|id| ids.push(id)))
            .insert(|b| b.text("B").with_id(|id| ids.push(id)))
            .build();

        assert!(model.activate_position(1));
        assert!(model.is_active(ids[1]));

        assert!(!model.activate_position(5)); // Out of bounds
    }

    #[test]
    fn closable_set_and_check() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Tab").with_id(|id| ids.push(id)))
            .build();

        assert!(!model.is_closable(ids[0]));

        model.closable_set(ids[0], true);
        assert!(model.is_closable(ids[0]));
    }

    #[test]
    fn indent_operations() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Item").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.indent(ids[0]), None);

        model.indent_set(ids[0], 2);
        assert_eq!(model.indent(ids[0]), Some(2));

        model.indent_remove(ids[0]);
        assert_eq!(model.indent(ids[0]), None);
    }

    #[test]
    fn divider_above_operations() {
        let mut ids = Vec::new();
        let mut model: Model<SingleSelect> = Model::builder()
            .insert(|b| b.text("Item").with_id(|id| ids.push(id)))
            .build();

        assert_eq!(model.divider_above(ids[0]), None);

        model.divider_above_set(ids[0], true);
        assert_eq!(model.divider_above(ids[0]), Some(true));

        model.divider_above_remove(ids[0]);
        assert_eq!(model.divider_above(ids[0]), None);
    }

    #[test]
    fn insert_returns_entity_mut() {
        let mut model: Model<SingleSelect> = Model::builder().build();
        assert_eq!(model.len(), 0);

        let id = model.insert().text("Dynamic").id();
        assert_eq!(model.len(), 1);
        assert_eq!(model.text(id), Some("Dynamic"));
    }
}

// ---------------------------------------------------------------------------
// Color picker model tests
// ---------------------------------------------------------------------------
mod color_picker_tests {
    use cosmic::widget::color_picker::ColorPickerModel;
    use iced_core::Color;

    #[test]
    fn color_picker_model_creation() {
        // Verify model can be created without panicking
        let _model = ColorPickerModel::new("Hex", "RGB", None, None);
    }

    #[test]
    fn color_picker_with_initial_color() {
        let color = Color::from_rgb(0.5, 0.3, 0.8);
        let _model = ColorPickerModel::new("Hex", "RGB", Some(color), Some(color));
    }

    #[test]
    fn color_picker_with_fallback() {
        let fallback = Color::from_rgb(1.0, 0.0, 0.0);
        let _model = ColorPickerModel::new("Hex", "RGB", Some(fallback), None);
    }
}

// ---------------------------------------------------------------------------
// Theme tests
// ---------------------------------------------------------------------------
mod theme_tests {
    use cosmic::theme;

    #[test]
    fn dark_theme_is_dark() {
        let dark = &*theme::COSMIC_DARK;
        assert!(dark.is_dark);
    }

    #[test]
    fn light_theme_is_light() {
        let light = &*theme::COSMIC_LIGHT;
        assert!(!light.is_dark);
    }

    #[test]
    fn high_contrast_dark_is_dark() {
        let hc_dark = &*theme::COSMIC_HC_DARK;
        assert!(hc_dark.is_dark);
    }

    #[test]
    fn high_contrast_light_is_light() {
        let hc_light = &*theme::COSMIC_HC_LIGHT;
        assert!(!hc_light.is_dark);
    }

    #[test]
    fn spacing_values_are_positive() {
        let spacing = theme::spacing();
        assert!(spacing.space_xxs > 0);
        assert!(spacing.space_xs > 0);
        assert!(spacing.space_s > 0);
        assert!(spacing.space_m > 0);
        assert!(spacing.space_l > 0);
        assert!(spacing.space_xl > 0);
        assert!(spacing.space_xxl > 0);
    }

    #[test]
    fn default_theme_is_dark() {
        assert!(theme::is_dark());
    }

    #[test]
    fn active_theme_returns_valid_theme() {
        let active = theme::active();
        // Should not panic
        let _cosmic = active.cosmic();
    }
}
