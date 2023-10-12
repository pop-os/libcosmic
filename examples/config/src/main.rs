// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic_config::{Config, ConfigGet, ConfigSet};

fn test_config(config: Config) {
    let watcher = config
        .watch(|config, keys| {
            println!("Changed: {:?}", keys);
            for key in keys.iter() {
                println!(" - {} = {:?}", key, config.get::<ron::Value>(key));
            }
        })
        .unwrap();

    println!("Setting example-bool to true");
    println!(
        "Set example-bool to true: {:?}",
        config.set("example-bool", true)
    );
    println!(
        "Get example-bool as bool: {:?}",
        config.get::<bool>("example-bool")
    );
    println!(
        "Get example-bool as u32: {:?}",
        config.get::<u32>("example-bool")
    );
    println!(
        "Get example-bool as String: {:?}",
        config.get::<String>("example-bool")
    );
    println!();

    println!("Setting example-int to 1");
    println!("Set example-int to 1: {:?}", config.set("example-int", 1));
    println!(
        "Get example-int as u32: {:?}",
        config.get::<u32>("example-int")
    );
    println!(
        "Get example-int as bool: {:?}",
        config.get::<bool>("example-int")
    );
    println!(
        "Get example-int as String: {:?}",
        config.get::<String>("example-int")
    );
    println!();

    println!("Setting example-string to \"example\"");
    println!(
        "Set example-string to \"example\": {:?}",
        config.set("example-string", "example")
    );
    println!(
        "Get example-string as String: {:?}",
        config.get::<String>("example-string")
    );
    println!(
        "Get example-string as bool: {:?}",
        config.get::<bool>("example-string")
    );
    println!(
        "Get example-string as u32: {:?}",
        config.get::<u32>("example-string")
    );
    println!();

    println!("Create transaction");
    let tx = config.transaction();
    println!(
        "Set example-bool to false: {:?}",
        tx.set("example-bool", false)
    );
    println!("Set example-int to 0: {:?}", tx.set("example-int", 0));
    println!(
        "Set example-string to \"\": {:?}",
        tx.set("example-string", "")
    );
    println!("Committing transaction");
    println!("Commit transaction: {:?}", tx.commit());
}

pub fn main() {
    println!("Testing config");
    test_config(Config::new("com.system76.Example", 1).unwrap());

    println!("Testing state");
    test_config(Config::new_state("com.system76.Example", 1).unwrap());
}
