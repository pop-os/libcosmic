// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Renderer;
use crate::widget::nav_bar::{nav_bar_pages_style, nav_bar_sections_style};
use crate::widget::{icon, scrollable};
use crate::iced::{Length, Background};
use crate::iced_lazy::Component;
use crate::iced::widget::{button, column, container, text};
use crate::iced_native::{row, Alignment, Element};
use crate::iced_style::button::Appearance;
use crate::{theme, Theme};
use std::collections::BTreeMap;
use derive_setters::Setters;

#[derive(Setters, Default)]
pub struct NavBar<'a, Message> {
    source: BTreeMap<NavBarItem, Vec<NavBarItem>>,
    active: bool,
    condensed: bool,
    #[setters(strip_option)]
    on_page_selected: Option<Box<dyn Fn(usize, usize) -> Message + 'a>>,
}

impl<'a, Message> NavBar<'a, Message> {
    pub fn new() -> Self {
        Self {
            source: Default::default(),
            active: false,
            condensed: false,
            on_page_selected: None,
        }
    }
}

pub fn nav_bar<'a, Message>() -> NavBar<'a, Message> {
    NavBar::new()
}

#[derive(Setters, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NavBarItem {
    #[setters(into)]
    title: String,
    #[setters(into)]
    icon: String,
}

impl NavBarItem {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn nav_bar_item() -> NavBarItem {
    NavBarItem::new()
}

#[derive(Clone)]
pub enum NavBarEvent {
    SectionSelected(usize),
    PageSelected(usize, usize),
    RevealSections,
}

#[derive(Default)]
pub struct NavBarState {
    selected_section: usize,
    selected_page: Option<usize>,
}

impl<'a, Message> Component<Message, Renderer> for NavBar<'a, Message> {
    type State = NavBarState;
    type Event = NavBarEvent;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            NavBarEvent::SectionSelected(section) => {
                state.selected_section = section;
                state.selected_page = None;
                None
            }
            NavBarEvent::PageSelected(section, page) => {
                state.selected_page = Some(page);
                self.on_page_selected
                    .as_ref()
                    .map(|on_page_selected| (on_page_selected)(section, page))
            }
            NavBarEvent::RevealSections => {
                state.selected_page = None;
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<Self::Event, Renderer> {
        if self.active {
            let mut sections: Vec<Element<Self::Event, Renderer>> = vec![];
            let mut pages: Vec<Element<Self::Event, Renderer>> = vec![];

            for (section_index, (section, section_pages)) in self.source.iter().enumerate() {
                sections.push(
                    button(
                        column(vec![
                            icon(&section.icon, 20).into(),
                            text(&section.title).size(14).into(),
                        ])
                        .width(Length::Units(100))
                        .height(Length::Units(50))
                        .align_items(Alignment::Center),
                    )
                    .style(if section_index == state.selected_section {
                        theme::Button::Primary.into()
                    } else {
                        theme::Button::Text.into()
                    })
                    .on_press(NavBarEvent::SectionSelected(section_index))
                    .into(),
                );
                if section_index == state.selected_section {
                    for (page_index, page) in section_pages.iter().enumerate() {
                        pages.push(if self.condensed {
                            button(
                                    column(vec![
                                    icon(&page.icon, 20).into(),
                                    text(&page.title).size(14).into(),
                                    ])
                                    .width(Length::Units(100))
                                    .height(Length::Units(50))
                                    .align_items(Alignment::Center),
                            )
                            .style(if let Some(selected_page) = state.selected_page {
                                if page_index == selected_page {
                                    theme::Button::Primary.into()
                                } else {
                                    theme::Button::Text.into()
                                }
                            } else {
                                theme::Button::Text.into()
                            }).on_press(NavBarEvent::PageSelected(section_index, page_index))
                            .into()
                        } else {
                            button(row![
                                icon(&page.icon, 20),
                                text(&page.title).size(16).width(Length::Fill)
                            ].spacing(10))
                            .padding(10)
                            .style(if let Some(selected_page) = state.selected_page {
                                if page_index == selected_page {
                                    theme::Button::Primary.into()
                                } else {
                                    theme::Button::Text.into()
                                }
                            } else {
                                theme::Button::Text.into()
                            })
                            .on_press(NavBarEvent::PageSelected(section_index, page_index))
                            .into()
                        });
                    }
                }
            }

            let nav_bar: Element<Self::Event, Renderer> = container(if self.condensed {
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
                        .max_width(100)
                        .align_items(Alignment::Center)
                        .width(Length::Units(100))
                        .height(Length::Shrink)))
                    .height(Length::Fill)
                    .style(theme::Container::Custom(nav_bar_pages_style)),
                ]
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

pub fn section_button_style(theme: &Theme) -> Appearance {
    let primary = &theme.cosmic().primary;
    Appearance {
        shadow_offset: Default::default(),
        background: Some(Background::Color(primary.base.into())),
        border_radius: 5.0,
        border_width: 0.0,
        border_color: Default::default(),
        text_color: Default::default(),
    }
}
