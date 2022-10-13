use std::vec;

use crate::{list_box_row, separator, widget::ListRow};
use apply::Apply;
use derive_setters::Setters;
use iced::{
    theme,
    widget::{self, button, container, horizontal_space, row, text, Column},
    Alignment, Background, Element, Length, Renderer, Theme,
};
use iced_lazy::Component;
use iced_native::widget::{column, event_container};

#[derive(Setters)]
pub struct Expander<'a, Message> {
    title: &'a str,
    #[setters(strip_option)]
    subtitle: Option<&'a str>,
    #[setters(strip_option)]
    icon: Option<String>,
    expansible: bool,
    #[setters(skip)]
    rows: Option<Vec<ListRow<'a>>>,
    #[setters(strip_option)]
    on_row_selected: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

pub fn expander<'a, Message>() -> Expander<'a, Message> {
    Expander {
        title: "",
        subtitle: None,
        icon: None,
        expansible: false,
        rows: None,
        on_row_selected: None,
    }
}

pub struct ExpanderState {
    pub expanded: bool,
}

impl Default for ExpanderState {
    fn default() -> Self {
        Self { expanded: true }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ExpanderEvent {
    Expand,
    RowSelected(usize),
}

impl<'a, Message> Expander<'a, Message> {
    pub fn rows(mut self, rows: Vec<ListRow<'a>>) -> Self {
        self.rows = Some(rows);
        self.expansible = true;
        self
    }

    pub fn push(&mut self, row: ListRow<'a>) {
        if self.rows.is_none() {
            self.rows = Some(vec![])
        }
        self.rows.as_mut().unwrap().push(row);
    }
}

impl<'a, Message: Clone + 'a> Component<Message, Renderer> for Expander<'a, Message> {
    type State = ExpanderState;

    type Event = ExpanderEvent;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            ExpanderEvent::Expand => {
                state.expanded = !state.expanded;
                None
            }
            ExpanderEvent::RowSelected(index) => self
                .on_row_selected
                .as_ref()
                .map(|on_row_selected| (on_row_selected)(index)),
        }
    }

    fn view(&self, state: &Self::State) -> Element<Self::Event> {
        let heading: Element<ExpanderEvent, Renderer> = {
            let mut captions = vec![text(&self.title).size(18).into()];
            if let Some(subtitle) = &self.subtitle {
                captions.push(text(subtitle).size(16).into());
            }
            let text = column(captions);
            let space: Element<ExpanderEvent, Renderer> = horizontal_space(Length::Fill).into();
            let toggler: Element<ExpanderEvent, Renderer> = {
                let mut icon = super::icon(
                    if state.expanded {
                        "go-down-symbolic"
                    } else {
                        "go-next-symbolic"
                    },
                    16,
                )
                .apply(button)
                .width(Length::Units(25));
                if self.expansible {
                    icon = icon.on_press(ExpanderEvent::Expand);
                }
                icon.into()
            };

            let items = if let Some(icon) = &self.icon {
                let icon = super::icon(icon.as_str(), 20)
                    .apply(event_container)
                    .padding(10);
                row![icon, text, space, toggler]
            } else {
                row![text, space, toggler]
            };

            container(items.align_items(Alignment::Center))
                .style(theme::Container::Custom(expander_heading_style))
                .padding(10)
                .into()
        };

        let rows: Vec<Element<_>> = if let Some(rows) = &self.rows {
            rows.iter()
                .enumerate()
                .map(|(index, row)| {
                    let subtitle = row.subtitle.unwrap_or_default();
                    if let Some(icon) = &row.icon {
                        list_box_row!(row.title, subtitle, icon.as_str())
                            .apply(event_container)
                            .on_press(ExpanderEvent::RowSelected(index))
                            .into()
                    } else {
                        list_box_row!(row.title, subtitle)
                            .apply(event_container)
                            .on_press(ExpanderEvent::RowSelected(index))
                            .into()
                    }
                })
                .enumerate()
                .flat_map(|(index, child)| {
                    if index != rows.len() - 1 {
                        vec![child, separator!(1).into()]
                    } else {
                        vec![child]
                    }
                })
                .collect()
        } else {
            vec![]
        };

        let rows: Element<ExpanderEvent> = Column::with_children(rows).into();

        let mut layout = vec![heading];
        if state.expanded && self.expansible {
            layout.push(rows)
        }

        column(layout)
            .apply(widget::container)
            .height(Length::Shrink)
            .style(theme::Container::Custom(expander_row_style))
            .into()
    }
}

impl<'a, Message: Clone + 'a> From<Expander<'a, Message>> for Element<'a, Message> {
    fn from(expander: Expander<'a, Message>) -> Self {
        iced_lazy::component(expander)
    }
}

pub fn expander_heading_style(theme: &Theme) -> widget::container::Appearance {
    let primary = &theme.cosmic().primary;
    let accent = &theme.cosmic().accent;
    widget::container::Appearance {
        text_color: Some(accent.base.into()),
        background: Some(Background::Color(primary.divider.into())),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: primary.on.into(),
    }
}

pub fn expander_row_style(theme: &Theme) -> widget::container::Appearance {
    let cosmic = &theme.cosmic().primary;
    widget::container::Appearance {
        text_color: Some(cosmic.on.into()),
        background: Some(Background::Color(cosmic.base.into())),
        border_radius: 8.0,
        border_width: 0.4,
        border_color: cosmic.divider.into(),
    }
}

pub fn separator_style(theme: &Theme) -> widget::rule::Appearance {
    let cosmic = &theme.cosmic().primary;
    widget::rule::Appearance {
        color: cosmic.divider.into(),
        width: 1,
        radius: 0.0,
        fill_mode: widget::rule::FillMode::Padded(10),
    }
}
