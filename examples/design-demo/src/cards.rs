// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{App, Message};
use cosmic::iced_core::Length;
use cosmic::widget::{container, cosmic_container, text};
use cosmic::Element;
use cosmic_time::{anim, chain, id, once_cell::sync::Lazy};

static CARDS: Lazy<id::Cards> = Lazy::new(id::Cards::unique);

impl App
where
    Self: cosmic::Application,
{
    pub fn update_cards(&mut self) {
        let timeline = &mut self.timeline;
        let chain = if self.cards_value {
            chain::Cards::on(CARDS.clone(), 1.)
        } else {
            chain::Cards::off(CARDS.clone(), 1.)
        };
        timeline.set_chain(chain);
        timeline.start();
    }

    pub fn view_cards(&self) -> Element<Message> {
        container(
            cosmic_container::container(anim!(
                CARDS,
                &self.timeline,
                vec![
                    text("Card 1").size(24).width(Length::Fill).into(),
                    text("Card 2").size(24).width(Length::Fill).into(),
                    text("Card 3").size(24).width(Length::Fill).into(),
                    text("Card 4").size(24).width(Length::Fill).into(),
                ],
                Message::Ignore,
                |_, e| Message::CardsToggled(e),
                "Show More",
                "Show Less",
                "Clear All",
                None,
                self.cards_value,
            ))
            .layer(cosmic::cosmic_theme::Layer::Secondary)
            .padding(16)
            .style(cosmic::theme::Container::Secondary),
        )
        .into()
    }
}
