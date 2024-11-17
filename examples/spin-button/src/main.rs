use cosmic::widget::divider::horizontal;
use cosmic::widget::{button, container, text, spin_button, spin_button::Direction};
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
    i8_num: i8,
    i16_num: i16,
    i32_num: i32,
    i64_num: i64,
    i128_num: i128,
    f16_num: f16,
    f32_num: f32,
    f64_num: f64,
    f128_num: f128,
    spinner_msg: String,
}

#[derive(Debug, Clone)]
pub enum SpinBtnMessages {
    UpdateI8Num(i8),
    UpdateI16Num(i16),
    UpdateI32Num(i32),
    UpdateI64Num(i64),
    UpdateI128Num(i128),
    UpdateF16Num(f16),
    UpdateF32Num(f32),
    UpdateF64Num(f64),
    UpdateF128Num(f128),
    UpdateSpinnerMsg,
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
                i8_num: 0,
                i16_num: 0,
                i32_num: 0,
                i64_num: 0,
                i128_num: 0,
                f16_num: 0.,
                f32_num: 0.,
                f64_num: 0.,
                f128_num: 0.,
                spinner_msg: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            SpinBtnMessages::UpdateI8Num(new_i8) => {
                self.i8_num = new_i8;
                SpinBtnMessages::UpdateSpinnerMsg
            },
            SpinBtnMessages::UpdateI16Num(new_i16) => self.i16_num = new_i16,
            SpinBtnMessages::UpdateI32Num(new_i32) => self.i32_num = new_i32,
            SpinBtnMessages::UpdateI64Num(new_i64) => self.i64_num = new_i64,
            SpinBtnMessages::UpdateI128Num(new_i128) => self.i128_num = new_i128,
            SpinBtnMessages::UpdateF16Num(new_f16) => self.f16_num = new_f16,
            SpinBtnMessages::UpdateF32Num(new_f32) => self.f32_num = new_f32,
            SpinBtnMessages::UpdateF64Num(new_f64) => self.f64_num = new_f64,
            SpinBtnMessages::UpdateF128Num(new_f128) => self.f128_num = new_f128,
            SpinBtnMessages::UpdateSpinnerMsg => {
                self.spinner_msg = format!("i8: {}, i16: {}, i32: {}, i64: {}, i128: {}\nf16: {}, f32: {}, f64: {}, f128: {}");
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let vert_spinner_row = row![
            spin_button(
                "i8",
                1,
                self.i8_num,
                -5,
                5,
                Direction::Vertical,
                SpinBtnMessages::UpdateI8Num
            ),
            spin_button(
                "i16",
                1,
                self.i16_num,
                0,
                10,
                Direction::Vertical,
                SpinBtnMessages::UpdateI16Num
            ),
            spin_button(
                "i32",
                1,
                self.i32_num,
                0,
                12,
                Direction::Vertical,
                SpinBtnMessages::UpdateI32Num
            ),
            spin_button(
                "i64",
                15,
                self.i64_num,
                15,
                35,
                Direction::Vertical,
                SpinBtnMessages::UpdateI64Num
            ),
        ]
        .align_y(Vertical::Center);

        let horiz_spinner_row = column![
            row![
                spin_button(
                    "i128",
                    100,
                    self.i128_num,
                    -1000,
                    i128::MAX,
                    Direction::Horizontal,
                    SpinBtnMessages::UpdateI128Num
                )
            ].into(),
            row![
                spin_button(
                    "f16",
                    0.2,
                    self.f16_num,
                    -2.2,
                    4.8,
                    Direction::Horizontal,
                    SpinBtnMessages::UpdateF16Num
                )
            ].into(),
            row![
                spin_button(
                    "f32", 
                    1.5,
                    self.f32_num,
                    -35.3,
                    12.3,
                    Direction::Horizontal,
                    SpinBtnMessages::UpdateF32Num
                )
            ].into(),
            row![
                spin_button(
                    "f64",
                    1.0,
                    self.f64_num,
                    0.0,
                    3.0,
                    Direction::Horizontal,
                    SpinBtnMessages::UpdateF64Num
                )
            ].into(),
        ]
        .into()
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
    cosmic::app::run::<VertSpinnerApp>(settings, ())?;

    Ok(())
}
