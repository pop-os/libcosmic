// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Choose a location to save a file to.
//!
//! Check out the [open-dialog](https://github.com/pop-os/libcosmic/tree/master/examples/open-dialog)
//! example in our repository.

#[cfg(feature = "xdg-portal")]
pub use portal::{Response, file};

#[cfg(feature = "rfd")]
pub use rust_fd::{Response, file};

use super::Error;
use std::path::PathBuf;

/// A builder for an save file dialog.
#[derive(derive_setters::Setters)]
#[must_use]
pub struct Dialog {
    /// The label for the dialog's window title.
    title: String,

    /// The label for the accept button. Mnemonic underlines are allowed.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    accept_label: Option<String>,

    /// Modal dialogs require user input before continuing the program.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    modal: bool,

    /// Set starting file name of the dialog.
    #[setters(strip_option)]
    file_name: Option<String>,

    /// Sets the starting directory of the dialog.
    #[setters(strip_option)]
    directory: Option<PathBuf>,

    /// Sets the absolute path of the file
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    current_file: Option<PathBuf>,

    /// Adds a list of choices.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    choices: Vec<super::Choice>,

    /// Specifies the default file filter.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    current_filter: Option<super::FileFilter>,

    /// A collection of file filters.
    #[setters(skip)]
    filters: Vec<super::FileFilter>,
}

impl Dialog {
    pub const fn new() -> Self {
        Self {
            title: String::new(),
            #[cfg(feature = "xdg-portal")]
            accept_label: None,
            #[cfg(feature = "xdg-portal")]
            modal: true,
            file_name: None,
            directory: None,
            #[cfg(feature = "xdg-portal")]
            current_file: None,
            #[cfg(feature = "xdg-portal")]
            current_filter: None,
            #[cfg(feature = "xdg-portal")]
            choices: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// The label for the accept button. Mnemonic underlines are allowed.
    #[cfg(feature = "xdg-portal")]
    pub fn accept_label(mut self, label: impl Into<String>) -> Self {
        self.accept_label = Some(label.into());
        self
    }

    /// Adds a choice.
    #[cfg(feature = "xdg-portal")]
    pub fn choice(mut self, choice: impl Into<super::Choice>) -> Self {
        self.choices.push(choice.into());
        self
    }

    /// Set the current file filter.
    #[cfg(feature = "xdg-portal")]
    pub fn current_filter(mut self, filter: impl Into<super::FileFilter>) -> Self {
        self.current_filter = Some(filter.into());
        self
    }

    /// Adds a files filter.
    pub fn filter(mut self, filter: impl Into<super::FileFilter>) -> Self {
        self.filters.push(filter.into());
        self
    }

    /// Modal dialogs require user input before continuing the program.
    #[cfg(feature = "xdg-portal")]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Create a save file dialog request.
    pub async fn save_file(self) -> Result<Response, Error> {
        file(self).await
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "xdg-portal")]
mod portal {
    use super::Dialog;
    use crate::dialog::file_chooser::Error;
    use ashpd::desktop::file_chooser::SelectedFiles;
    use std::path::Path;
    use url::Url;

    /// Create a save file dialog request.
    pub async fn file(dialog: Dialog) -> Result<Response, Error> {
        ashpd::desktop::file_chooser::SaveFileRequest::default()
            .title(Some(dialog.title.as_str()))
            .accept_label(dialog.accept_label.as_deref())
            .modal(dialog.modal)
            .choices(dialog.choices)
            .filters(dialog.filters)
            .current_filter(dialog.current_filter)
            .current_name(dialog.file_name.as_deref())
            .current_folder::<&Path>(dialog.directory.as_deref())
            .map_err(Error::SetDirectory)?
            .current_file::<&Path>(dialog.current_file.as_deref())
            .map_err(Error::SetAbsolutePath)?
            .send()
            .await
            .map_err(Error::Save)?
            .response()
            .map_err(Error::Save)
            .map(Response)
    }

    /// A dialog response containing the selected file or folder.
    pub struct Response(pub SelectedFiles);

    impl Response {
        pub fn choices(&self) -> &[(String, String)] {
            self.0.choices()
        }

        pub fn url(&self) -> Option<&Url> {
            self.0.uris().first()
        }
    }
}

#[cfg(feature = "rfd")]
mod rust_fd {
    use super::Dialog;
    use crate::dialog::file_chooser::Error;
    use url::Url;

    /// Create a save file dialog request.
    pub async fn file(dialog: Dialog) -> Result<Response, Error> {
        let mut request = rfd::AsyncFileDialog::new().set_title(dialog.title);

        if let Some(directory) = dialog.directory {
            request = request.set_directory(directory);
        }

        if let Some(file_name) = dialog.file_name {
            request = request.set_file_name(file_name);
        }

        for filter in dialog.filters {
            request = request.add_filter(filter.description, &filter.extensions);
        }

        if let Some(handle) = request.save_file().await {
            let url = Url::from_file_path(handle.path()).map_err(|_| Error::UrlAbsolute)?;

            return Ok(Response(Some(url)));
        }

        Ok(Response(None))
    }

    /// A dialog response containing the selected file or folder.
    pub struct Response(Option<Url>);

    impl Response {
        pub fn url(&self) -> Option<&Url> {
            self.0.as_ref()
        }
    }
}
