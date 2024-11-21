// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Dialogs for opening and save files.
//!
//! # Features
//!
//! - On Linux, the `xdg-portal` feature will use XDG Portal dialogs.
//! - Alternatively, `rfd` can be used for platform support beyond Linux.
//!
//! # Open a file
//!
//! ```no_run
//! cosmic::task::future(async {
//!     use cosmic::dialog::file_chooser;
//!
//!     let dialog = file_chooser::open::Dialog::new()
//!         .title("Choose a file");
//!
//!     match dialog.open_file().await {
//!         Ok(response) => println!("selected to open {:?}", response.url()),
//!
//!         Err(file_chooser::Error::Cancelled) => (),
//!
//!         Err(why) => eprintln!("error selecting file to open: {why:?}")
//!     }
//! });
//! ```
//!
//! # Open multiple files
//!
//! ```no_run
//! cosmic::task::future(async {
//!     use cosmic::dialog::file_chooser;
//!
//!     let dialog = file_chooser::open::Dialog::new()
//!         .title("Choose multiple files");
//!
//!     match dialog.open_files().await {
//!         Ok(response) => println!("selected to open {:?}", response.urls()),
//!
//!         Err(file_chooser::Error::Cancelled) => (),
//!
//!         Err(why) => eprintln!("error selecting file(s) to open: {why:?}")
//!     }
//! });
//! ```
//!
//! # Open a folder
//!
//! ```no_run
//! cosmic::task::future(async {
//!     use cosmic::dialog::file_chooser;
//!
//!     let dialog = file_chooser::open::Dialog::new()
//!         .title("Choose a folder");
//!
//!     match dialog.open_folder().await {
//!         Ok(response) => println!("selected to open {:?}", response.url()),
//!
//!         Err(file_chooser::Error::Cancelled) => (),
//!
//!         Err(why) => eprintln!("error selecting folder to open: {why:?}")
//!     }
//! });
//! ```
//!
//! # Open multiple folders
//!
//! ```no_run
//! cosmic::task::future(async {
//!     use cosmic::dialog::file_chooser;
//!
//!     let dialog = file_chooser::open::Dialog::new()
//!         .title("Choose a folder");
//!
//!     match dialog.open_folders().await {
//!         Ok(response) => println!("selected to open {:?}", response.urls()),
//!
//!         Err(file_chooser::Error::Cancelled) => (),
//!
//!         Err(why) => eprintln!("error selecting folder(s) to open: {why:?}")
//!     }
//! });
//! ```

/// Open file dialog.
pub mod open;

/// Save file dialog.
pub mod save;

#[cfg(feature = "xdg-portal")]
pub use ashpd::desktop::file_chooser::{Choice, FileFilter};

use thiserror::Error;

/// A file filter, to limit the available file choices to certain extensions.
#[cfg(feature = "rfd")]
#[must_use]
pub struct FileFilter {
    description: String,
    extensions: Vec<String>,
}

#[cfg(feature = "rfd")]
impl FileFilter {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            extensions: Vec::new(),
        }
    }

    pub fn extension(mut self, extension: impl Into<String>) -> Self {
        self.extensions.push(extension.into());
        self
    }
}

/// Errors that my occur when interacting with the file chooser subscription
#[derive(Debug, Error)]
pub enum Error {
    #[error("dialog request cancelled")]
    Cancelled,
    #[error("dialog close failed")]
    Close(#[source] DialogError),
    #[error("open dialog failed")]
    Open(#[source] DialogError),
    #[error("dialog response failed")]
    Response(#[source] DialogError),
    #[error("save dialog failed")]
    Save(#[source] DialogError),
    #[error("could not set directory")]
    SetDirectory(#[source] DialogError),
    #[error("could not set absolute path for file name")]
    SetAbsolutePath(#[source] DialogError),
    #[error("path from dialog was not absolute")]
    UrlAbsolute,
}

#[cfg(feature = "xdg-portal")]
pub type DialogError = ashpd::Error;

#[cfg(feature = "rfd")]
#[derive(Debug, Error)]
#[error("no file selected")]
pub struct DialogError {}
