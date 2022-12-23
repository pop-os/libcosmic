// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::nav_bar::{nav_bar_pages_style, nav_bar_sections_style};
use crate::widget::{icon, scrollable};
use crate::{theme, Renderer, Theme};
use derive_setters::Setters;
use iced::{Background, Length};
use iced_lazy::Component;
use iced_native::widget::{button, column, container, text};
use iced_native::{row, Alignment, Element};
use iced_style::button::Appearance;
use std::collections::BTreeMap;

#[derive(Setters, Default)]
pub struct NavBar<'a, Message> {
    source: BTreeMap<NavBarSection, Vec<NavBarPage>>,
    active: bool,
    condensed: bool,
    on_page_selected: Option<Box<dyn Fn(NavBarSection, NavBarPage) -> Message + 'a>>,
}

impl<'a, Message> NavBar<'a, Message> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: BTreeMap::default(),
            active: false,
            condensed: false,
            on_page_selected: None,
        }
    }
}

#[must_use]
pub fn nav_bar<'a, Message>() -> NavBar<'a, Message> {
    NavBar::new()
}

#[derive(Setters, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NavBarSection {
    #[setters(into)]
    title: String,
    #[setters(into)]
    icon: String,
}

impl NavBarSection {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[must_use]
pub fn nav_bar_section() -> NavBarSection {
    NavBarSection::new()
}

#[derive(Default, Clone, Setters, PartialOrd, Ord, PartialEq, Eq)]
pub struct NavBarPage {
    #[setters(into)]
    title: String,
}

impl NavBarPage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: String::new(),
        }
    }
}

#[must_use]
pub fn nav_bar_page(title: &str) -> NavBarPage {
    let mut page = NavBarPage::new();
    page.title = title.to_string();
    page
}

#[derive(Clone)]
pub enum NavBarEvent {
    SectionSelected(NavBarSection),
    PageSelected(NavBarSection, NavBarPage),
}

#[derive(Default)]
pub struct NavBarState {
    selected_section: NavBarSection,
    section_active: bool,
    selected_page: Option<NavBarPage>,
    page_active: bool,
}

impl<'a, Message> Component<Message, Renderer> for NavBar<'a, Message> {
    type State = NavBarState;
    type Event = NavBarEvent;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            NavBarEvent::SectionSelected(section) => {
                if state.selected_section == section {
                    state.section_active = !state.section_active;
                } else {
                    state.selected_section = section;
                    state.section_active = true;
                }
                state.selected_page = None;
                state.page_active = false;
                None
            }
            NavBarEvent::PageSelected(section, page) => {
                if state.selected_page.is_some() && &page == state.selected_page.as_ref().unwrap() {
                    state.page_active = !state.page_active;
                } else {
                    state.selected_page = Some(page.clone());
                    state.page_active = true;
                }
                self.on_page_selected
                    .as_ref()
                    .map(|on_page_selected| (on_page_selected)(section, page))
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'a, Self::Event, Renderer> {
        if self.active {
            let mut sections: Vec<Element<'a, Self::Event, Renderer>> = vec![];
            let mut pages: Vec<Element<'a, Self::Event, Renderer>> = vec![];

            for (section, section_pages) in &self.source {
                sections.push(
                    button(
                        column(vec![
                            icon(section.icon.clone(), 20).into(),
                            text(section.title.clone()).size(14).into(),
                        ])
                        .width(Length::Units(100))
                        .height(Length::Units(50))
                        .align_items(Alignment::Center),
                    )
                    .style(
                        if *section == state.selected_section && state.section_active {
                            theme::Button::Primary
                        } else {
                            theme::Button::Text
                        },
                    )
                    .on_press(NavBarEvent::SectionSelected(section.clone()))
                    .into(),
                );
                if *section == state.selected_section {
                    for page in section_pages {
                        pages.push(
                            button(row![text(&page.title).size(16).width(Length::Fill)])
                                .padding(10)
                                .style(if let Some(selected_page) = &state.selected_page {
                                    if state.page_active && page == selected_page {
                                        theme::Button::Primary
                                    } else {
                                        theme::Button::Text
                                    }
                                } else {
                                    theme::Button::Text
                                })
                                .on_press(NavBarEvent::PageSelected(section.clone(), page.clone()))
                                .into(),
                        );
                    }
                }
            }

            let nav_bar: Element<Self::Event, Renderer> =
                container(if self.condensed && state.selected_page.is_some() {
                    row![container(scrollable(column(pages)
                            .spacing(10)
                            .padding(10)
                            .max_width(200)
                            .width(Length::Units(200))
                            .height(Length::Shrink)))
                        .height(Length::Fill)
                        .style(theme::Container::Custom(nav_bar_pages_style))]
                } else if !state.section_active || self.condensed && state.selected_page.is_none() {
                    row![scrollable(column(sections)
                        .spacing(10)
                        .padding(10)
                        .max_width(100)
                        .align_items(Alignment::Center)
                        .height(Length::Shrink))]
                } else {
                    row![
                        scrollable(column(sections)
                            .spacing(10)
                            .padding(10)
                            .max_width(100)
                            .align_items(Alignment::Center)
                            .height(Length::Shrink)),
                        container(scrollable(column(pages)
                            .spacing(10)
                            .padding(10)
                            .max_width(200)
                            .width(Length::Units(200))
                            .height(Length::Shrink)))
                        .height(Length::Fill)
                        .style(theme::Container::Custom(nav_bar_pages_style)),
                    ]
                })
                .height(Length::Fill)
                .style(theme::Container::Custom(nav_bar_sections_style))
                .into();
            nav_bar
        } else {
            row![].into()
        }
    }
}

impl<'a, Message: 'static> From<NavBar<'a, Message>>
    for Element<'a, Message, Renderer>
{
    fn from(nav_bar: NavBar<'a, Message>) -> Self {
        iced_lazy::component(nav_bar)
    }
}

#[must_use]
pub fn section_button_style(theme: &Theme) -> Appearance {
    let primary = &theme.cosmic().primary;
    Appearance {
        shadow_offset: iced::Vector::default(),
        background: Some(Background::Color(primary.base.into())),
        border_radius: 5.0,
        border_width: 0.0,
        border_color: iced::Color::default(),
        text_color: iced::Color::default(),
    }
}
