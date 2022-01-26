// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

extern crate cosmic_component_system as ccs;

mod components;

use self::components::App;
use ccs::{AppRunner, Component};

fn main() {
    gtk4::builders::ApplicationBuilder::new()
        .application_id("org.pop.CosmicComponentExample")
        .cosmic_run(|app| {
            App::init(app);
        });
}
