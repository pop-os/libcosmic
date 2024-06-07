use cosmic::app::{Application, Command, Core, Settings};
use cosmic::iced::executor;
use cosmic::iced_core::Size;
use cosmic::widget::{column, text_input};
use cosmic::Element;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Message {
    InputName(String),
    InputEmail(String),
}

struct Window {
    core: Core,
    name: String,
    email: String,
}

impl Application for Window {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.system76.InputValidation";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                core,
                name: String::new(),
                email: String::new(),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InputName(name) => self.name = name,
            Message::InputEmail(email) => self.email = email,
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let emailregex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
        let validation_condition = emailregex.is_match(&self.email);

        let name_input = text_input("Name", &self.name).on_input(Message::InputName);
        let email_input = text_input("E-mail", &self.email)
            .validation_condition(validation_condition)
            .on_input(Message::InputEmail);

        column()
            .push(name_input)
            .push(email_input)
            .spacing(10)
            .into()
    }
}

fn main() -> std::io::Result<()> {
    let mut settings = Settings::default();
    settings = settings.size(Size {
        width: 480.,
        height: 480.,
    });

    let _ = cosmic::app::run::<Window>(settings, ());

    Ok(())
}
