// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays an interactive calendar.

use std::cmp;

use crate::iced_core::{Alignment, Length, Padding};
use crate::widget::{button, column, grid, icon, row, text, Grid};
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};

/// A widget that displays an interactive calendar.
pub fn calendar<M>(
    model: &CalendarModel,
    on_select: impl Fn(NaiveDate) -> M + 'static,
    on_prev: impl Fn() -> M + 'static,
    on_next: impl Fn() -> M + 'static,
) -> Calendar<M> {
    Calendar {
        model,
        on_select: Box::new(on_select),
        on_prev: Box::new(on_prev),
        on_next: Box::new(on_next),
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
            selected: naive_now.clone(),
            visible: naive_now,
        }
    }

    pub fn new(selected: NaiveDate, visible: NaiveDate) -> Self {
        CalendarModel { selected, visible }
    }

    pub fn show_prev_month(&mut self) {
        let prev_month_date = self
            .visible
            .clone()
            .checked_sub_months(Months::new(1))
            .expect("valid naivedate");

        self.visible = prev_month_date.clone();
    }

    pub fn show_next_month(&mut self) {
        let next_month_date = self
            .visible
            .clone()
            .checked_add_months(Months::new(1))
            .expect("valid naivedate");

        self.visible = next_month_date.clone();
    }

    pub fn set_prev_month(&mut self) {
        self.show_prev_month();
        self.selected = self.visible.clone();
    }

    pub fn set_next_month(&mut self) {
        self.show_next_month();
        self.selected = self.visible.clone();
    }

    pub fn set_selected_visible(&mut self, selected: NaiveDate) {
        self.selected = selected;
        self.visible = self.selected.clone();
    }
}

pub struct Calendar<'a, M> {
    model: &'a CalendarModel,
    on_select: Box<dyn Fn(NaiveDate) -> M>,
    on_prev: Box<dyn Fn() -> M>,
    on_next: Box<dyn Fn() -> M>,
}

impl<'a, Message> From<Calendar<'a, Message>> for crate::Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(this: Calendar<'a, Message>) -> Self {
        let date = text(this.model.visible.format("%B %Y").to_string()).size(18);

        let month_controls = row::with_capacity(2)
            .push(
                button::icon(icon::from_name("go-previous-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_prev)()),
            )
            .push(
                button::icon(icon::from_name("go-next-symbolic"))
                    .padding([0, 12])
                    .on_press((this.on_next)()),
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

        let content_list = column::with_children(vec![
            row::with_children(vec![
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

fn date_button<Message>(
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

fn menu_control_padding() -> Padding {
    let guard = crate::theme::THEME.lock().unwrap();
    let cosmic = guard.cosmic();
    [cosmic.space_xxs(), cosmic.space_m()].into()
}
