//! Keeps the only test that changes XDG environment variables in its own binary.
#![cfg(unix)]

use cosmic_config::{Config, ConfigGet};
use std::fs;

#[test]
fn empty_xdg_data_home_dir_does_not_hide_xdg_data_dirs_defaults() {
    let root = std::env::temp_dir().join(format!("cosmic-config-xdg-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let data_home = root.join("data-home");
    let data_dir = root.join("data-dir");
    let app = "com.example.CosmicConfigTest/v1";
    fs::create_dir_all(data_home.join("cosmic").join(app)).unwrap();
    fs::create_dir_all(data_dir.join("cosmic").join(app)).unwrap();
    fs::write(data_dir.join("cosmic").join(app).join("key"), "42").unwrap();

    // SAFETY: this is the only test in this binary, so no other thread reads the environment.
    unsafe {
        std::env::set_var("XDG_DATA_HOME", &data_home);
        std::env::set_var("XDG_DATA_DIRS", &data_dir);
    }

    let config = Config::system("com.example.CosmicConfigTest", 1).unwrap();
    assert_eq!(config.get_system_default::<u32>("key").unwrap(), 42);

    fs::remove_dir_all(root).unwrap();
}
