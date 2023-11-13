// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Dialogs for opening and save files.

pub mod open;
pub mod save;

pub use ashpd::desktop::file_chooser::{Choice, FileFilter, SelectedFiles};
use iced::futures::{channel, SinkExt, StreamExt};
use iced::{Command, Subscription};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

/// Prevents duplicate file chooser dialog requests.
static OPENED: AtomicBool = AtomicBool::new(false);

/// Whether a file chooser dialog is currently active.
fn dialog_active() -> bool {
    OPENED.load(Ordering::Relaxed)
}

/// Sets the existence of a file chooser dialog.
fn dialog_active_set(value: bool) {
    OPENED.store(value, Ordering::SeqCst);
}

/// Creates an [`open::Dialog`] if no other file chooser exists.
pub fn open_file() -> Option<open::Dialog> {
    if dialog_active() {
        None
    } else {
        Some(open::Dialog::new())
    }
}

/// Creates a [`save::Dialog`] if no other file chooser exists.
pub fn save_file() -> Option<save::Dialog> {
    if dialog_active() {
        None
    } else {
        Some(save::Dialog::new())
    }
}

/// Creates a subscription for file chooser events.
pub fn subscription<M, H>(handle: H) -> Subscription<M>
where
    M: Send + 'static,
    H: Fn(Message) -> M + Send + Sync + 'static,
{
    let type_id = std::any::TypeId::of::<Handler<M, H>>();

    iced::subscription::channel(type_id, 1, move |output| async move {
        let mut state = Handler {
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
                        dialog_active_set(false);
                    }

                    Request::Save(dialog) => {
                        state.save(dialog).await;
                        dialog_active_set(false);
                    }

                    Request::Response => state.response().await,
                }
            }
        }
    })
}

/// Errors that my occur when interacting with the file chooser subscription
#[derive(Debug, Error)]
pub enum Error {
    #[error("dialog close failed")]
    Close(#[source] ashpd::Error),
    #[error("dialog open failed")]
    Open(#[source] ashpd::Error),
    #[error("dialog response failed")]
    Response(#[source] ashpd::Error),
}

/// Requests for the file chooser subscription
enum Request {
    Close,
    Open(open::Dialog),
    Save(save::Dialog),
    Response,
}

/// Messages from the file chooser subscription.
pub enum Message {
    Closed,
    Err(Error),
    Init(Sender),
    Opened,
    Selected(SelectedFiles),
}

/// Sends requests to the file chooser subscription.
#[derive(Clone, Debug)]
pub struct Sender(channel::mpsc::Sender<Request>);

impl Sender {
    /// Creates a [`Command`] that closes a file chooser dialog.
    pub fn close(&mut self) -> Command<()> {
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Close).await;
            ()
        })
    }

    /// Creates a [`Command`] that opens the file chooser.
    pub fn open(&mut self, dialog: open::Dialog) -> Command<()> {
        dialog_active_set(true);
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Open(dialog)).await;
            ()
        })
    }

    /// Creates a [`Command`] that requests the response from a file chooser dialog.
    pub fn response(&mut self) -> Command<()> {
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Response).await;
            ()
        })
    }

    /// Creates a [`Command`] that opens a new save file dialog.
    pub fn save(&mut self, dialog: save::Dialog) -> Command<()> {
        dialog_active_set(true);
        let mut sender = self.0.clone();

        crate::command::future(async move {
            let _res = sender.send(Request::Save(dialog)).await;
            ()
        })
    }
}

struct Handler<M, Handle: Fn(Message) -> M> {
    active: Option<ashpd::desktop::Request<SelectedFiles>>,
    handle: Handle,
    output: channel::mpsc::Sender<M>,
}

impl<M, Handle: Fn(Message) -> M> Handler<M, Handle> {
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
    async fn open(&mut self, dialog: open::Dialog) {
        let response = match open::create(dialog).await {
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

    /// Creates a new dialog, and closes any prior active dialogs.
    async fn save(&mut self, dialog: save::Dialog) {
        let response = match save::create(dialog).await {
            Ok(request) => {
                self.active = Some(request);
                Message::Opened
            }
            Err(why) => Message::Err(Error::Open(why)),
        };

        self.emit(response).await;
    }
}
