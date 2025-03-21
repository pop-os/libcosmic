// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Request to open files and/or directories.
//!
//! Check out the [open-dialog](https://github.com/pop-os/libcosmic/tree/master/examples/open-dialog)
//! example in our repository.

#[cfg(feature = "xdg-portal")]
pub use portal::{FileResponse, MultiFileResponse, file, files, folder, folders};

#[cfg(feature = "rfd")]
pub use rust_fd::{FileResponse, MultiFileResponse, file, files, folder, folders};

use super::Error;
use std::path::PathBuf;

/// A builder for an open file dialog
#[derive(derive_setters::Setters)]
#[must_use]
pub struct Dialog {
    /// The label for the dialog's window title.
    #[setters(into)]
    title: String,

    /// The label for the accept button. Mnemonic underlines are allowed.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    accept_label: Option<String>,

    /// Sets the starting directory of the dialog.
    #[setters(into, strip_option)]
    #[allow(dead_code)] // TODO: ashpd does not expose this yet
    directory: Option<PathBuf>,

    /// Set starting file name of the dialog.
    #[setters(into, strip_option)]
    #[allow(dead_code)] // TODO: ashpd does not expose this yet
    file_name: Option<String>,

    /// Modal dialogs require user input before continuing the program.
    #[cfg(feature = "xdg-portal")]
    #[setters(skip)]
    modal: bool,

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
    pub(self) filters: Vec<super::FileFilter>,
}

impl Dialog {
    pub const fn new() -> Self {
        Self {
            title: String::new(),
            #[cfg(feature = "xdg-portal")]
            accept_label: None,
            directory: None,
            file_name: None,
            #[cfg(feature = "xdg-portal")]
            modal: true,
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

    /// Specifies the default file filter.
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

    /// Create an open file dialog.
    pub async fn open_file(self) -> Result<FileResponse, Error> {
        file(self).await
    }

    /// Create an open file dialog with multiple file select.
    pub async fn open_files(self) -> Result<MultiFileResponse, Error> {
        files(self).await
    }

    /// Create an open folder dialog.
    pub async fn open_folder(self) -> Result<FileResponse, Error> {
        folder(self).await
    }

    /// Create an open folder dialog with multi file select.
    pub async fn open_folders(self) -> Result<MultiFileResponse, Error> {
        folders(self).await
    }
}

#[cfg(feature = "xdg-portal")]
mod portal {
    use super::Dialog;
    use crate::dialog::file_chooser::Error;
    use ashpd::desktop::file_chooser::SelectedFiles;
    use url::Url;

    fn error_or_cancel(error: ashpd::Error) -> Error {
        if let ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled) = error {
            Error::Cancelled
        } else {
            Error::Open(error)
        }
    }

    /// Creates a new file dialog, and begins to await its responses.
    #[cfg(feature = "xdg-portal")]
    pub async fn create(
        dialog: super::Dialog,
        folders: bool,
        multiple: bool,
    ) -> Result<ashpd::desktop::Request<SelectedFiles>, Error> {
        // TODO: Set window identifier
        ashpd::desktop::file_chooser::OpenFileRequest::default()
            .title(Some(dialog.title.as_str()))
            .accept_label(dialog.accept_label.as_deref())
            .directory(folders)
            .modal(dialog.modal)
            .multiple(multiple)
            .choices(dialog.choices)
            .filters(dialog.filters)
            .current_filter(dialog.current_filter)
            .send()
            .await
            .map_err(error_or_cancel)
    }

    fn file_response(
        request: ashpd::desktop::Request<SelectedFiles>,
    ) -> Result<FileResponse, Error> {
        request
            .response()
            .map(FileResponse)
            .map_err(error_or_cancel)
    }

    fn multi_file_response(
        request: ashpd::desktop::Request<SelectedFiles>,
    ) -> Result<MultiFileResponse, Error> {
        request
            .response()
            .map(MultiFileResponse)
            .map_err(error_or_cancel)
    }

    pub async fn file(dialog: Dialog) -> Result<FileResponse, Error> {
        file_response(create(dialog, false, false).await?)
    }

    pub async fn files(dialog: Dialog) -> Result<MultiFileResponse, Error> {
        multi_file_response(create(dialog, false, true).await?)
    }

    pub async fn folder(dialog: Dialog) -> Result<FileResponse, Error> {
        file_response(create(dialog, true, false).await?)
    }

    pub async fn folders(dialog: Dialog) -> Result<MultiFileResponse, Error> {
        multi_file_response(create(dialog, true, true).await?)
    }

    /// A dialog response containing the selected file or folder.
    pub struct FileResponse(pub SelectedFiles);

    impl FileResponse {
        pub fn choices(&self) -> &[(String, String)] {
            self.0.choices()
        }

        pub fn url(&self) -> &Url {
            self.0.uris().first().expect("no files selected")
        }
    }

    /// A dialog response containing the selected file(s) or folder(s).
    pub struct MultiFileResponse(pub SelectedFiles);

    impl MultiFileResponse {
        pub fn choices(&self) -> &[(String, String)] {
            self.0.choices()
        }

        pub fn urls(&self) -> &[Url] {
            self.0.uris()
        }
    }
}

#[cfg(feature = "rfd")]
mod rust_fd {
    use super::Dialog;
    use crate::dialog::file_chooser::Error;
    use url::Url;

    pub fn create(dialog: Dialog) -> rfd::AsyncFileDialog {
        let mut builder = rfd::AsyncFileDialog::new().set_title(dialog.title);

        if let Some(directory) = dialog.directory {
            builder = builder.set_directory(directory);
        }

        if let Some(file_name) = dialog.file_name {
            builder = builder.set_file_name(file_name);
        }

        for filter in dialog.filters {
            builder = builder.add_filter(filter.description, &filter.extensions);
        }

        builder
    }

    fn file_response(request: Option<rfd::FileHandle>) -> Result<FileResponse, Error> {
        if let Some(handle) = request {
            let url = Url::from_file_path(handle.path()).map_err(|_| Error::UrlAbsolute)?;

            return Ok(FileResponse(url));
        }

        Err(Error::Cancelled)
    }

    fn multi_file_response(
        request: Option<Vec<rfd::FileHandle>>,
    ) -> Result<MultiFileResponse, Error> {
        if let Some(handles) = request {
            let mut urls = Vec::with_capacity(handles.len());

            for handle in &handles {
                urls.push(Url::from_file_path(handle.path()).map_err(|()| Error::UrlAbsolute)?);
            }

            return Ok(MultiFileResponse(urls));
        }

        Err(Error::Cancelled)
    }

    pub async fn file(dialog: Dialog) -> Result<FileResponse, Error> {
        file_response(create(dialog).pick_file().await)
    }

    pub async fn files(dialog: Dialog) -> Result<MultiFileResponse, Error> {
        multi_file_response(create(dialog).pick_files().await)
    }

    pub async fn folder(dialog: Dialog) -> Result<FileResponse, Error> {
        file_response(create(dialog).pick_folder().await)
    }

    pub async fn folders(dialog: Dialog) -> Result<MultiFileResponse, Error> {
        multi_file_response(create(dialog).pick_folders().await)
    }

    /// A dialog response containing the selected file or folder.
    pub struct FileResponse(Url);

    impl FileResponse {
        pub fn choices(&self) -> &[(String, String)] {
            &[]
        }

        pub fn url(&self) -> &Url {
            &self.0
        }
    }

    /// A dialog response containing the selected file(s) or folder(s).
    pub struct MultiFileResponse(Vec<Url>);

    impl MultiFileResponse {
        pub fn choices(&self) -> &[(String, String)] {
            &[]
        }

        pub fn urls(&self) -> &[Url] {
            &self.0
        }
    }
}
