// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An application which provides an open dialog

use apply::Apply;
use cosmic::app::{Command, Core, Settings};
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced_core::Length;
use cosmic::widget::button;
use cosmic::{executor, iced, ApplicationExt, Element};
use tokio::io::AsyncReadExt;
use url::Url;

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default()
        .size(cosmic::iced::Size::new(1024.0, 768.0));

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    CloseError,
    DialogClosed,
    DialogInit(file_chooser::Sender),
    DialogOpened,
    Error(String),
    FileRead(Url, String),
    OpenFile,
    Selected(Url),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    open_sender: Option<file_chooser::Sender>,
    file_contents: String,
    selected_file: Option<Url>,
    error_status: Option<String>,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = ();

    /// Message type specific to our [`App`].
    type Message = Message;

    const APP_ID: &'static str = "org.cosmic.OpenDialogDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = Self {
            core,
            open_sender: None,
            file_contents: String::new(),
            selected_file: None,
            error_status: None,
        };

        app.set_header_title("Open a file".into());
        let cmd = app.set_window_title(
            "COSMIC OpenDialog Demo".into(),
            cosmic::iced::window::Id::MAIN,
        );

        (app, cmd)
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        // Places a button the header to create open dialogs.
        vec![button::suggested("Open").on_press(Message::OpenFile).into()]
    }

    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        // Creates a subscription for handling open dialogs.
        file_chooser::subscription(|response| match response {
            file_chooser::Message::Closed => Message::DialogClosed,
            file_chooser::Message::Opened => Message::DialogOpened,
            file_chooser::Message::Selected(files) => match files.uris().first() {
                Some(file) => Message::Selected(file.to_owned()),
                None => Message::DialogClosed,
            },
            file_chooser::Message::Init(sender) => Message::DialogInit(sender),
            file_chooser::Message::Err(why) => {
                let mut source: &dyn std::error::Error = &why;
                let mut string = format!("open dialog subscription errored\n    cause: {source}");

                while let Some(new_source) = source.source() {
                    string.push_str(&format!("\n    cause: {new_source}"));
                    source = new_source;
                }

                Message::Error(string)
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DialogClosed => {
                eprintln!("dialog closed");
            }

            Message::DialogOpened => {
                if let Some(sender) = self.open_sender.as_mut() {
                    eprintln!("requesting selection");
                    return sender.response().map(|_| cosmic::app::Message::None);
                }
            }

            Message::FileRead(url, contents) => {
                eprintln!("read file");
                self.selected_file = Some(url);
                self.file_contents = contents;
            }

            Message::Selected(url) => {
                eprintln!("selected file");

                // Take existing file contents buffer to reuse its allocation.
                let mut contents = String::new();
                std::mem::swap(&mut contents, &mut self.file_contents);

                // Set the file's URL as the application title.
                self.set_header_title(url.to_string());

                // Reads the selected file into memory.
                return cosmic::command::future(async move {
                    // Check if its a valid local file path.
                    let path = match url.scheme() {
                        "file" => url.path(),
                        other => {
                            return Message::Error(format!("{url} has unknown scheme: {other}"));
                        }
                    };

                    // Open the file by its path.
                    let mut file = match tokio::fs::File::open(path).await {
                        Ok(file) => file,
                        Err(why) => {
                            return Message::Error(format!("failed to open {path}: {why}"));
                        }
                    };

                    // Read the file into our contents buffer.
                    contents.clear();

                    if let Err(why) = file.read_to_string(&mut contents).await {
                        return Message::Error(format!("failed to read {path}: {why}"));
                    }

                    contents.shrink_to_fit();

                    // Send this back to the application.
                    Message::FileRead(url, contents)
                })
                .map(cosmic::app::message::app);
            }

            // Creates a new open dialog.
            Message::OpenFile => {
                if let Some(sender) = self.open_sender.as_mut() {
                    if let Some(dialog) = file_chooser::open_file() {
                        eprintln!("opening new dialog");

                        return dialog
                            // Sets title of the dialog window.
                            .title("Choose a file".into())
                            // Sets the label of the accept button.
                            .accept_label("_Open".into())
                            // Exclude directories from file selection.
                            .include_directories(false)
                            // Defines whether to block the main window while requesting input.
                            .modal(false)
                            // Only accept one file as input.
                            .multiple_files(false)
                            // Accept only plain text files
                            .filter(FileFilter::new("Text files").mimetype("text/plain"))
                            // Emits the dialog to our sender
                            .create(sender)
                            // Ignores the output because it's empty.
                            .map(|_| cosmic::app::message::none());
                    }
                }
            }

            // Displays an error in the application's warning bar.
            Message::Error(why) => {
                self.error_status = Some(why);
            }

            // Closes the warning bar, if it was shown.
            Message::CloseError => {
                self.error_status = None;
            }

            // The open dialog. subscription provides this on register.
            Message::DialogInit(sender) => {
                eprintln!("dialog subscription enabled");
                self.open_sender = Some(sender);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content = Vec::new();

        if let Some(error) = self.error_status.as_deref() {
            content.push(
                cosmic::widget::warning(error)
                    .on_close(Message::CloseError)
                    .into(),
            );
            content.push(iced::widget::vertical_space(Length::Fixed(12.0)).into())
        }

        content.push(if self.selected_file.is_none() {
            center(iced::widget::text("Choose a text file"))
        } else {
            cosmic::widget::text(&self.file_contents)
                .apply(iced::widget::scrollable)
                .width(iced::Length::Fill)
                .into()
        });

        iced::widget::column(content).into()
    }
}

fn center<'a>(input: impl Into<Element<'a, Message>> + 'a) -> Element<'a, Message> {
    iced::widget::container(input.into())
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}
