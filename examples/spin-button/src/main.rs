use cosmic::iced::Length;
use cosmic::widget::{column, container, spin_button};
use cosmic::Apply;
use cosmic::{
    app::{Core, Task},
    iced::{
        self,
        alignment::{Horizontal, Vertical},
        Alignment, Size,
    },
    Application, Element,
};
use fraction::Decimal;

pub struct SpinButtonExamplApp {
    core: Core,
    i8_num: i8,
    i8_str: String,
    i16_num: i16,
    i16_str: String,
    i32_num: i32,
    i32_str: String,
    i64_num: i64,
    i64_str: String,
    i128_num: i128,
    i128_str: String,
    f32_num: f32,
    f32_str: String,
    f64_num: f64,
    f64_str: String,
    dec_num: Decimal,
    dec_str: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateI8(i8),
    UpdateI16(i16),
    UpdateI32(i32),
    UpdateI64(i64),
    UpdateI128(i128),
    UpdateF32(f32),
    UpdateF64(f64),
    UpdateDec(Decimal),
}

impl Application for SpinButtonExamplApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

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
                i8_str: 0.to_string(),
                i16_num: 0,
                i16_str: 0.to_string(),
                i32_num: 0,
                i32_str: 0.to_string(),
                i64_num: 15,
                i64_str: 15.to_string(),
                i128_num: 0,
                i128_str: 0.to_string(),
                f32_num: 0.,
                f32_str: format!("{:.02}", 0.0),
                f64_num: 0.,
                f64_str: format!("{:.02}", 0.0),
                dec_num: Decimal::from(0.0),
                dec_str: format!("{:.02}", 0.0),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::UpdateI8(value) => {
                self.i8_num = value;
                self.i8_str = value.to_string();
            }

            Message::UpdateI16(value) => {
                self.i16_num = value;
                self.i16_str = value.to_string();
            }

            Message::UpdateI32(value) => {
                self.i32_num = value;
                self.i32_str = value.to_string();
            }

            Message::UpdateI64(value) => {
                self.i64_num = value;
                self.i64_str = value.to_string();
            }

            Message::UpdateI128(value) => {
                self.i128_num = value;
                self.i128_str = value.to_string();
            }

            Message::UpdateF32(value) => {
                self.f32_num = value;
                self.f32_str = format!("{value:.02}");
            }

            Message::UpdateF64(value) => {
                self.f64_num = value;
                self.f64_str = format!("{value:.02}");
            }

            Message::UpdateDec(value) => {
                self.dec_num = value;
                self.dec_str = format!("{value:.02}");
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let space_xs = cosmic::theme::spacing().space_xs;

        let vert_spinner_row = iced::widget::row![
            spin_button::vertical(&self.i8_str, self.i8_num, 1, -5, 5, Message::UpdateI8),
            spin_button::vertical(&self.i16_str, self.i16_num, 1, 0, 10, Message::UpdateI16),
            spin_button::vertical(&self.i32_str, self.i32_num, 1, 0, 12, Message::UpdateI32),
            spin_button::vertical(&self.i64_str, self.i64_num, 10, 15, 35, Message::UpdateI64),
        ]
        .spacing(space_xs)
        .align_y(Vertical::Center);

        let horiz_spinner_row = iced::widget::column![
            spin_button(
                &self.i128_str,
                self.i128_num,
                100,
                -1000,
                500,
                Message::UpdateI128
            ),
            spin_button(
                &self.f32_str,
                self.f32_num,
                1.3,
                -35.3,
                12.3,
                Message::UpdateF32
            ),
            spin_button(
                &self.f64_str,
                self.f64_num,
                1.3,
                0.0,
                3.0,
                Message::UpdateF64
            ),
            spin_button(
                &self.dec_str,
                self.dec_num,
                Decimal::from(0.25),
                Decimal::from(-5.0),
                Decimal::from(5.0),
                Message::UpdateDec
            ),
        ]
        .spacing(space_xs)
        .align_x(Alignment::Center);

        column::with_capacity(3)
            .push(vert_spinner_row)
            .push(horiz_spinner_row)
            .spacing(space_xs)
            .align_x(Alignment::Center)
            .apply(container)
            .width(Length::Fill)
            .height(Length::Fill)
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
