// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use static_rc::StaticRc;

/// Uses [`StaticRc`] to create two halves of value with shared ownership, with no runtime reference counting required.
pub(crate) fn static_rc_halves<T>(value: T) -> (StaticRc<T, 1, 3>, StaticRc<T, 2, 3>) {
    StaticRc::split::<1, 2>(StaticRc::<T, 3, 3>::new(value))
}