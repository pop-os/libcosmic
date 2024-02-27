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
    primary_action: Option<(Cow<'a, str>, Message, bool)>,
    secondary_action: Option<(Cow<'a, str>, Message)>,
    tertiary_action: Option<(Cow<'a, str>, Message)>,
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

    pub fn primary_action(mut self, name: impl Into<Cow<'a, str>>, message: Message) -> Self {
        self.primary_action = Some((name.into(), message, false));
        self
    }

    pub fn primary_action_destructive(
        mut self,
        name: impl Into<Cow<'a, str>>,
        message: Message,
    ) -> Self {
        self.primary_action = Some((name.into(), message, true));
        self
    }

    pub fn secondary_action(mut self, name: impl Into<Cow<'a, str>>, message: Message) -> Self {
        self.secondary_action = Some((name.into(), message));
        self
    }

    pub fn tertiary_action(mut self, name: impl Into<Cow<'a, str>>, message: Message) -> Self {
        self.tertiary_action = Some((name.into(), message));
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
        } = theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();
            theme.spacing
        });

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
        if let Some((name, message)) = dialog.tertiary_action {
            button_row = button_row.push(widget::button::text(name).on_press(message));
        }
        button_row = button_row.push(widget::horizontal_space(Length::Fill));
        if let Some((name, message)) = dialog.secondary_action {
            button_row = button_row.push(widget::button::standard(name).on_press(message));
        }
        if let Some((name, message, destructive)) = dialog.primary_action {
            if destructive {
                button_row = button_row.push(widget::button::destructive(name).on_press(message));
            } else {
                button_row = button_row.push(widget::button::suggested(name).on_press(message));
            }
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
