pub use iced::{widget, Background, Color};
pub use crate::Theme;

pub mod list_view {
    #[macro_export]
    macro_rules! list_view {
        ($($x:expr),+ $(,)?) => (
            $crate::iced::widget::Column::with_children(
                vec![$($crate::iced::Element::from($x)),+]
            )
            .spacing(24)
            .padding(24)
            .max_width(600)
        );
    }

    #[macro_export]
    macro_rules! list_view_row {
        ($($x:expr),+ $(,)?) => (
            $crate::iced::widget::Row::with_children(vec![
                $($crate::iced::Element::from($x)),+
            ])
            .align_items(Alignment::Center)
            .padding([0, 8])
            .spacing(12)
        );
    }

    #[macro_export]
    macro_rules! list_view_section {
        ($title:expr, $($x:expr),+ $(,)?) => (
            $crate::iced::widget::Column::with_children(vec![
                $crate::iced::widget::Text::new($title)
                .font($crate::font::FONT_SEMIBOLD)
                .into()
                ,
                $crate::iced::widget::Container::new({
                    let mut children = vec![$($crate::iced::Element::from($x)),+];

                    //TODO: more efficient method for adding separators
                    let mut i = 1;
                    while i < children.len() {
                        children.insert(i, $crate::separator!(12).into());
                        i += 2;
                    }

                    $crate::iced::widget::Column::with_children(children)
                    .spacing(12)
                })
                .padding([12, 16])
                .style(theme::Container::Custom(
                    list_section_style
                ))
                .into()
            ])
            .spacing(8)
        );
    }

    #[macro_export]
    macro_rules! list_view_item {
        ($title:expr, $($x:expr),+ $(,)?) => (
            $crate::list_view_row!(
                $crate::iced::widget::Text::new($title),
                $crate::iced::widget::horizontal_space(
                    $crate::iced::Length::Fill
                ),
                $($x),+
            )
        );
    }

    pub fn list_section_style(theme: &Theme) -> widget::container::Appearance {
        let cosmic = &theme.cosmic().primary;
        widget::container::Appearance {
            text_color: Some(cosmic.on.into()),
            background: Some(Background::Color(cosmic.base.into())),
            border_radius: 8.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    use crate::widget::{Background, Color};
    use iced::widget;
    use crate::Theme;

    pub use list_view;
    pub use list_view_item;
    pub use list_view_row;
    pub use list_view_section;
}

pub mod list_box {
    #[macro_export]
    macro_rules! list_box_row {
        ($title:expr) => {
            $crate::iced::widget::container(
                $crate::iced::widget::row![
                    text($title).size(18),
                    $crate::iced::widget::vertical_space(Length::Fill),
                    $crate::iced::widget::horizontal_space(Length::Fill)
                ]
                .height(Length::Fill)
                .align_items($crate::iced::alignment::Alignment::Center),
            )
            .max_height(60)
            .padding(10)
        };
        ($title:expr, $subtitle:expr) => {
            $crate::iced::widget::container(
                $crate::iced::widget::row![
                    column(vec![
                        text($title).size(18).into(),
                        text($subtitle).size(16).into(),
                    ]),
                    $crate::iced::widget::vertical_space(Length::Fill),
                    $crate::iced::widget::horizontal_space(Length::Fill)
                ]
                .height(Length::Fill)
                .align_items($crate::iced::alignment::Alignment::Center),
            )
            .max_height(60)
            .padding(10)
        };
        ($title:expr, $subtitle:expr, $icon:expr) => {
            $crate::iced::widget::container(
                $crate::iced::widget::row![
                    container($crate::widget::icon($icon, 20)).padding(10),
                    column(vec![
                        text($title).size(18).into(),
                        text($subtitle).size(16).into(),
                    ]),
                    $crate::iced::widget::vertical_space(Length::Fill),
                    $crate::iced::widget::horizontal_space(Length::Fill)
                ]
                .height(Length::Fill)
                .align_items($crate::iced::alignment::Alignment::Center),
            )
            .max_height(60)
            .padding(10)
        };
    }

    pub use list_box_row;
}
