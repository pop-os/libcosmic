// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::*;

mod elmless;
mod elmlike;

pub use self::elmless::*;
pub use self::elmlike::*;

/// The pieces that make up the state of the component.
pub struct ComponentInner<Model, Widgets, Input, Output> {
    pub model: Model,
    pub widgets: Widgets,
    pub input: Sender<Input>,
    pub output: Sender<Output>,
}

/// Used to drop the component's event loop when the managed widget is destroyed.
enum InnerMessage<T> {
    Drop,
    Message(T),
}
