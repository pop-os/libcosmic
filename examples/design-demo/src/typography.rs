// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{App, Message};
use cosmic::widget::{column, divider, row, text};
use cosmic::Element;

impl App
where
    Self: cosmic::Application,
{
    pub fn view_typography(&self) -> Element<Message> {
        // TODO: Implement with grid widget once grid widget is finished.
        const WIDTH: u16 = 128;
        static SAMPLE_TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

        column()
            .spacing(32)
            .push(
                column()
                    .spacing(4)
                    .push(
                        row()
                            .push(text::heading("Text Style").width(WIDTH))
                            .push(text::heading("Font Size").width(WIDTH))
                            .push(text::heading("Line Height").width(WIDTH))
                            .push(text::heading("Weight").width(WIDTH))
                            .push(text::heading("Two-line Example").width(463)),
                    )
                    .push(divider::horizontal::default()),
            )
            .push(
                row()
                    .push(text::title1("Title 1").width(WIDTH))
                    .push(text::title1("32px").width(WIDTH))
                    .push(text::title1("44px").width(WIDTH))
                    .push(text::title1("Light (300)").width(WIDTH))
                    .push(text::title1(SAMPLE_TEXT).width(463)),
            )
            .push(
                row()
                    .push(text::title2("Title 2").width(WIDTH))
                    .push(text::title2("28px").width(WIDTH))
                    .push(text::title2("36px").width(WIDTH))
                    .push(text::title2("Regular (400)").width(WIDTH))
                    .push(text::title2(SAMPLE_TEXT).width(376)),
            )
            .push(
                row()
                    .push(text::title3("Title 3").width(WIDTH))
                    .push(text::title3("24px").width(WIDTH))
                    .push(text::title3("32px").width(WIDTH))
                    .push(text::title3("Regular (400)").width(WIDTH))
                    .push(text::title3(SAMPLE_TEXT).width(376)),
            )
            .push(
                row()
                    .push(text::title4("Title 4").width(WIDTH))
                    .push(text::title4("20px").width(WIDTH))
                    .push(text::title4("28px").width(WIDTH))
                    .push(text::title4("Regular (400)").width(WIDTH))
                    .push(text::title4(SAMPLE_TEXT).width(335)),
            )
            .push(
                row()
                    .push(text::heading("Heading").width(WIDTH))
                    .push(text::heading("14px").width(WIDTH))
                    .push(text::heading("20px").width(WIDTH))
                    .push(text::heading("Semibold (600)").width(WIDTH))
                    .push(text::heading(SAMPLE_TEXT).width(234)),
            )
            .push(
                row()
                    .push(text::caption_heading("Caption Heading").width(WIDTH))
                    .push(text::caption_heading("10px").width(WIDTH))
                    .push(text::caption_heading("14px").width(WIDTH))
                    .push(text::caption_heading("Semibold (600)").width(WIDTH))
                    .push(text::caption_heading(SAMPLE_TEXT).width(164)),
            )
            .push(
                row()
                    .push(text::body("Body").width(WIDTH))
                    .push(text::body("14px").width(WIDTH))
                    .push(text::body("20px").width(WIDTH))
                    .push(text::body("Regular (400)").width(WIDTH))
                    .push(text::body(SAMPLE_TEXT).width(234)),
            )
            .push(
                row()
                    .push(text::caption("Caption").width(WIDTH))
                    .push(text::caption("10px").width(WIDTH))
                    .push(text::caption("14px").width(WIDTH))
                    .push(text::caption("Regular (400)").width(WIDTH))
                    .push(text::caption(SAMPLE_TEXT).width(164)),
            )
            .push(
                row()
                    .push(text::monotext("Monotext").width(WIDTH))
                    .push(text::monotext("14px").width(WIDTH))
                    .push(text::monotext("20px").width(WIDTH))
                    .push(text::monotext("Regular (400)").width(WIDTH))
                    .push(text::monotext(SAMPLE_TEXT).width(280)),
            )
            .into()
    }
}
