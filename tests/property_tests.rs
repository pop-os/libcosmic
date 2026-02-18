// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Property-based tests for COSMIC data models and utility functions.
//!
//! Uses proptest to generate random inputs and verify that invariants hold
//! across a wide range of values. This catches edge cases that hand-written
//! unit tests often miss.

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Spin button property tests
// ---------------------------------------------------------------------------
mod spin_button_props {
    use super::*;

    // We can't directly test the private increment/decrement functions,
    // but we can replicate their logic and verify the same invariants.

    fn increment(value: i32, step: i32, _min: i32, max: i32) -> i32 {
        if value > max - step {
            max
        } else {
            value + step
        }
    }

    fn decrement(value: i32, step: i32, min: i32, _max: i32) -> i32 {
        if value < min + step {
            min
        } else {
            value - step
        }
    }

    proptest! {
        #[test]
        fn increment_never_exceeds_max(
            value in -1000i32..1000,
            step in 1i32..100,
            max in 0i32..2000,
        ) {
            let min = -1000;
            let result = increment(value, step, min, max);
            prop_assert!(result <= max, "increment({value}, {step}) = {result} > max({max})");
        }

        #[test]
        fn decrement_never_goes_below_min(
            value in -1000i32..1000,
            step in 1i32..100,
            min in -2000i32..0,
        ) {
            let max = 1000;
            let result = decrement(value, step, min, max);
            prop_assert!(result >= min, "decrement({value}, {step}) = {result} < min({min})");
        }

        #[test]
        fn increment_at_max_stays_at_max(
            step in 1i32..100,
            max in 0i32..1000,
        ) {
            let min = -1000;
            let result = increment(max, step, min, max);
            prop_assert_eq!(result, max);
        }

        #[test]
        fn decrement_at_min_stays_at_min(
            step in 1i32..100,
            min in -1000i32..0,
        ) {
            let max = 1000;
            let result = decrement(min, step, min, max);
            prop_assert_eq!(result, min);
        }

        #[test]
        fn increment_then_decrement_returns_to_original_or_clamped(
            value in -500i32..500,
            step in 1i32..50,
        ) {
            let min = -1000;
            let max = 1000;
            let incremented = increment(value, step, min, max);
            let round_tripped = decrement(incremented, step, min, max);
            // If we didn't hit the max, we should return to original
            if value <= max - step {
                prop_assert_eq!(round_tripped, value);
            } else {
                prop_assert!(round_tripped >= value || round_tripped >= min);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Calendar property tests
// ---------------------------------------------------------------------------
mod calendar_props {
    use super::*;
    use chrono::{Datelike, NaiveDate, Weekday};
    use cosmic::widget::calendar::{get_calender_first, set_day};

    proptest! {
        #[test]
        fn get_calendar_first_returns_correct_weekday(
            year in 2000i32..2100,
            month in 1u32..=12,
            weekday_idx in 0u32..7,
        ) {
            let weekday = match weekday_idx {
                0 => Weekday::Mon,
                1 => Weekday::Tue,
                2 => Weekday::Wed,
                3 => Weekday::Thu,
                4 => Weekday::Fri,
                5 => Weekday::Sat,
                _ => Weekday::Sun,
            };
            let first = get_calender_first(year, month, weekday);
            prop_assert_eq!(first.weekday(), weekday);
        }

        #[test]
        fn get_calendar_first_is_before_or_at_month_start(
            year in 2000i32..2100,
            month in 1u32..=12,
        ) {
            let first = get_calender_first(year, month, Weekday::Mon);
            let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            prop_assert!(first <= month_start,
                "Calendar first {first} is after month start {month_start}");
        }

        #[test]
        fn get_calendar_first_is_within_one_week_of_month_start(
            year in 2000i32..2100,
            month in 1u32..=12,
        ) {
            let first = get_calender_first(year, month, Weekday::Mon);
            let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let diff = (month_start - first).num_days();
            prop_assert!(diff >= 0 && diff < 7,
                "Calendar first {first} is {diff} days before month start {month_start}");
        }

        #[test]
        fn set_day_preserves_month_and_year(
            month in 1u32..=12,
            from_day in 1u32..=28,
            to_day in 1u32..=28,
        ) {
            let date = NaiveDate::from_ymd_opt(2024, month, from_day).unwrap();
            let result = set_day(date, to_day);
            prop_assert_eq!(result.month(), month);
            prop_assert_eq!(result.year(), 2024);
        }

        #[test]
        fn set_day_idempotent(
            month in 1u32..=12,
            day in 1u32..=28,
        ) {
            let date = NaiveDate::from_ymd_opt(2024, month, day).unwrap();
            let result = set_day(date, day);
            prop_assert_eq!(result, date);
        }
    }
}

// ---------------------------------------------------------------------------
// Segmented button model property tests
// ---------------------------------------------------------------------------
mod segmented_model_props {
    use super::*;
    use cosmic::widget::segmented_button::{Model, SingleSelect, MultiSelect};

    proptest! {
        #[test]
        fn model_length_matches_insertions(count in 0usize..50) {
            let mut builder = Model::<SingleSelect>::builder();
            for i in 0..count {
                builder = builder.insert(move |b| b.text(format!("Item {i}")));
            }
            let model = builder.build();
            prop_assert_eq!(model.len(), count);
        }

        #[test]
        fn clear_always_empties_model(count in 1usize..20) {
            let mut builder = Model::<SingleSelect>::builder();
            for i in 0..count {
                builder = builder.insert(move |b| b.text(format!("Item {i}")));
            }
            let mut model = builder.build();
            model.clear();
            prop_assert_eq!(model.len(), 0);
        }

        #[test]
        fn multi_select_activate_is_toggle(count in 2usize..10) {
            let mut builder = Model::<MultiSelect>::builder();
            for i in 0..count {
                builder = builder.insert(move |b| b.text(format!("Item {i}")));
            }
            let mut model = builder.build();

            // Collect entities from the model
            let ids: Vec<_> = model.iter().collect();

            // Activate first item
            model.activate(ids[0]);
            prop_assert!(model.is_active(ids[0]));

            // Toggle it off
            model.activate(ids[0]);
            prop_assert!(!model.is_active(ids[0]));
        }
    }
}

// ---------------------------------------------------------------------------
// Theme spacing property tests
// ---------------------------------------------------------------------------
mod theme_props {
    use super::*;

    #[test]
    fn spacing_values_monotonically_increase() {
        let spacing = cosmic::theme::spacing();
        // xxs < xs < s < m < l < xl < xxl
        assert!(spacing.space_xxs <= spacing.space_xs);
        assert!(spacing.space_xs <= spacing.space_s);
        assert!(spacing.space_s <= spacing.space_m);
        assert!(spacing.space_m <= spacing.space_l);
        assert!(spacing.space_l <= spacing.space_xl);
        assert!(spacing.space_xl <= spacing.space_xxl);
    }

    proptest! {
        #[test]
        fn cosmic_theme_dark_default_is_valid(_dummy in 0..1u8) {
            let theme = cosmic_theme::Theme::dark_default();
            prop_assert!(theme.is_dark);
            prop_assert!(theme.spacing.space_xxs > 0);
        }

        #[test]
        fn cosmic_theme_light_default_is_valid(_dummy in 0..1u8) {
            let theme = cosmic_theme::Theme::light_default();
            prop_assert!(!theme.is_dark);
            prop_assert!(theme.spacing.space_xxs > 0);
        }
    }
}
