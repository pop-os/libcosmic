// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

extern crate cosmic_component_system as ccs;

mod components;

use self::components::App;
use ccs::Component;

fn main() {
    ccs::run(|app| {
        App::default().register(app);
    });
}
