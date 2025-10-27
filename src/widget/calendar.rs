// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays an interactive calendar.

use std::cmp;

use crate::fl;
use crate::iced_core::{Alignment, Length, Padding};
use crate::widget::{Grid, button, column, grid, icon, row, text};
use apply::Apply;
use chrono::{Datelike, Days, Local, Month, Months, NaiveDate, Weekday};

/// A widget that displays an interactive calendar.
pub fn calendar<M>(
    model: &CalendarModel,
    on_select: impl Fn(NaiveDate) -> M + 'static,
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CalendarModel {
    pub selected: NaiveDate,
    pub visible: NaiveDate,
}

impl CalendarModel {
    pub fn now() -> Self {
        let now = Local::now();
        let naive_now = NaiveDate::from(now.naive_local());
        CalendarModel {
            selected: naive_now,
            visible: naive_now,
        }
    }

    #[inline]
    pub fn new(selected: NaiveDate, visible: NaiveDate) -> Self {
        CalendarModel { selected, visible }
    }

    pub fn show_prev_month(&mut self) {
        let prev_month_date = self
            .visible
            .checked_sub_months(Months::new(1))
            .expect("valid naivedate");

        self.visible = prev_month_date;
    }

    pub fn show_next_month(&mut self) {
        let next_month_date = self
            .visible
            .checked_add_months(Months::new(1))
            .expect("valid naivedate");

        self.visible = next_month_date;
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
    pub fn set_selected_visible(&mut self, selected: NaiveDate) {
        self.selected = selected;
        self.visible = self.selected;
    }
}

pub struct Calendar<'a, M> {
    model: &'a CalendarModel,
    on_select: Box<dyn Fn(NaiveDate) -> M>,
    on_prev: Box<dyn Fn() -> M>,
    on_next: Box<dyn Fn() -> M>,
    first_day_of_week: Weekday,
}

impl<'a, Message> From<Calendar<'a, Message>> for crate::Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(this: Calendar<'a, Message>) -> Self {
        macro_rules! icon {
            ($name:expr, $on_press:expr) => {{
                #[cfg(target_os = "linux")]
                let icon = { icon::from_name($name).apply(button::icon) };
                #[cfg(not(target_os = "linux"))]
                let icon = {
                    icon::from_svg_bytes(include_bytes!(concat!("../../res/icons/", $name, ".svg")))
                        .symbolic(true)
                        .apply(button::icon)
                };
                icon.padding([0, 12]).on_press($on_press)
            }};
        }
        macro_rules! translate_month {
            ($month:expr, $year:expr) => {{
                match $month {
                    chrono::Month::January => fl!("january", year = $year),
                    chrono::Month::February => fl!("february", year = $year),
                    chrono::Month::March => fl!("march", year = $year),
                    chrono::Month::April => fl!("april", year = $year),
                    chrono::Month::May => fl!("may", year = $year),
                    chrono::Month::June => fl!("june", year = $year),
                    chrono::Month::July => fl!("july", year = $year),
                    chrono::Month::August => fl!("august", year = $year),
                    chrono::Month::September => fl!("september", year = $year),
                    chrono::Month::October => fl!("october", year = $year),
                    chrono::Month::November => fl!("november", year = $year),
                    chrono::Month::December => fl!("december", year = $year),
                }
            }};
        }
        macro_rules! translate_weekday {
            ($weekday:expr) => {{
                match $weekday {
                    Weekday::Mon => fl!("monday"),
                    Weekday::Tue => fl!("tuesday"),
                    Weekday::Wed => fl!("wednesday"),
                    Weekday::Thu => fl!("thursday"),
                    Weekday::Fri => fl!("friday"),
                    Weekday::Sat => fl!("saturday"),
                    Weekday::Sun => fl!("sunday"),
                }
            }};
        }

        let date = text(translate_month!(
            Month::try_from(this.model.visible.month() as u8)
                .expect("Previously valid month is suddenly invalid"),
            this.model.visible.year()
        ))
        .size(18);

        let month_controls = row::with_capacity(2)
            .push(icon!("go-previous-symbolic", (this.on_prev)()))
            .push(icon!("go-next-symbolic", (this.on_next)()));

        // Calender
        let mut calendar_grid: Grid<'_, Message> =
            grid().padding([0, 12].into()).width(Length::Fill);

        let mut first_day_of_week = this.first_day_of_week;
        for _ in 0..7 {
            calendar_grid = calendar_grid.push(
                text(translate_weekday!(first_day_of_week))
                    .size(12)
                    .width(Length::Fixed(36.0))
                    .align_x(Alignment::Center),
            );

            first_day_of_week = first_day_of_week.succ();
        }
        calendar_grid = calendar_grid.insert_row();

        let monday = get_calender_first(
            this.model.visible.year(),
            this.model.visible.month(),
            first_day_of_week,
        );
        let mut day_iter = monday.iter_days();
        for i in 0..42 {
            if i > 0 && i % 7 == 0 {
                calendar_grid = calendar_grid.insert_row();
            }

            let date = day_iter.next().unwrap();
            let is_currently_viewed_month = date.month() == this.model.visible.month()
                && date.year_ce() == this.model.visible.year_ce();
            let is_currently_selected_month = date.month() == this.model.selected.month()
                && date.year_ce() == this.model.selected.year_ce();
            let is_currently_selected_day =
                date.day() == this.model.selected.day() && is_currently_selected_month;

            calendar_grid = calendar_grid.push(date_button(
                date,
                is_currently_viewed_month,
                is_currently_selected_day,
                &this.on_select,
            ));
        }

        let content_list = column::with_children([
            row::with_children([
                date.into(),
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

fn date_button<Message: Clone + 'static>(
    date: NaiveDate,
    is_currently_viewed_month: bool,
    is_currently_selected_day: bool,
    on_select: &dyn Fn(NaiveDate) -> Message,
) -> crate::widget::Button<'static, Message> {
    let style = if is_currently_selected_day {
        button::ButtonClass::Suggested
    } else {
        button::ButtonClass::Text
    };

    let button = button::custom(text(format!("{}", date.day())).center())
        .class(style)
        .height(Length::Fixed(36.0))
        .width(Length::Fixed(36.0));

    if is_currently_viewed_month {
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

#[inline]
fn menu_control_padding() -> Padding {
    let guard = crate::theme::THEME.lock().unwrap();
    let cosmic = guard.cosmic();
    [cosmic.space_xxs(), cosmic.space_m()].into()
}
