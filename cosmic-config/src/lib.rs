use notify::{
    event::{EventKind, ModifyKind},
    Watcher,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[cfg(feature = "subscription")]
mod subscription;
pub use subscription::*;

#[cfg(all(feature = "dbus", feature = "subscription"))]
pub mod dbus;

#[cfg(feature = "macro")]
pub use cosmic_config_derive;

#[cfg(feature = "calloop")]
pub mod calloop;

#[derive(Debug)]
pub enum Error {
    AtomicWrites(atomicwrites::Error<std::io::Error>),
    InvalidName(String),
    Io(std::io::Error),
    NoConfigDirectory,
    Notify(notify::Error),
    Ron(ron::Error),
    RonSpanned(ron::error::SpannedError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AtomicWrites(err) => err.fmt(f),
            Self::InvalidName(name) => write!(f, "invalid config name '{}'", name),
            Self::Io(err) => err.fmt(f),
            Self::NoConfigDirectory => write!(f, "cosmic config directory not found"),
            Self::Notify(err) => err.fmt(f),
            Self::Ron(err) => err.fmt(f),
            Self::RonSpanned(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<atomicwrites::Error<std::io::Error>> for Error {
    fn from(f: atomicwrites::Error<std::io::Error>) -> Self {
        Self::AtomicWrites(f)
    }
}

impl From<std::io::Error> for Error {
    fn from(f: std::io::Error) -> Self {
        Self::Io(f)
    }
}

impl From<notify::Error> for Error {
    fn from(f: notify::Error) -> Self {
        Self::Notify(f)
    }
}

impl From<ron::Error> for Error {
    fn from(f: ron::Error) -> Self {
        Self::Ron(f)
    }
}

impl From<ron::error::SpannedError> for Error {
    fn from(f: ron::error::SpannedError) -> Self {
        Self::RonSpanned(f)
    }
}

pub trait ConfigGet {
    /// Get a configuration value
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error>;
}

pub trait ConfigSet {
    /// Set a configuration value
    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct Config {
    system_path: PathBuf,
    user_path: PathBuf,
}

impl Config {
    /// Get the config for the libcosmic toolkit
    pub fn libcosmic() -> Result<Self, Error> {
        Self::new("com.system76.libcosmic", 1)
    }

    /// Get config for the given application name and config version
    // Use folder at XDG config/name for config storage, return Config if successful
    //TODO: fallbacks for flatpak (HOST_XDG_CONFIG_HOME, xdg-desktop settings proxy)
    pub fn new(name: &str, version: u64) -> Result<Self, Error> {
        // Get libcosmic system defaults path
        //TODO: support non-UNIX OS
        let cosmic_system_path = Path::new("/usr/share/cosmic");
        // Append [name]/v[version]
        let system_path = cosmic_system_path.join(name).join(format!("v{}", version));

        // Get libcosmic user configuration directory
        let cosmic_user_path = dirs::config_dir()
            .ok_or(Error::NoConfigDirectory)?
            .join("cosmic");
        // Append [name]/v[version]
        let user_path = cosmic_user_path.join(name).join(format!("v{}", version));

        // If the app paths are children of the cosmic paths
        if system_path.starts_with(&cosmic_system_path) && user_path.starts_with(&cosmic_user_path)
        {
            // Create app user path
            fs::create_dir_all(&user_path)?;
            // Return Config
            Ok(Self {
                system_path,
                user_path,
            })
        } else {
            // Return error for invalid name
            Err(Error::InvalidName(name.to_string()))
        }
    }

    /// Get state for the given application name and config version. State is meant to be used to
    /// store items that may need to be exposed to other programs but will change regularly without
    /// user action
    // Use folder at XDG config/name for config storage, return Config if successful
    //TODO: fallbacks for flatpak (HOST_XDG_CONFIG_HOME, xdg-desktop settings proxy)
    pub fn new_state(name: &str, version: u64) -> Result<Self, Error> {
        // Get libcosmic system defaults path
        //TODO: support non-UNIX OS
        let cosmic_system_path = Path::new("/var/lib/cosmic");
        // Append [name]/v[version]
        let system_path = cosmic_system_path.join(name).join(format!("v{}", version));

        // Get libcosmic user configuration directory
        let cosmic_user_path = dirs::state_dir()
            .ok_or(Error::NoConfigDirectory)?
            .join("cosmic");
        // Append [name]/v[version]
        let user_path = cosmic_user_path.join(name).join(format!("v{}", version));

        // If the app paths are children of the cosmic paths
        if system_path.starts_with(&cosmic_system_path) && user_path.starts_with(&cosmic_user_path)
        {
            // Create app user path
            fs::create_dir_all(&user_path)?;
            // Return Config
            Ok(Self {
                system_path,
                user_path,
            })
        } else {
            // Return error for invalid name
            Err(Error::InvalidName(name.to_string()))
        }
    }

    // Start a transaction (to set multiple configs at the same time)
    pub fn transaction<'a>(&'a self) -> ConfigTransaction<'a> {
        ConfigTransaction {
            config: self,
            updates: Mutex::new(Vec::new()),
        }
    }

    // Watch keys for changes, will be triggered once per transaction
    // This may end up being an mpsc channel instead of a function
    // See EventHandler in the notify crate: https://docs.rs/notify/latest/notify/trait.EventHandler.html
    // Having a callback allows for any application abstraction to be used
    pub fn watch<F>(&self, f: F) -> Result<notify::RecommendedWatcher, Error>
    // Argument is an array of all keys that changed in that specific transaction
    //TODO: simplify F requirements
    where
        F: Fn(&Self, &[String]) + Send + Sync + 'static,
    {
        let watch_config = self.clone();
        let mut watcher =
            notify::recommended_watcher(move |event_res: Result<notify::Event, notify::Error>| {
                // println!("{:#?}", event_res);
                match &event_res {
                    Ok(event) => {
                        match &event.kind {
                            EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) => {
                                // Data not mutated
                                return;
                            }
                            _ => {}
                        }

                        let mut keys = Vec::new();
                        for path in event.paths.iter() {
                            match path.strip_prefix(&watch_config.user_path) {
                                Ok(key_path) => match key_path.to_str() {
                                    Some(key) => {
                                        // Skip any .atomicwrite temporary files
                                        if key.starts_with(".atomicwrite") {
                                            continue;
                                        }
                                        keys.push(key.to_string());
                                    }
                                    None => {
                                        //TODO: handle errors
                                    }
                                },
                                Err(_err) => {
                                    //TODO: handle errors
                                }
                            }
                        }
                        if !keys.is_empty() {
                            f(&watch_config, &keys);
                        }
                    }
                    Err(_err) => {
                        //TODO: handle errors
                    }
                }
            })?;
        watcher.watch(&self.user_path, notify::RecursiveMode::NonRecursive)?;
        Ok(watcher)
    }

    fn default_path(&self, key: &str) -> Result<PathBuf, Error> {
        let default_path = self.system_path.join(key);
        // Ensure key path is a direct child of config directory
        if default_path.parent() == Some(&self.system_path) {
            Ok(default_path)
        } else {
            Err(Error::InvalidName(key.to_string()))
        }
    }

    fn key_path(&self, key: &str) -> Result<PathBuf, Error> {
        let key_path = self.user_path.join(key);
        // Ensure key path is a direct child of config directory
        if key_path.parent() == Some(&self.user_path) {
            Ok(key_path)
        } else {
            Err(Error::InvalidName(key.to_string()))
        }
    }
}

// Getting any setting is available on a Config object
impl ConfigGet for Config {
    //TODO: check for transaction
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        // If key path exists
        let key_path = self.key_path(key)?;
        let data = if key_path.is_file() {
            // Load user override
            fs::read_to_string(key_path)?
        } else {
            // Load system default
            let default_path = self.default_path(key)?;
            fs::read_to_string(default_path)?
        };
        let t = ron::from_str(&data)?;
        Ok(t)
    }
}

// Setting any setting in this way will do one transaction per set call
impl ConfigSet for Config {
    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error> {
        // Wrap up single key/value sets in a transaction
        let tx = self.transaction();
        tx.set(key, value)?;
        tx.commit()
    }
}

#[must_use = "Config transaction must be committed"]
pub struct ConfigTransaction<'a> {
    config: &'a Config,
    //TODO: use map?
    updates: Mutex<Vec<(PathBuf, String)>>,
}

impl<'a> ConfigTransaction<'a> {
    /// Apply all pending changes from ConfigTransaction
    //TODO: apply all changes at once
    pub fn commit(self) -> Result<(), Error> {
        let mut updates = self.updates.lock().unwrap();
        for (key_path, data) in updates.drain(..) {
            atomicwrites::AtomicFile::new(
                key_path,
                atomicwrites::OverwriteBehavior::AllowOverwrite,
            )
            .write(|file| file.write_all(data.as_bytes()))?;
        }
        Ok(())
    }
}

// Setting any setting in this way will do one transaction for all settings
// when commit finishes that transaction
impl<'a> ConfigSet for ConfigTransaction<'a> {
    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error> {
        //TODO: sanitize key (no slashes, cannot be . or ..)
        let key_path = self.config.key_path(key)?;
        let data = ron::to_string(&value)?;
        //TODO: replace duplicates?
        {
            let mut updates = self.updates.lock().unwrap();
            updates.push((key_path, data));
        }
        Ok(())
    }
}

pub trait CosmicConfigEntry
where
    Self: Sized,
{
    const VERSION: u64;

    fn write_entry(&self, config: &Config) -> Result<(), crate::Error>;
    fn get_entry(config: &Config) -> Result<Self, (Vec<crate::Error>, Self)>;
    /// Returns the keys that were updated
    fn update_keys<T: AsRef<str>>(
        &mut self,
        config: &Config,
        changed_keys: &[T],
    ) -> (Vec<crate::Error>, Vec<&'static str>);
}
