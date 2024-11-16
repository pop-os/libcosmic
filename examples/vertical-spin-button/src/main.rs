use cosmic::widget::{button, container, text, spin_button::vertical_spin_button, spin_button::spin_button};
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
    spin_button_num: f32,
    time_msg: String,
    spin_btn_msg: String,
}

#[derive(Debug, Clone)]
pub enum SpinBtnMessages {
    UpdateHours(i32),
    UpdateMins(i16),
    UpdateSecs(i8),
    UpdateSpinBtnVal(f32),
    UpdateTimeMessage,
}

impl Application for VertSpinnerApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = SpinBtnMessages;

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
                spin_button_num: 0.0,
                time_msg: String::new(),
                spin_btn_msg: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            SpinBtnMessages::UpdateHours(hours) => self.hours = hours,
            SpinBtnMessages::UpdateMins(mins) => self.mins = mins,
            SpinBtnMessages::UpdateSecs(secs) => self.secs = secs,
            SpinBtnMessages::UpdateSpinBtnVal(spin_btn_val) => self.spin_button_num = spin_btn_val,
            SpinBtnMessages::UpdateTimeMessage => {
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
                );

                self.spin_btn_msg = format!("This came from the spin button: {}", self.spin_button_num);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let spinner_row = row![
            spin_button::spin_button(
                "Hours",
                1,
                self.hours,
                0,
                12,
                spin_button::Direction::Vertical,
                SpinBtnMessages::UpdateHours
            ),
            spin_button::spin_button(
                "Minutes",
                1,
                self.mins,
                0,
                59,
                spin_button::Direction::Vertical,
                SpinBtnMessages::UpdateMins
            ),
            spin_button::spin_button(
                "Seconds",
                1,
                self.secs,
                0,
                59,
                spin_button::Direction::Vertical,
                SpinBtnMessages::UpdateSecs
            ),
            spin_button::spin_button(
                "Spin Button Demo", 
                0.5, 
                self.spin_button_num, 
                0.0, 
                10.0, 
                spin_button::Direction::Horizontal, 
                SpinBtnMessages::UpdateSpinBtnVal
            )
        ]
        .align_y(Vertical::Center);

        let status_row = row![
            text(self.time_msg.clone()),
            cosmic::widget::horizontal_space().width(10),
            text(self.spin_btn_msg.clone())
        ];

        let final_col = column![
            spinner_row,
            button::standard("Update Time").on_press(SpinBtnMessages::UpdateTimeMessage),
            vertical_space().height(10),
            status_row,
        ]
        .align_x(Alignment::Center);

        container(final_col)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = cosmic::app::Settings::default().size(Size::new(550., 300.));
    cosmic::app::run::<VertSpinnerApp>(settings, ())?;

    Ok(())
}
