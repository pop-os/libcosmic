// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays an interactive calendar.

use crate::fl;
use crate::iced_core::{Alignment, Length};
use crate::widget::{button, column, grid, icon, row, text};
use apply::Apply;
use iced::alignment::Vertical;
use jiff::{
    ToSpan,
    civil::{Date, Weekday},
};

/// A widget that displays an interactive calendar.
pub fn calendar<M>(
    model: &CalendarModel,
    on_select: impl Fn(Date) -> M + 'static,
    on_prev: impl Fn() -> M + 'static,
    on_next: impl Fn() -> M + 'static,
    first_day_of_week: Weekday,
) -> Calendar<'_, M> {
    Calendar {
        model,
        on_select: Box::new(on_select),
        on_prev: Box::new(on_prev),
        on_next: Box::new(on_next),
        first_day_of_week,
    }
}

pub fn set_day(date_selected: Date, day: i8) -> Date {
    date_selected
        .with()
        .day(day)
        .build()
        .unwrap_or(date_selected)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CalendarModel {
    pub selected: Date,
    pub visible: Date,
}

impl CalendarModel {
    pub fn now() -> Self {
        let now = jiff::Zoned::now().date();
        CalendarModel {
            selected: now,
            visible: now,
        }
    }

    #[inline]
    pub fn new(selected: Date, visible: Date) -> Self {
        CalendarModel { selected, visible }
    }

    pub fn show_prev_month(&mut self) {
        self.visible = self.visible.checked_sub(1.month()).expect("valid date");
    }

    pub fn show_next_month(&mut self) {
        self.visible = self.visible.checked_add(1.month()).expect("valid date");
    }

    #[inline]
    pub fn set_prev_month(&mut self) {
        self.show_prev_month();
        self.selected = self.visible;
    }

    #[inline]
    pub fn set_next_month(&mut self) {
        self.show_next_month();
        self.selected = self.visible;
    }

    #[inline]
    pub fn set_selected_visible(&mut self, selected: Date) {
        self.selected = selected;
        self.visible = self.selected;
    }
}

pub struct Calendar<'a, M> {
    model: &'a CalendarModel,
    on_select: Box<dyn Fn(Date) -> M>,
    on_prev: Box<dyn Fn() -> M>,
    on_next: Box<dyn Fn() -> M>,
    first_day_of_week: Weekday,
}

impl<'a, Message> From<Calendar<'a, Message>> for crate::Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(this: Calendar<'a, Message>) -> Self {
        macro_rules! translate_month {
            ($month:expr, $year:expr) => {{
                match $month {
                    1 => fl!("january", year = $year),
                    2 => fl!("february", year = $year),
                    3 => fl!("march", year = $year),
                    4 => fl!("april", year = $year),
                    5 => fl!("may", year = $year),
                    6 => fl!("june", year = $year),
                    7 => fl!("july", year = $year),
                    8 => fl!("august", year = $year),
                    9 => fl!("september", year = $year),
                    10 => fl!("october", year = $year),
                    11 => fl!("november", year = $year),
                    12 => fl!("december", year = $year),
                    _ => unreachable!(),
                }
            }};
        }
        macro_rules! translate_weekday {
            ($weekday:expr, short) => {{
                match $weekday {
                    Weekday::Monday => fl!("mon"),
                    Weekday::Tuesday => fl!("tue"),
                    Weekday::Wednesday => fl!("wed"),
                    Weekday::Thursday => fl!("thu"),
                    Weekday::Friday => fl!("fri"),
                    Weekday::Saturday => fl!("sat"),
                    Weekday::Sunday => fl!("sun"),
                }
            }};
            ($weekday:expr, long) => {{
                match $weekday {
                    Weekday::Monday => fl!("monday"),
                    Weekday::Tuesday => fl!("tuesday"),
                    Weekday::Wednesday => fl!("wednesday"),
                    Weekday::Thursday => fl!("thursday"),
                    Weekday::Friday => fl!("friday"),
                    Weekday::Saturday => fl!("saturday"),
                    Weekday::Sunday => fl!("sunday"),
                }
            }};
        }

        let date = text(translate_month!(
            this.model.visible.month(),
            this.model.visible.year()
        ))
        .size(18);

        let day = text::body(translate_weekday!(this.model.visible.weekday(), long));

        let month_controls = row::with_capacity(2)
            .spacing(8)
            .push(
                icon::from_name("go-previous-symbolic")
                    .apply(button::icon)
                    .on_press((this.on_prev)()),
            )
            .push(
                icon::from_name("go-next-symbolic")
                    .apply(button::icon)
                    .on_press((this.on_next)()),
            );

        // Calendar
        let mut calendar_grid = grid().padding([0, 12].into()).width(Length::Fill);

        let mut first_day_of_week = this.first_day_of_week;
        for _ in 0..7 {
            calendar_grid = calendar_grid.push(
                text::caption(translate_weekday!(first_day_of_week, short))
                    .width(Length::Fixed(44.0))
                    .align_x(Alignment::Center),
            );

            first_day_of_week = first_day_of_week.next();
        }
        calendar_grid = calendar_grid.insert_row();

        let first = get_calendar_first(
            this.model.visible.year(),
            this.model.visible.month(),
            this.first_day_of_week,
        );

        let today = jiff::Zoned::now().date();
        for i in 0..42 {
            if i > 0 && i % 7 == 0 {
                calendar_grid = calendar_grid.insert_row();
            }

            let date = first
                .checked_add(i.days())
                .expect("valid date in calendar range");
            let is_currently_viewed_month =
                date.first_of_month() == this.model.visible.first_of_month();
            let is_currently_selected_month =
                date.first_of_month() == this.model.selected.first_of_month();
            let is_currently_selected_day =
                date.day() == this.model.selected.day() && is_currently_selected_month;
            let is_today = date == today;

            calendar_grid = calendar_grid.push(date_button(
                date,
                is_currently_viewed_month,
                is_currently_selected_day,
                is_today,
                &this.on_select,
            ));
        }

        let content_list = column::with_children([
            row::with_children([
                column().push(date).push(day).into(),
                crate::widget::space::horizontal()
                    .width(Length::Fill)
                    .into(),
                month_controls.into(),
            ])
            .align_y(Vertical::Center)
            .padding([12, 20])
            .into(),
            calendar_grid.into(),
        ])
        .width(360)
        .padding([8, 0]);

        Self::new(content_list)
    }
}

fn date_button<Message: Clone + 'static>(
    date: Date,
    is_currently_viewed_month: bool,
    is_currently_selected_day: bool,
    is_today: bool,
    on_select: &dyn Fn(Date) -> Message,
) -> crate::widget::Button<'static, Message> {
    let style = if is_currently_selected_day {
        button::ButtonClass::Suggested
    } else if is_today {
        button::ButtonClass::Standard
    } else {
        button::ButtonClass::Text
    };

    let button = button::custom(text(format!("{}", date.day())).center())
        .class(style)
        .height(Length::Fixed(44.0))
        .width(Length::Fixed(44.0));

    if is_currently_viewed_month {
        button.on_press((on_select)(set_day(date, date.day())))
    } else {
        button
    }
}

/// Gets the first date that will be visible on the calendar
#[must_use]
pub fn get_calendar_first(year: i16, month: i8, from_weekday: Weekday) -> Date {
    let date = Date::new(year, month, 1).expect("valid date");
    let num_days = date.weekday().since(from_weekday);
    date.checked_sub(num_days.days()).expect("valid date")
}
