// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Track the cursor of a text input.
use super::value::Value;

/// The cursor of a text input.
#[derive(Debug, Copy, Clone)]
pub struct Cursor {
    state: State,
}

/// The state of a [`Cursor`].
#[derive(Debug, Copy, Clone)]
pub enum State {
    /// Cursor without a selection
    Index(usize),

    /// Cursor selecting a range of text
    Selection {
        /// The start of the selection
        start: usize,
        /// The end of the selection
        end: usize,
    },
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self {
            state: State::Index(0),
        }
    }
}

impl Cursor {
    /// Returns the [`State`] of the [`Cursor`].
    #[must_use]
    #[inline(never)]
    pub fn state(&self, value: &Value) -> State {
        match self.state {
            State::Index(index) => State::Index(index.min(value.len())),
            State::Selection { start, end } => {
                let start = start.min(value.len());
                let end = end.min(value.len());

                if start == end {
                    State::Index(start)
                } else {
                    State::Selection { start, end }
                }
            }
        }
    }

    /// Returns the current selection of the [`Cursor`] for the given [`Value`].
    ///
    /// `start` is guaranteed to be <= than `end`.
    #[must_use]
    #[inline]
    pub fn selection(&self, value: &Value) -> Option<(usize, usize)> {
        match self.state(value) {
            State::Selection { start, end } => Some((start.min(end), start.max(end))),
            State::Index(_) => None,
        }
    }

    #[inline]
    pub(crate) fn move_to(&mut self, position: usize) {
        self.state = State::Index(position);
    }

    #[inline]
    pub(crate) fn move_right(&mut self, value: &Value) {
        self.move_right_by_amount(value, 1);
    }

    #[inline]
    pub(crate) fn move_right_by_words(&mut self, value: &Value) {
        self.move_to(value.next_end_of_word(self.right(value)));
    }

    #[inline]
    pub(crate) fn move_right_by_amount(&mut self, value: &Value, amount: usize) {
        match self.state(value) {
            State::Index(index) => self.move_to(index.saturating_add(amount).min(value.len())),
            State::Selection { start, end } => self.move_to(end.max(start)),
        }
    }

    #[inline]
    pub(crate) fn move_left(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index > 0 => self.move_to(index - 1),
            State::Selection { start, end } => self.move_to(start.min(end)),
            State::Index(_) => self.move_to(0),
        }
    }

    #[inline]
    pub(crate) fn move_left_by_words(&mut self, value: &Value) {
        self.move_to(value.previous_start_of_word(self.left(value)));
    }

    #[inline]
    pub(crate) fn select_range(&mut self, start: usize, end: usize) {
        self.state = if start == end {
            State::Index(start)
        } else {
            State::Selection { start, end }
        };
    }

    #[inline]
    pub(crate) fn select_left(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index > 0 => self.select_range(index, index - 1),
            State::Selection { start, end } if end > 0 => self.select_range(start, end - 1),
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn select_right(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) if index < value.len() => self.select_range(index, index + 1),
            State::Selection { start, end } if end < value.len() => {
                self.select_range(start, end + 1);
            }
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn select_left_by_words(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) => self.select_range(index, value.previous_start_of_word(index)),
            State::Selection { start, end } => {
                self.select_range(start, value.previous_start_of_word(end));
            }
        }
    }

    #[inline]
    pub(crate) fn select_right_by_words(&mut self, value: &Value) {
        match self.state(value) {
            State::Index(index) => self.select_range(index, value.next_end_of_word(index)),
            State::Selection { start, end } => {
                self.select_range(start, value.next_end_of_word(end));
            }
        }
    }

    #[inline]
    pub(crate) fn select_all(&mut self, value: &Value) {
        self.select_range(0, value.len());
    }

    #[inline]
    pub(crate) fn start(&self, value: &Value) -> usize {
        let start = match self.state {
            State::Index(index) => index,
            State::Selection { start, .. } => start,
        };

        start.min(value.len())
    }

    #[inline]
    pub(crate) fn end(&self, value: &Value) -> usize {
        let end = match self.state {
            State::Index(index) => index,
            State::Selection { end, .. } => end,
        };

        end.min(value.len())
    }

    #[inline]
    fn left(&self, value: &Value) -> usize {
        match self.state(value) {
            State::Index(index) => index,
            State::Selection { start, end } => start.min(end),
        }
    }

    #[inline]
    fn right(&self, value: &Value) -> usize {
        match self.state(value) {
            State::Index(index) => index,
            State::Selection { start, end } => start.max(end),
        }
    }
}
