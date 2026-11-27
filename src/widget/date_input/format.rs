// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use jiff::civil::Date;
use jiff::fmt::strtime;

use crate::fl;

/// A date format accepted by [`super::date_input`].
pub struct Format<'a> {
    format: &'a str,
    placeholder: &'a str,
}

impl<'a> Format<'a> {
    /// The ISO 8601 calendar date format, `YYYY-MM-DD`.
    pub const ISO: Self = Self::strftime("%Y-%m-%d", "YYYY-MM-DD");

    /// A `strftime`-style format and its display placeholder.
    ///
    /// The placeholder is used for auto-inserting separators while typing and
    /// for the validation message.
    pub const fn strftime(format: &'a str, placeholder: &'a str) -> Self {
        Self {
            format,
            placeholder,
        }
    }

    pub(super) fn placeholder(&self) -> &'a str {
        self.placeholder
    }

    pub(super) fn parse(&self, value: &str) -> Option<Date> {
        strtime::parse(self.format, value).ok()?.to_date().ok()
    }

    pub(super) fn format(&self, date: Date) -> Option<String> {
        Some(date.strftime(self.format).to_string())
    }
}

/// A placeholder-derived template for inserting date separators while typing.
///
/// For example, `YYYY-MM-DD` becomes segments `[4, 2, 2]` and separators
/// `["-", "-"]`. Input is reduced to the accepted characters and redistributed
/// into those segments, so `20261126` becomes `2026-11-26`.
#[derive(Debug)]
struct Mask {
    /// Character counts for each date component in the placeholder.
    segments: Vec<usize>,
    /// Literal separators between consecutive date components.
    separators: Vec<String>,
    /// Whether only ASCII digits should be kept from the input.
    digits_only: bool,
}

impl Mask {
    /// Builds a mask from a placeholder such as `YYYY-MM-DD`.
    ///
    /// Returns `None` when the placeholder cannot be split into alphanumeric
    /// segments separated by literal text.
    fn from_placeholder(placeholder: &str) -> Option<Self> {
        let mut chars = placeholder.chars().peekable();
        let mut segments = Vec::new();
        let mut separators = Vec::new();

        while chars.peek().is_some() {
            let mut segment = String::new();
            while let Some(char) = chars.peek().copied() {
                if !char.is_ascii_alphanumeric() {
                    break;
                }
                segment.push(char);
                chars.next();
            }
            if segment.is_empty() {
                return None;
            }
            segments.push(segment.chars().count());

            let mut separator = String::new();
            while let Some(char) = chars.peek().copied() {
                if char.is_ascii_alphanumeric() {
                    break;
                }
                separator.push(char);
                chars.next();
            }
            if separator.is_empty() {
                break;
            }
            separators.push(separator);
        }

        (segments.len() == separators.len() + 1).then_some(Self {
            segments,
            separators,
            digits_only: placeholder
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .all(|c| matches!(c, 'Y' | 'M' | 'D')),
        })
    }

    /// Normalizes `value` into the mask's shape.
    ///
    /// Extra characters are dropped, mismatched separators are replaced with
    /// the configured ones, and a trailing separator is preserved when the
    /// user has just typed one.
    fn apply(&self, value: &str) -> String {
        let total: usize = self.segments.iter().sum();
        let values: Vec<char> = value
            .chars()
            .filter(|c| {
                if self.digits_only {
                    c.is_ascii_digit()
                } else {
                    c.is_ascii_alphanumeric()
                }
            })
            .take(total)
            .collect();

        let mut formatted = String::new();
        let mut index = 0;
        for (segment, len) in self.segments.iter().enumerate() {
            if index == values.len() {
                break;
            }
            if segment > 0
                && let Some(separator) = self.separators.get(segment - 1)
            {
                formatted.push_str(separator);
            }
            for char in &values[index..(index + *len).min(values.len())] {
                formatted.push(*char);
            }
            index = (index + *len).min(values.len());
        }

        let mut filled = 0;
        for (index, len) in self
            .segments
            .iter()
            .enumerate()
            .take(self.segments.len() - 1)
        {
            filled += *len;
            if values.len() == filled && value.ends_with(self.separators[index].as_str()) {
                formatted.push_str(&self.separators[index]);
            }
        }

        formatted
    }
}

pub(super) fn sanitize(value: &str, formats: &[Format<'_>]) -> String {
    if let [format] = formats
        && let Some(mask) = Mask::from_placeholder(format.placeholder())
    {
        return mask.apply(value);
    }

    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "-/. ,".contains(*c))
        .take(max_placeholder_len(formats))
        .collect()
}

pub(super) fn format_error(formats: &[Format<'_>]) -> String {
    let formats = formats
        .iter()
        .map(Format::placeholder)
        .collect::<Vec<_>>()
        .join(", ");
    fl!("invalid-date", formats = formats)
}

pub(super) fn min_placeholder_len(formats: &[Format<'_>]) -> usize {
    formats
        .iter()
        .map(Format::placeholder)
        .map(|placeholder| placeholder.chars().count())
        .min()
        .unwrap_or(10)
        .max(1)
}

fn max_placeholder_len(formats: &[Format<'_>]) -> usize {
    formats
        .iter()
        .map(Format::placeholder)
        .map(|placeholder| placeholder.chars().count())
        .max()
        .unwrap_or(10)
        .max(1)
}
