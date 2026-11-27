// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use jiff::civil::Date;

use crate::widget::popover;

use super::format::{Format, format_error, min_placeholder_len};
use super::locale::locale_formats;

type ValidateFn<'a> = Box<dyn Fn(Date) -> Result<(), String> + 'a>;

/// Configuration for [`super::date_input`].
pub struct Options<'a> {
    formats: &'a [Format<'a>],
    validate: Option<ValidateFn<'a>>,
    position: popover::Position,
}

impl<'a> Options<'a> {
    /// Accepts the given formats, in any order.
    #[must_use]
    pub fn new(formats: &'a [Format<'a>]) -> Self {
        Self {
            formats,
            validate: None,
            position: popover::Position::Bottom,
        }
    }

    /// Accepts the date format detected from the system locale.
    #[must_use]
    pub fn auto() -> Self {
        Self::new(locale_formats())
    }

    /// Places the calendar popover relative to the input.
    ///
    /// Defaults to [`popover::Position::Bottom`].
    #[must_use]
    pub fn popover_position(mut self, position: popover::Position) -> Self {
        self.position = position;
        self
    }

    /// Validates parsed dates, using the returned message for invalid dates.
    #[must_use]
    pub fn validate(mut self, validate: impl Fn(Date) -> Result<(), String> + 'a) -> Self {
        self.validate = Some(Box::new(validate));
        self
    }

    pub(super) fn formats(&self) -> &'a [Format<'a>] {
        self.formats
    }

    pub(super) fn position(&self) -> popover::Position {
        self.position
    }

    pub(super) fn placeholder(&self) -> String {
        if self.formats.is_empty() {
            "YYYY-MM-DD".to_owned()
        } else {
            self.formats
                .iter()
                .map(Format::placeholder)
                .collect::<Vec<_>>()
                .join(" / ")
        }
    }

    /// Returns the validation error for a completed date, if any.
    #[must_use]
    pub fn validation_message(&self, value: &str) -> Option<String> {
        if value.is_empty() || value.chars().count() < min_placeholder_len(self.formats) {
            return None;
        }
        self.validate_result(value).err()
    }

    /// Formats a date using the first configured format that can format it.
    #[must_use]
    pub fn format(&self, date: Date) -> Option<String> {
        self.formats.iter().find_map(|format| format.format(date))
    }

    /// Parses a date using the configured formats and validation.
    #[must_use]
    pub fn parse(&self, value: &str) -> Option<Date> {
        self.validate_result(value).ok()
    }

    fn validate_result(&self, value: &str) -> Result<Date, String> {
        for format in self.formats {
            if let Some(date) = format.parse(value) {
                return match &self.validate {
                    Some(validate) => validate(date).map(|()| date),
                    None => Ok(date),
                };
            }
        }
        Err(format_error(self.formats))
    }
}

impl Default for Options<'static> {
    fn default() -> Self {
        Self::new(&[Format::ISO])
    }
}
