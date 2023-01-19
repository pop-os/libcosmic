// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use derive_setters::Setters;
use fraction::{Bounded, Decimal};
use std::hash::Hash;
use std::ops::{Add, Sub};

/// A message emitted by the [`SpinButton`](super) widget.
#[derive(Clone, Copy, Debug, Hash)]
pub enum Message {
    Increment,
    Decrement,
}

#[derive(Setters)]
pub struct Model<T> {
    /// The current value of the spin button.
    #[setters(into)]
    pub value: T,
    /// The amount to increment the value.
    #[setters(into)]
    pub step: T,
    /// The minimum value permitted.
    #[setters(into)]
    pub min: T,
    /// The maximum value permitted.
    #[setters(into)]
    pub max: T,
}

impl<T: 'static> Model<T>
where
    T: Copy + Hash + Sub<Output = T> + Add<Output = T> + Ord,
{
    pub fn update(&mut self, message: Message) {
        self.value = match message {
            Message::Increment => {
                std::cmp::min(std::cmp::max(self.value + self.step, self.min), self.max)
            }
            Message::Decrement => {
                std::cmp::max(std::cmp::min(self.value - self.step, self.max), self.min)
            }
        }
    }
}

impl Default for Model<i8> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i8::MIN,
            max: i8::MAX,
        }
    }
}

impl Default for Model<i16> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i16::MIN,
            max: i16::MAX,
        }
    }
}

impl Default for Model<i32> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: i32::MIN,
            max: i32::MAX,
        }
    }
}

impl Default for Model<isize> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: isize::MIN,
            max: isize::MAX,
        }
    }
}

impl Default for Model<u8> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u8::MIN,
            max: u8::MAX,
        }
    }
}

impl Default for Model<u16> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u16::MIN,
            max: u16::MAX,
        }
    }
}

impl Default for Model<u32> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: u32::MIN,
            max: u32::MAX,
        }
    }
}

impl Default for Model<usize> {
    fn default() -> Self {
        Self {
            value: 0,
            step: 1,
            min: usize::MIN,
            max: usize::MAX,
        }
    }
}

impl Default for Model<Decimal> {
    fn default() -> Self {
        Self {
            value: Decimal::from(0.0),
            step: Decimal::from(0.0),
            min: Decimal::min_positive_value(),
            max: Decimal::max_value(),
        }
    }
}
