use cosmic::widget::{button, container, text, spin_button::vertical_spin_button};
use cosmic::{
    app::{Core, Task},
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{column, row, vertical_space},
        Alignment, Size,
    },
    Application, Element,
};

pub struct VertSpinnerApp {
    core: Core,
    hours: i32,
    mins: i16,
    secs: i8,
    time_msg: String,
}

#[derive(Debug, Clone)]
pub enum VertSpinnerMessages {
    UpdateHours(i32),
    UpdateMins(i16),
    UpdateSecs(i8),
    UpdateTimeMessage,
}

impl Application for VertSpinnerApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = VertSpinnerMessages;

    const APP_ID: &'static str = "com.system76.VertSpinnerExample";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (
            Self {
                core,
                hours: 0,
                mins: 0,
                secs: 0,
                time_msg: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            VertSpinnerMessages::UpdateHours(hours) => self.hours = hours,
            VertSpinnerMessages::UpdateMins(mins) => self.mins = mins,
            VertSpinnerMessages::UpdateSecs(secs) => self.secs = secs,
            VertSpinnerMessages::UpdateTimeMessage => {
                self.time_msg = format!(
                    "{}:{}:{}",
                    self.hours,
                    // Add a zero in front of a single digit number
                    if self.mins < 10 {
                        format!("0{}", self.mins)
                    } else {
                        self.mins.to_string()
                    },
                    // Add a zero in front of a single digit number
                    if self.secs < 10 {
                        format!("0{}", self.secs)
                    } else {
                        self.secs.to_string()
                    }
                )
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let spinner_row = row![
            vertical_spin_button(
                "Hours",
                1,
                self.hours,
                0,
                12,
                VertSpinnerMessages::UpdateHours
            ),
            vertical_spin_button(
                "Minutes",
                1,
                self.mins,
                0,
                59,
                VertSpinnerMessages::UpdateMins
            ),
            vertical_spin_button(
                "Seconds",
                1,
                self.secs,
                0,
                59,
                VertSpinnerMessages::UpdateSecs
            ),
        ]
        .align_y(Vertical::Center);

        let final_col = column![
            spinner_row,
            button::standard("Update Time").on_press(VertSpinnerMessages::UpdateTimeMessage),
            vertical_space().height(10),
            text(self.time_msg.clone()),
        ]
        .align_x(Alignment::Center);

        container(final_col)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = cosmic::app::Settings::default().size(Size::new(260., 240.));
    cosmic::app::run::<VertSpinnerApp>(settings, ())?;

    Ok(())
}
