// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use gtk4::prelude::{ApplicationExt, ApplicationExtManual};

pub trait AppRunner {
    /// Convenience method to activates and run a GTK4 application.
    fn cosmic_run<F: Fn(gtk4::Application) + 'static>(self, func: F);
}

impl AppRunner for gtk4::Application {
    fn cosmic_run<F: Fn(gtk4::Application) + 'static>(self, func: F) {
        self.connect_activate(move |app| func(app.clone()));
        self.run();
    }
}

impl AppRunner for gtk4::builders::ApplicationBuilder {
    fn cosmic_run<F: Fn(gtk4::Application) + 'static>(self, func: F) {
        self.build().cosmic_run(func);
    }
}
