// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod app_runner;
mod component;
mod handle;
mod macros;

use gtk4::prelude::*;
use tokio::sync::mpsc;

pub use self::app_runner::AppRunner;
pub use self::component::{Component, ComponentInner};
pub use self::handle::{Handle, Registered};
pub use gtk4 as gtk;
pub use relm4_macros::view;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

/// Provides a convenience function for getting a widget out of a type.
pub trait Widget<W> {
    fn widget(&self) -> &W;
}

pub trait CosmicWidgetExt<W>: Widget<W>
where
    W: AsRef<gtk::Widget>,
{
    fn attach_size_group(&self, sg: &gtk::SizeGroup) -> &Self {
        sg.add_widget(self.widget().as_ref());
        self
    }
}

/// Convenience function for forwarding events from a receiver to different sender.
pub fn forward<I: 'static, O: 'static, F: (Fn(I) -> O) + 'static>(
    mut receiver: Receiver<I>,
    sender: Sender<O>,
    transformer: F,
) {
    spawn_local(async move {
        while let Some(event) = receiver.recv().await {
            if sender.send(transformer(event)).is_err() {
                break;
            }
        }
    })
}

/// Convenience function for spawning tasks on the local executor
pub fn spawn_local<F: std::future::Future<Output = ()> + 'static>(func: F) {
    gtk4::glib::MainContext::default().spawn_local(func);
}
