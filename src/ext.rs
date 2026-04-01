// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::Color;
use iced_core::Widget;

pub trait ElementExt {
    #[must_use]
    fn debug(self, debug: bool) -> Self;
}

impl<Message: 'static> ElementExt for crate::Element<'_, Message> {
    fn debug(self, debug: bool) -> Self {
        if debug {
            self.explain(Color::WHITE)
        } else {
            self
        }
    }
}

pub trait ColorExt {
    /// Combines color with background to create appearance of transparency.
    #[must_use]
    fn blend_alpha(self, background: Self, alpha: f32) -> Self;
}

impl ColorExt for iced::Color {
    fn blend_alpha(self, background: Self, alpha: f32) -> Self {
        Self {
            a: 1.0,
            r: (self.r - background.r).mul_add(alpha, background.r),
            g: (self.g - background.g).mul_add(alpha, background.g),
            b: (self.b - background.b).mul_add(alpha, background.b),
        }
    }
}
