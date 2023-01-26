// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::State;
use crate::iced;

/// A model for managing the state of a search widget.
pub struct Model {
    pub input_id: iced::widget::text_input::Id,
    pub phrase: String,
    pub state: State,
}

impl Model {
    /// Focuses the search field.
    #[must_use]
    pub fn focus<Message: 'static>(&mut self) -> crate::iced::Command<Message> {
        self.state = State::Active;
        iced::widget::text_input::focus(self.input_id.clone())
    }

    /// Check if the search field is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state == State::Active
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            input_id: iced::widget::text_input::Id::unique(),
            phrase: String::with_capacity(32),
            state: State::Inactive,
        }
    }
}
