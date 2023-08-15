// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Choose a location to save a file to.
//!
//! Check out the [open-dialog](https://github.com/pop-os/libcosmic/tree/master/examples/open-dialog)
//! example in our repository.

use derive_setters::Setters;
use iced::Command;
use std::path::{Path, PathBuf};

/// A builder for an save file dialog, passed as a request by a [`Sender`]
#[derive(Setters)]
#[must_use]
pub struct Dialog {
    /// The label for the dialog's window title.
    title: String,

    /// The label for the accept button. Mnemonic underlines are allowed.
    #[setters(strip_option)]
    accept_label: Option<String>,

    /// Modal dialogs require user input before continuing the program.
    modal: bool,

    /// Sets the current file name.
    #[setters(strip_option)]
    current_name: Option<String>,

    /// Sets the current folder.
    #[setters(strip_option)]
    current_folder: Option<PathBuf>,

    /// Sets the absolute path of the file
    #[setters(strip_option)]
    current_file: Option<PathBuf>,

    /// Adds a list of choices.
    choices: Vec<super::Choice>,

    /// Specifies the default file filter.
    #[setters(into)]
    current_filter: Option<super::FileFilter>,

    /// A collection of file filters.
    filters: Vec<super::FileFilter>,
}

impl Dialog {
    pub(super) const fn new() -> Self {
        Self {
            title: String::new(),
            accept_label: None,
            modal: true,
            current_name: None,
            current_folder: None,
            current_file: None,
            current_filter: None,
            choices: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Creates a [`Command`] which opens the dialog.
    pub fn create(self, sender: &mut super::Sender) -> Command<()> {
        sender.save(self)
    }

    /// Adds a choice.
    pub fn choice(mut self, choice: impl Into<super::Choice>) -> Self {
        self.choices.push(choice.into());
        self
    }

    /// Adds a files filter.
    pub fn filter(mut self, filter: impl Into<super::FileFilter>) -> Self {
        self.filters.push(filter.into());
        self
    }
}

/// Creates a new file dialog, and begins to await its responses.
pub(super) async fn create(
    dialog: Dialog,
) -> ashpd::Result<ashpd::desktop::Request<super::SelectedFiles>> {
    ashpd::desktop::file_chooser::SaveFileRequest::default()
        .title(Some(dialog.title.as_str()))
        .accept_label(dialog.accept_label.as_deref())
        .modal(dialog.modal)
        .choices(dialog.choices)
        .filters(dialog.filters)
        .current_filter(dialog.current_filter)
        .current_name(dialog.current_name.as_deref())
        .current_folder::<&Path>(dialog.current_folder.as_deref())?
        .current_file::<&Path>(dialog.current_file.as_deref())?
        .send()
        .await
}
