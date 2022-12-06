// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::Color;

pub trait ElementExt {
    #[must_use]
    fn debug(self, debug: bool) -> Self;
}

impl<'a, Message: 'static> ElementExt for crate::Element<'a, Message> {
    fn debug(self, debug: bool) -> Self {
        if debug {
            self.explain(Color::WHITE)
        } else {
            self
        }
    }
}