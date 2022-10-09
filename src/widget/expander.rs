use std::vec;

use apply::Apply;
use iced::{
    Element,
    Length,
    widget::{
        row, 
        horizontal_space, button, container, text, Column
    }, alignment::{Vertical, Horizontal}, theme 
};
use iced_native::widget::column;

#[derive(Default)]
pub struct Expander
{
    pub expanded: bool
}

#[derive(Clone, Copy, Debug)]
pub enum ExpanderMsg {
    Expand,
}

impl Expander {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render<'a, T>(&self, children: Vec<Element<'a, T>>) -> Element<'a, T> 
        where T: Clone + From<ExpanderMsg> + 'static
    {
        let title = text("Title")
            .size(18)
            .vertical_alignment(Vertical::Center)
            .horizontal_alignment(Horizontal::Center)
            .into();
        let subtitle = iced::widget::text("Subtitle")
            .size(14)
            .vertical_alignment(Vertical::Center)
            .horizontal_alignment(Horizontal::Center)
            .into();
        let header = column(
            vec![title, subtitle]
        ).into();
        let space = horizontal_space(Length::Fill).into();
        let icon = super::icon(
            if self.expanded {
                "go-down-symbolic"
            } else {
                "go-next-symbolic"
            },
            16
        )
        .apply(button)
        .on_press(T::from(ExpanderMsg::Expand))
        .width(Length::Units(25))
        .into();
    
        container(
            column(
                if self.expanded {
                    vec![
                        row(vec![header, space, icon]).into(),
                        container(
                            Column::with_children(children)
                        )
                        .style(theme::Container::Transparent)
                        .padding(5)
                        .into()
                    ]
                } else {
                    vec![row(vec![header, space, icon]).into()]
                }
            )
            .padding(5)
        )
        .style(theme::Container::Box)
        .into()
    }
}