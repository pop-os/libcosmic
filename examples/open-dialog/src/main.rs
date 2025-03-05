// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An application which provides an open dialog

use apply::Apply;
use cosmic::app::{Core, Settings, Task};
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced_core::Length;
use cosmic::widget::button;
use cosmic::{executor, iced, ApplicationExt, Element};
use std::sync::Arc;
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
    Cancelled,
    CloseError,
    Error(String),
    FileRead(Url, String),
    OpenError(Arc<file_chooser::Error>),
    OpenFile,
    Selected(Url),
    Surface(cosmic::surface::Action),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
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

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Task<Self::Message>) {
        let id = core.main_window_id().unwrap();
        let mut app = App {
            core,
            file_contents: String::new(),
            selected_file: None,
            error_status: None,
        };

        app.set_header_title("Open a file".into());
        let cmd = app.set_window_title("COSMIC OpenDialog Demo".into(), id);

        (app, cmd)
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        // Places a button the header to create open dialogs.
        vec![button::suggested("Open").on_press(Message::OpenFile).into()]
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Cancelled => {
                eprintln!("open file dialog cancelled");
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
                return cosmic::task::future(async move {
                    // Check if its a valid local file path.
                    let path = match url.scheme() {
                        "file" => url.to_file_path().unwrap(),
                        other => {
                            return Message::Error(format!("{url} has unknown scheme: {other}"));
                        }
                    };

                    // Open the file by its path.
                    let mut file = match tokio::fs::File::open(&path).await {
                        Ok(file) => file,
                        Err(why) => {
                            return Message::Error(format!(
                                "failed to open {}: {why}",
                                path.display()
                            ));
                        }
                    };

                    // Read the file into our contents buffer.
                    contents.clear();

                    if let Err(why) = file.read_to_string(&mut contents).await {
                        return Message::Error(format!("failed to read {}: {why}", path.display()));
                    }

                    contents.shrink_to_fit();

                    // Send this back to the application.
                    Message::FileRead(url, contents)
                });
            }
            Message::OpenFile => {
                return cosmic::task::future(async move {
                    eprintln!("opening new dialog");

                    #[cfg(feature = "rfd")]
                    let filter = FileFilter::new("Text files").extension("txt");

                    #[cfg(feature = "xdg-portal")]
                    let filter = FileFilter::new("Text files").glob("*.txt");

                    let dialog = file_chooser::open::Dialog::new()
                        // Sets title of the dialog window.
                        .title("Choose a file")
                        // Accept only plain text files
                        .filter(filter);

                    match dialog.open_file().await {
                        Ok(response) => Message::Selected(response.url().to_owned()),

                        Err(file_chooser::Error::Cancelled) => Message::Cancelled,

                        Err(why) => Message::OpenError(Arc::new(why)),
                    }
                });
            }
            Message::Error(why) => {
                self.error_status = Some(why);
            }
            Message::OpenError(why) => {
                if let Some(why) = Arc::into_inner(why) {
                    let mut source: &dyn std::error::Error = &why;
                    let mut string =
                        format!("open dialog subscription errored\n    cause: {source}");

                    while let Some(new_source) = source.source() {
                        string.push_str(&format!("\n    cause: {new_source}"));
                        source = new_source;
                    }

                    self.error_status = Some(string);
                }
            }
            Message::CloseError => {
                self.error_status = None;
            }
            Message::Surface(surface) => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let mut content = Vec::new();

        if let Some(error) = self.error_status.as_deref() {
            content.push(
                cosmic::widget::warning(error)
                    .on_close(Message::CloseError)
                    .into(),
            );

            content.push(
                iced::widget::vertical_space()
                    .height(Length::Fixed(12.0))
                    .into(),
            );
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
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .into()
}
