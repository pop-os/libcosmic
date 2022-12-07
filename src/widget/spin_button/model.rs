// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{SpinButton, SpinMessage};
use crate::Element;
use derive_setters::Setters;
use std::hash::Hash;
use std::ops::{Add, Sub};

#[derive(Setters)]
pub struct SpinButtonModel<T> {
    /// The current value of the spin button.
    pub value: T,
    /// The amount to increment the value.
    pub step: T,
    /// The minimum value permitted.
    pub min: T,
    /// The maximum value permitted.
    pub max: T,
}

impl<T: 'static> SpinButtonModel<T>
where
    T: Copy + Hash + ToString + Sub<Output = T> + Add<Output = T> + Ord,
{
    pub fn view(&self) -> Element<'static, SpinMessage> {
        SpinButton::new(self.value).into_element()
    }

    pub fn update(&mut self, message: SpinMessage) {
        self.value = match message {
            SpinMessage::Increment => {
                std::cmp::min(std::cmp::max(self.value + self.step, self.min), self.max)
            }
            SpinMessage::Decrement => {
                std::cmp::max(std::cmp::min(self.value - self.step, self.max), self.min)
            }
        }
    }
}

impl Default for SpinButtonModel<i8> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i8::MIN,
            max: i8::MAX,
        }
    }
}

impl Default for SpinButtonModel<i16> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i16::MIN,
            max: i16::MAX,
        }
    }
}

impl Default for SpinButtonModel<i32> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i32::MIN,
            max: i32::MAX,
        }
    }
}

impl Default for SpinButtonModel<isize> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: isize::MIN,
            max: isize::MAX,
        }
    }
}

impl Default for SpinButtonModel<u8> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u8::MIN,
            max: u8::MAX,
        }
    }
}

impl Default for SpinButtonModel<u16> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u16::MIN,
            max: u16::MAX,
        }
    }
}

impl Default for SpinButtonModel<u32> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u32::MIN,
            max: u32::MAX,
        }
    }
}

impl Default for SpinButtonModel<usize> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: usize::MIN,
            max: usize::MAX,
        }
    }
}

impl Default for SpinButtonModel<f32> {
    fn default() -> Self {
        Self {
            value: 0.0,
            step: 1.0,
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Default for SpinButtonModel<f64> {
    fn default() -> Self {
        Self {
            value: 0.0,
            step: 1.0,
            min: f64::MIN,
            max: f64::MAX,
        }
    }
}
