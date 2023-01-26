// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A COSMIC search widget
//!
//! ## Example
//!
//! Store the model in the application:
//!
//! ```ignore
//! App {
//!     search: search::Model::default()
//! }
//! ```
//!
//! Generate the element in the view:
//!
//! ```ignore
//! let search_field = search::search(&self.search, Message::Search);
//! ```
//!
//! Handle messages in the update method:
//!
//! ```ignore
//! match message {
//!     Message::Search(search::Message::Activate) => {
//!         // Returns command to focus the text input.
//!         return self.search.focus();
//!     }
//!     Message::Search(search::Message::Changed) => {
//!         self.search.phrase = phrase;
//!         self.search_changed();
//!     }
//!     Message::Search(search::Message::Clear) => {
//!         self.search_clear();
//!     },
//!     Message::Search(search::Message::Submit) => {
//!         self.search_submit();
//!     }
//! }

mod field;
mod model;

mod button {
    use crate::iced::{self, widget::container};
    use apply::Apply;

    /// A search button which converts to a search [`field`] on click.
    #[must_use]
    pub fn button<Message: 'static + Clone>(on_press: Message) -> crate::Element<'static, Message> {
        super::icon::search(16)
            .style(crate::theme::Svg::SymbolicActive)
            .apply(iced::widget::button)
            .style(crate::theme::Button::Text)
            .on_press(on_press)
            .apply(container)
            .padding([0, 0, 0, 11])
            .into()
    }
}

pub mod icon {
    use crate::widget::IconSource;

    #[must_use]
    pub fn search(size: u16) -> crate::widget::Icon<'static> {
        crate::widget::icon(
            IconSource::svg_from_memory(&include_bytes!("search.svg")[..]),
            size,
        )
    }

    #[must_use]
    pub fn edit_clear(size: u16) -> crate::widget::Icon<'static> {
        crate::widget::icon(IconSource::from("edit-clear-symbolic"), size)
    }
}

pub use button::button;
pub use field::{field, Field};
pub use model::Model;

/// Creates the COSMIC search field widget
///
/// A button is displayed when inactive, and the search field when active.
pub fn search<M: 'static + Clone>(model: &Model, on_emit: fn(Message) -> M) -> crate::Element<M> {
    let element = match model.state {
        State::Active => field(
            model.input_id.clone(),
            &model.phrase,
            Message::Changed,
            Message::Clear,
            Some(Message::Clear),
        )
        .into(),

        State::Inactive => button(Message::Activate),
    };

    element.map(on_emit)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Activate,
    Changed(String),
    Clear,
    Submit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Inactive,
}
