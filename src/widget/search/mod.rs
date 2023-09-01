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
    use crate::widget::{container, icon};
    use apply::Apply;

    /// A search button which converts to a search [`field`] on click.
    #[must_use]
    pub fn button<Message: 'static + Clone>(on_press: Message) -> crate::Element<'static, Message> {
        icon::handle::from_svg_bytes(&include_bytes!("search.svg")[..])
            .symbolic(true)
            .apply(crate::widget::button::icon)
            .on_press(on_press)
            .apply(container)
            .padding([0, 0, 0, 11])
            .into()
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
