// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays an interactive calendar.

use std::cmp;

use crate::iced_core::{Length, Padding};
use crate::widget::{button, column, grid, icon, row, text, Grid};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use iced::alignment::{Horizontal, Vertical};

/// A widget that displays an interactive calendar.
pub fn calendar<M>(
    selected: &NaiveDate,
    on_select: impl Fn(NaiveDate) -> M + 'static,
) -> Calendar<M> {
    Calendar {
        selected,
        on_select: Box::new(on_select),
    }
}

pub fn set_day(date_selected: NaiveDate, day: u32) -> NaiveDate {
    let current = date_selected.day();

    let new_date = match current.cmp(&day) {
        cmp::Ordering::Less => date_selected.checked_add_days(Days::new((day - current) as u64)),

        cmp::Ordering::Greater => date_selected.checked_sub_days(Days::new((current - day) as u64)),

        _ => None,
    };

    if let Some(new) = new_date {
        new
    } else {
        date_selected
    }
}

pub fn set_prev_month(date_selected: NaiveDate) -> NaiveDate {
    date_selected
        .checked_sub_months(Months::new(1))
        .expect("valid naivedate")
}

pub fn set_next_month(date_selected: NaiveDate) -> NaiveDate {
    date_selected
        .checked_add_months(Months::new(1))
        .expect("valid naivedate")
}

pub struct Calendar<'a, M> {
    selected: &'a NaiveDate,
    on_select: Box<dyn Fn(NaiveDate) -> M>,
}

impl<'a, Message> From<Calendar<'a, Message>> for crate::Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(this: Calendar<'a, Message>) -> Self {
        let date = text(this.selected.format("%B %-d, %Y").to_string()).size(18);
        let day_of_week = text(this.selected.format("%A").to_string()).size(14);

        let month_controls = row::with_capacity(2)
            .push(
                button::icon(icon::from_name("go-previous-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_select)(set_prev_month(this.selected.clone()))),
            )
            .push(
                button::icon(icon::from_name("go-next-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_select)(set_next_month(this.selected.clone()))),
            );

        // Calender
        let mut calendar_grid: Grid<'_, Message> =
            grid().padding([0, 12].into()).width(Length::Fill);

        let mut first_day_of_week = Weekday::Sun; // TODO: Configurable
        for _ in 0..7 {
            calendar_grid = calendar_grid.push(
                text(first_day_of_week.to_string())
                    .size(12)
                    .width(Length::Fixed(36.0))
                    .horizontal_alignment(Horizontal::Center),
            );

            first_day_of_week = first_day_of_week.succ();
        }
        calendar_grid = calendar_grid.insert_row();

        let monday = get_calender_first(
            this.selected.year(),
            this.selected.month(),
            first_day_of_week,
        );
        let mut day_iter = monday.iter_days();
        for i in 0..42 {
            if i > 0 && i % 7 == 0 {
                calendar_grid = calendar_grid.insert_row();
            }

            let date = day_iter.next().unwrap();
            let is_month =
                date.month() == this.selected.month() && date.year_ce() == this.selected.year_ce();
            let is_day = date.day() == this.selected.day() && is_month;

            calendar_grid =
                calendar_grid.push(date_button(date, is_month, is_day, &this.on_select));
        }

        let content_list = column::with_children(vec![
            row::with_children(vec![
                column::with_children(vec![date.into(), day_of_week.into()]).into(),
                crate::widget::Space::with_width(Length::Fill).into(),
                month_controls.into(),
            ])
            .padding([12, 20])
            .into(),
            calendar_grid.into(),
            padded_control(crate::widget::divider::horizontal::default()).into(),
        ])
        .width(315)
        .padding([8, 0]);

        Self::new(content_list)
    }
}

fn date_button<Message>(
    date: NaiveDate,
    is_month: bool,
    is_day: bool,
    on_select: &dyn Fn(NaiveDate) -> Message,
) -> crate::widget::Button<'static, Message> {
    let style = if is_day {
        button::Style::Suggested
    } else {
        button::Style::Text
    };

    let button = button::custom(
        text(format!("{}", date.day()))
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .style(style)
    .height(Length::Fixed(36.0))
    .width(Length::Fixed(36.0));

    if is_month {
        button.on_press((on_select)(set_day(date, date.day())))
    } else {
        button
    }
}

/// Gets the first date that will be visible on the calender
#[must_use]
pub fn get_calender_first(year: i32, month: u32, from_weekday: Weekday) -> NaiveDate {
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let num_days = (date.weekday() as u32 + 7 - from_weekday as u32) % 7; // chrono::Weekday.num_days_from
    date.checked_sub_days(Days::new(num_days as u64)).unwrap()
}

// TODO: Refactor to use same function from applet module.
fn padded_control<'a, Message>(
    content: impl Into<crate::Element<'a, Message>>,
) -> crate::widget::container::Container<'a, Message, crate::Theme, crate::Renderer> {
    crate::widget::container(content)
        .padding(menu_control_padding())
        .width(Length::Fill)
}

fn menu_control_padding() -> Padding {
    let guard = crate::theme::THEME.lock().unwrap();
    let cosmic = guard.cosmic();
    [cosmic.space_xxs(), cosmic.space_m()].into()
}
