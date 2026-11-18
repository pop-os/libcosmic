use crate::widget::{self, container};
use crate::{Apply, Core, Element, Renderer};
use iced::widget::scrollable::{Scrollable, Viewport};
use iced::{Length, Padding, Pixels};

pub struct ScrollableView<'a, Message> {
    header: Option<Element<'a, Message>>,
    scrollable: Scrollable<'a, Message, crate::Theme, Renderer>,
    footer: Option<Element<'a, Message>>,
    header_padding: Padding,
    footer_padding: Padding,
}

impl<'a, Message: 'a> ScrollableView<'a, Message> {
    pub fn new(core: &Core, content: impl Into<Element<'a, Message>>) -> Self {
        let cosmic_theme::Spacing {
            space_xxs, space_l, ..
        } = crate::theme::spacing();
        let hp = core.horizontal_padding();

        Self {
            header: None,
            scrollable: container(content)
                .center_x(Length::Fill)
                .padding([0, hp, space_l, hp])
                .apply(widget::scrollable)
                .height(Length::Fill),
            footer: None,
            header_padding: Padding::from([0, hp, space_xxs, hp]),
            footer_padding: Padding::from([space_xxs, hp, core.border_padding(), hp]),
        }
    }

    /// Content placed above the scrollable
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Content placed below the scrollable
    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Spacing between header and scrollable
    pub fn header_spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.header_padding.bottom = amount.into().0;
        self
    }

    /// Spacing between footer and scrollable
    pub fn footer_spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.footer_padding.top = amount.into().0;
        self
    }

    /// Sets the [`widget::Id`] of the [`Scrollable`].
    pub fn id(mut self, id: widget::Id) -> Self {
        self.scrollable = self.scrollable.id(id);
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the [`Viewport`] of the [`Scrollable`]
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.scrollable = self.scrollable.on_scroll(f);
        self
    }
}

impl<'a, Message: 'a> From<ScrollableView<'a, Message>> for Element<'a, Message> {
    fn from(s: ScrollableView<'a, Message>) -> Self {
        widget::column::with_capacity(3)
            .push_maybe(s.header.map(|h| container(h).padding(s.header_padding)))
            .push(s.scrollable)
            .push_maybe(s.footer.map(|f| container(f).padding(s.footer_padding)))
            .into()
    }
}
