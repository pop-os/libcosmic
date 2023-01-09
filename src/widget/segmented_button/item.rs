// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Defines the model that's used by each button in the widget.

use crate::widget::IconSource;
use derive_setters::Setters;
use std::borrow::Cow;

/// Describes a button in a segmented button
#[must_use]
pub fn item() -> SegmentedItem {
    SegmentedItem::default()
}

/// Information about a specific button in a segmented button
#[derive(Setters)]
pub struct SegmentedItem {
    #[setters(into, strip_option)]
    /// The label to display in this button.
    pub text: Option<Cow<'static, str>>,

    #[setters(into, strip_option)]
    /// An optionally-displayed icon beside the label.
    pub icon: Option<IconSource<'static>>,

    /// Whether the button is clickable or not.
    pub enabled: bool,
}

impl Default for SegmentedItem {
    fn default() -> Self {
        Self {
            text: None,
            icon: None,
            enabled: true,
        }
    }
}

impl From<String> for SegmentedItem {
    fn from(text: String) -> Self {
        Self::from(Cow::Owned(text))
    }
}

impl From<&'static str> for SegmentedItem {
    fn from(text: &'static str) -> Self {
        Self::from(Cow::Borrowed(text))
    }
}

impl From<Cow<'static, str>> for SegmentedItem {
    fn from(text: Cow<'static, str>) -> Self {
        SegmentedItem::default().text(text)
    }
}
