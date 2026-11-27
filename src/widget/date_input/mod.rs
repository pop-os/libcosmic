// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A text input for dates, with an optional calendar popup.

mod format;
mod locale;
mod options;
mod state;

pub use format::Format;
pub use options::Options;
pub use state::DateInputState;

use crate::{
    Element,
    widget::{self, popover},
};
use iced_core::Length;
use jiff::civil::{Date, Weekday};

/// An event produced by [`date_input`].
#[derive(Clone, Debug)]
pub enum Event {
    /// The text value changed.
    Input(String),
    /// Inline editing started or stopped.
    Edit(bool),
    /// The input gained focus.
    Focus,
    /// The input lost focus.
    Unfocus,
    /// The calendar popup requested to close.
    Close,
    /// The previous month was requested.
    PreviousMonth,
    /// The next month was requested.
    NextMonth,
    /// A date was selected from the calendar.
    Select(Date),
}

/// Creates a date input with an optional calendar popup.
///
/// Forward the returned [`Event`]s to [`DateInputState::update`].
pub fn date_input<'a, Message: Clone + 'static>(
    value: &'a str,
    state: &'a DateInputState,
    options: Options<'_>,
    on_event: fn(Event) -> Message,
) -> Element<'a, Message> {
    let placeholder = options.placeholder();
    let message = options.validation_message(value);
    let position = options.position();

    let mut input =
        widget::editable_input(placeholder, value, state.is_editing(), move |editing| {
            on_event(Event::Edit(editing))
        })
        .width(Length::Fill)
        .on_focus(on_event(Event::Focus))
        .on_unfocus(on_event(Event::Unfocus))
        .on_input(move |input| on_event(Event::Input(input)))
        .on_submit(move |_| on_event(Event::Edit(false)));

    if let Some(message) = message {
        input = input.error(message);
    }

    let mut input = popover(input)
        .position(position)
        .on_close(on_event(Event::Close));
    if let Some(model) = state.calendar() {
        input = input.popup(
            widget::container(widget::calendar(
                model,
                move |date| on_event(Event::Select(date)),
                move || on_event(Event::PreviousMonth),
                move || on_event(Event::NextMonth),
                Weekday::Monday,
            ))
            .padding(crate::theme::spacing().space_xs)
            .class(crate::theme::Container::Card),
        );
    }

    input.into()
}
