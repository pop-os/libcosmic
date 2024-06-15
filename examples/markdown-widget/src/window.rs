use cosmic::{app::Core, widget, Application, Command};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Window {
    core: Core,
    markdown_text: String,
}

impl Application for Window {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "io.github.elevenhsoft.Markdown";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, cosmic::app::Command<Self::Message>) {
        let markdown_text = include_str!("../markdown.md").to_string();

        (
            Self {
                core,
                markdown_text,
            },
            Command::none(),
        )
    }

    fn view(&self) -> cosmic::prelude::Element<Self::Message> {
        let md = widget::text::markdown(self.markdown_text.clone());

        widget::row().push(widget::scrollable(md)).into()
    }
}
