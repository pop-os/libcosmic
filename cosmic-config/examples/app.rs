use cosmic_config::setting::{App, Setting, AppConfig};

struct ExampleApp;

impl App for ExampleApp {
    const ID: &'static str = "com.Example.App";
    const VERSION: u64 = 1;
}

struct DoFoo;

impl Setting<ExampleApp> for DoFoo {
    const NAME: &'static str = "do-foo";
    type Type = bool;
}

struct WhatBar;

impl Setting<ExampleApp> for WhatBar {
    const NAME: &'static str = "what-bar";
    type Type = String;
}

fn main() {
    let config = AppConfig::<ExampleApp>::new().unwrap();
    config.set::<DoFoo>(true).unwrap();
}
