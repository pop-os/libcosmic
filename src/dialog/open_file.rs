// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Request to open files and/or directories.
//!
//! Check out the [open-dialog](https://github.com/pop-os/libcosmic/tree/master/examples/open-dialog)
//! example in our repository.

use derive_setters::Setters;
use iced::futures::{channel, SinkExt, StreamExt};
use iced::{Command, Subscription};
use std::cell::Cell;
use thiserror::Error;

thread_local! {
    /// Prevents duplicate dialog open requests.
    static OPENED: Cell<bool> = Cell::new(false);
}

fn dialog_is_open() -> bool {
    OPENED.with(Cell::get)
}

/// Creates a [`Builder`] if no other open file dialog exists.
pub fn builder() -> Option<Builder> {
    if dialog_is_open() {
        None
    } else {
        Some(Builder::new())
    }
}

/// Creates a subscription for open file dialog events.
pub fn subscription<M: Send + 'static>(handle: fn(Message) -> M) -> Subscription<M> {
    let type_id = std::any::TypeId::of::<State<M>>();

    iced::subscription::channel(type_id, 1, move |output| async move {
        let mut state = State {
            active: None,
            handle,
            output,
        };

        loop {
            let (sender, mut receiver) = channel::mpsc::channel(1);

            state.emit(Message::Init(Sender(sender))).await;

            while let Some(request) = receiver.next().await {
                match request {
                    Request::Close => state.close().await,

                    Request::Open(dialog) => {
                        state.open(dialog).await;
                        OPENED.with(|last| last.set(false));
                    }

                    Request::Response => state.response().await,
                }
            }
        }
    })
}

/// Errors that my occur when interacting with an open file dialog subscription
#[derive(Debug, Error)]
pub enum Error {
    #[error("dialog close failed")]
    Close(#[source] ashpd::Error),
    #[error("dialog open failed")]
    Open(#[source] ashpd::Error),
    #[error("dialog response failed")]
    Response(#[source] ashpd::Error),
}

/// Requests for an open file dialog subscription
enum Request {
    Close,
    Open(Builder),
    Response,
}

/// Messages from an open file dialog subscription.
pub enum Message {
    Closed,
    Err(Error),
    Init(Sender),
    Opened,
    Selected(super::SelectedFiles),
}

/// Sends requests to an open file dialog subscription.
#[derive(Clone, Debug)]
pub struct Sender(channel::mpsc::Sender<Request>);

impl Sender {
    /// Creates a [`Command`] that closes an active open file dialog.
    pub fn close(&mut self) -> Command<()> {
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Close).await;
            ()
        })
    }

    /// Creates a [`Command`] that opens a new open file dialog.
    pub fn open(&mut self, dialog: Builder) -> Command<()> {
        OPENED.with(|opened| opened.set(true));

        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Open(dialog)).await;
            ()
        })
    }

    /// Creates a [`Command`] that requests the response from an active open file dialog.
    pub fn response(&mut self) -> Command<()> {
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Response).await;
            ()
        })
    }
}

/// A builder for an open file dialog, passed as a request by a [`Sender`]
#[derive(Setters)]
#[must_use]
pub struct Builder {
    /// The lab for the dialog's window title.
    title: String,

    /// The label for the accept button. Mnemonic underlines are allowed.
    #[setters(strip_option)]
    accept_label: Option<String>,

    /// Whether to select for folders instead of files. Default is to select files.
    include_directories: bool,

    /// Modal dialogs require user input before continuing the program.
    modal: bool,

    /// Whether to allow selection of multiple files. Default is no.
    multiple_files: bool,

    /// Adds a list of choices.
    choices: Vec<super::Choice>,

    /// Specifies the default file filter.
    #[setters(into)]
    current_filter: Option<super::FileFilter>,

    /// A collection of file filters.
    filters: Vec<super::FileFilter>,
}

impl Builder {
    const fn new() -> Self {
        Self {
            title: String::new(),
            accept_label: None,
            include_directories: false,
            modal: true,
            multiple_files: false,
            current_filter: None,
            choices: Vec::new(),
            filters: Vec::new(),
        }
    }

    /// Creates a [`Command`] which opens the dialog.
    pub fn create(self, sender: &mut Sender) -> Command<()> {
        sender.open(self)
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

struct State<M> {
    active: Option<ashpd::desktop::Request<super::SelectedFiles>>,
    handle: fn(Message) -> M,
    output: channel::mpsc::Sender<M>,
}

impl<M> State<M> {
    /// Emits close request if there is an active dialog request.
    async fn close(&mut self) {
        if let Some(request) = self.active.take() {
            if let Err(why) = request.close().await {
                self.emit(Message::Err(Error::Close(why))).await;
            }
        }
    }

    async fn emit(&mut self, response: Message) {
        let _res = self.output.send((self.handle)(response)).await;
    }

    /// Creates a new dialog, and closes any prior active dialogs.
    async fn open(&mut self, dialog: Builder) {
        let response = match create(dialog).await {
            Ok(request) => {
                self.active = Some(request);
                Message::Opened
            }
            Err(why) => Message::Err(Error::Open(why)),
        };

        self.emit(response).await;
    }

    /// Collects selected files from the active dialog.
    async fn response(&mut self) {
        if let Some(request) = self.active.as_ref() {
            let response = match request.response() {
                Ok(selected) => Message::Selected(selected),
                Err(why) => Message::Err(Error::Response(why)),
            };

            self.emit(response).await;
        }
    }
}

/// Creates a new file dialog, and begins to await its responses.
async fn create(dialog: Builder) -> ashpd::Result<ashpd::desktop::Request<super::SelectedFiles>> {
    ashpd::desktop::file_chooser::OpenFileRequest::default()
        .title(Some(dialog.title.as_str()))
        .accept_label(dialog.accept_label.as_deref())
        .directory(dialog.include_directories)
        .modal(dialog.modal)
        .multiple(dialog.multiple_files)
        .choices(dialog.choices)
        .filters(dialog.filters)
        .current_filter(dialog.current_filter)
        .send()
        .await
}
