use cosmic::widget::{button, container, text, spin_button, vertical_spin_button, /*spin_button::Orientation*/};
use cosmic::{
    app::{Core, Task},
    iced::{
        alignment::{Horizontal, Vertical},
        widget::{column, row, vertical_space},
        Alignment, Size,
    },
    Application, Element,
};

pub struct SpinButtonExamplApp {
    core: Core,
    i8_num: i8,
    i16_num: i16,
    i32_num: i32,
    i64_num: i64,
    i128_num: i128,
    f32_num: f32,
    f64_num: f64,
    spinner_msg: String,
}

#[derive(Debug, Clone)]
pub enum SpinBtnMessages {
    UpdateI8Num(i8),
    UpdateI16Num(i16),
    UpdateI32Num(i32),
    UpdateI64Num(i64),
    UpdateI128Num(i128),
    UpdateF32Num(f32),
    UpdateF64Num(f64),
    UpdateSpinnerMsg,
}

impl Application for SpinButtonExamplApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = SpinBtnMessages;

    const APP_ID: &'static str = "com.system76.SpinButtonExample";

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
                i8_num: 0,
                i16_num: 0,
                i32_num: 0,
                i64_num: 0,
                i128_num: 0,
                f32_num: 0.,
                f64_num: 0.,
                spinner_msg: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            SpinBtnMessages::UpdateI8Num(new_i8) => self.i8_num = new_i8,
            SpinBtnMessages::UpdateI16Num(new_i16) => self.i16_num = new_i16,
            SpinBtnMessages::UpdateI32Num(new_i32) => self.i32_num = new_i32,
            SpinBtnMessages::UpdateI64Num(new_i64) => self.i64_num = new_i64,
            SpinBtnMessages::UpdateI128Num(new_i128) => self.i128_num = new_i128,
            SpinBtnMessages::UpdateF32Num(new_f32) => self.f32_num = new_f32,
            SpinBtnMessages::UpdateF64Num(new_f64) => self.f64_num = new_f64,
            SpinBtnMessages::UpdateSpinnerMsg => {
                self.spinner_msg = format!("i8: {}, i16: {}, i32: {}, i64: {}, i128: {}\nf32: {}, f64: {}", 
                    self.i8_num, self.i16_num, self.i32_num, self.i64_num, self.i128_num, self.f32_num, self.f64_num);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let vert_spinner_row = row![
            vertical_spin_button(
                "i8", // label: displayed above the widget no matter the orientation
                1, // step: how much to increment/decrement by
                self.i8_num, // current value, this is also what's displayed in the center of the widget
                -5, // minimum value, if decremented below this the widget's current value rolls to the max value
                5, // maximum value, if incremented above this the widget's current value rolls to the min value
                SpinBtnMessages::UpdateI8Num // message to send to the application's update function
            ),
            vertical_spin_button(
                "i16",
                1,
                self.i16_num,
                0,
                10,
                SpinBtnMessages::UpdateI16Num
            ),
            vertical_spin_button(
                "i32",
                1,
                self.i32_num,
                0,
                12,
                SpinBtnMessages::UpdateI32Num
            ),
            vertical_spin_button(
                "i64",
                10,
                self.i64_num,
                15,
                35,
                SpinBtnMessages::UpdateI64Num
            ),
        ]
        .align_y(Vertical::Center);

        let horiz_spinner_row = column![
            row![
                // This function can be called instead if a Horizontal Spin Button is needed.
                spin_button(
                    "i128",
                    100,
                    self.i128_num,
                    -1000,
                    500,
                    SpinBtnMessages::UpdateI128Num
                ),
            ],
            vertical_space().height(5),
            row![
                spin_button(
                    "f32", 
                    1.3,
                    self.f32_num,
                    -35.3,
                    12.3,
                    SpinBtnMessages::UpdateF32Num
                )
            ],
            vertical_space().height(5),
            row![
                spin_button(
                    "f64",
                    1.3,
                    self.f64_num,
                    0.0,
                    3.0,
                    SpinBtnMessages::UpdateF64Num
                )
            ],
        ]
        .align_x(Alignment::Center);

        let status_row = row![
            text(self.spinner_msg.clone()),
        ];

        let final_col = column![
            vert_spinner_row,
            vertical_space().height(5),
            horiz_spinner_row,
            button::standard("Show Spinner Values Passed").on_press(SpinBtnMessages::UpdateSpinnerMsg),
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
    let settings = cosmic::app::Settings::default().size(Size::new(550., 1024.));
    cosmic::app::run::<SpinButtonExamplApp>(settings, ())?;

    Ok(())
}
