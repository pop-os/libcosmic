use crate::{iced::Length, style, theme, widget, Element};
use std::borrow::Cow;

pub fn dialog<'a, Message>(title: impl Into<Cow<'a, str>>) -> Dialog<'a, Message> {
    Dialog::new(title)
}

pub struct Dialog<'a, Message> {
    title: Cow<'a, str>,
    icon: Option<Element<'a, Message>>,
    body: Option<Cow<'a, str>>,
    controls: Vec<Element<'a, Message>>,
    primary_action: Option<Element<'a, Message>>,
    secondary_action: Option<Element<'a, Message>>,
    tertiary_action: Option<Element<'a, Message>>,
}

impl<'a, Message> Dialog<'a, Message> {
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            body: None,
            controls: Vec::new(),
            primary_action: None,
            secondary_action: None,
            tertiary_action: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn body(mut self, body: impl Into<Cow<'a, str>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn control(mut self, control: impl Into<Element<'a, Message>>) -> Self {
        self.controls.push(control.into());
        self
    }

    pub fn primary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.primary_action = Some(button.into());
        self
    }

    pub fn secondary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.secondary_action = Some(button.into());
        self
    }

    pub fn tertiary_action(mut self, button: impl Into<Element<'a, Message>>) -> Self {
        self.tertiary_action = Some(button.into());
        self
    }
}

impl<'a, Message: Clone + 'static> From<Dialog<'a, Message>> for Element<'a, Message> {
    fn from(dialog: Dialog<'a, Message>) -> Self {
        let cosmic_theme::Spacing {
            space_l,
            space_m,
            space_s,
            space_xxs,
            ..
        } = theme::THEME.lock().unwrap().cosmic().spacing;

        let mut content_col = widget::column::with_capacity(3 + dialog.controls.len() * 2);
        content_col = content_col.push(widget::text::title3(dialog.title));
        if let Some(body) = dialog.body {
            content_col = content_col.push(widget::vertical_space(Length::Fixed(space_xxs.into())));
            content_col = content_col.push(widget::text::body(body));
        }
        for control in dialog.controls {
            content_col = content_col.push(widget::vertical_space(Length::Fixed(space_s.into())));
            content_col = content_col.push(control);
        }

        let mut content_row = widget::row::with_capacity(2).spacing(space_s);
        if let Some(icon) = dialog.icon {
            content_row = content_row.push(icon);
        }
        content_row = content_row.push(content_col);

        let mut button_row = widget::row::with_capacity(4).spacing(space_xxs);
        if let Some(button) = dialog.tertiary_action {
            button_row = button_row.push(button);
        }
        button_row = button_row.push(widget::horizontal_space(Length::Fill));
        if let Some(button) = dialog.secondary_action {
            button_row = button_row.push(button);
        }
        if let Some(button) = dialog.primary_action {
            button_row = button_row.push(button);
        }

        Element::from(
            widget::container(
                widget::column::with_children(vec![content_row.into(), button_row.into()])
                    .spacing(space_l),
            )
            .style(style::Container::Dialog)
            .padding(space_m)
            .width(Length::Fixed(570.0)),
        )
    }
}
