//! Integrations for cosmic-config — the cosmic configuration system.

use notify::{
    RecommendedWatcher, Watcher,
    event::{EventKind, ModifyKind, RenameMode},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[cfg(feature = "subscription")]
mod subscription;
#[cfg(feature = "subscription")]
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
    NotFound,
    Ron(ron::Error),
    RonSpanned(ron::error::SpannedError),
    GetKey(String, std::io::Error),
}

impl fmt::Display for Error {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AtomicWrites(err) => err.fmt(f),
            Self::InvalidName(name) => write!(f, "invalid config name '{}'", name),
            Self::Io(err) => err.fmt(f),
            Self::NoConfigDirectory => write!(f, "cosmic config directory not found"),
            Self::Notify(err) => err.fmt(f),
            Self::NotFound => write!(f, "cosmic config key not configured"),
            Self::Ron(err) => err.fmt(f),
            Self::RonSpanned(err) => err.fmt(f),
            Self::GetKey(key, err) => write!(f, "failed to get key '{}': {}", key, err),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Whether the reason for the missing config is caused by an error.
    ///
    /// Useful for determining if it is appropriate to log as an error.
    #[inline]
    pub fn is_err(&self) -> bool {
        !matches!(self, Self::NoConfigDirectory | Self::NotFound)
    }
}

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
    ///
    /// Fallback to the system default if a local user override is not defined.
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error>;

    /// Get a locally-defined configuration value from the user's local config.
    fn get_local<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error>;

    /// Get the system-defined default configuration value.
    fn get_system_default<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error>;
}

pub trait ConfigSet {
    /// Set a configuration value
    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct Config {
    system_path: Option<PathBuf>,
    user_path: Option<PathBuf>,
}

/// Check that the name is relative and doesn't contain . or ..
fn sanitize_name(name: &str) -> Result<&Path, Error> {
    let path = Path::new(name);
    if path
        .components()
        .all(|x| matches!(x, std::path::Component::Normal(_)))
    {
        Ok(path)
    } else {
        Err(Error::InvalidName(name.to_owned()))
    }
}

impl Config {
    /// Get a system config for the given name and config version
    pub fn system(name: &str, version: u64) -> Result<Self, Error> {
        let path = sanitize_name(name)?.join(format!("v{version}"));
        #[cfg(unix)]
        let system_path = xdg::BaseDirectories::with_prefix("cosmic").find_data_file(path);

        #[cfg(windows)]
        let system_path =
            known_folders::get_known_folder_path(known_folders::KnownFolder::ProgramFilesCommon)
                .map(|x| x.join("COSMIC").join(&path));

        Ok(Self {
            system_path,
            user_path: None,
        })
    }

    /// Get config for the given application name and config version
    // Use folder at XDG config/name for config storage, return Config if successful
    //TODO: fallbacks for flatpak (HOST_XDG_CONFIG_HOME, xdg-desktop settings proxy)
    pub fn new(name: &str, version: u64) -> Result<Self, Error> {
        // Look for [name]/v[version]
        let path = sanitize_name(name)?.join(format!("v{}", version));

        // Search data file, which provides default (e.g. /usr/share)
        #[cfg(unix)]
        let system_path = xdg::BaseDirectories::with_prefix("cosmic").find_data_file(&path);

        #[cfg(windows)]
        let system_path =
            known_folders::get_known_folder_path(known_folders::KnownFolder::ProgramFilesCommon)
                .map(|x| x.join("COSMIC").join(&path));

        // Get libcosmic user configuration directory
        let mut user_path = dirs::config_dir().ok_or(Error::NoConfigDirectory)?;
        user_path.push("cosmic");
        user_path.push(path);

        // Create new configuration directory if not found.
        fs::create_dir_all(&user_path)?;

        // Return Config
        Ok(Self {
            system_path,
            user_path: Some(user_path),
        })
    }

    /// Get config for the given application name and config version and custom path.
    pub fn with_custom_path(name: &str, version: u64, custom_path: PathBuf) -> Result<Self, Error> {
        // Look for [name]/v[version]
        let path = sanitize_name(name)?.join(format!("v{version}"));

        let mut user_path = custom_path;
        user_path.push("cosmic");
        user_path.push(path);
        // Create new configuration directory if not found.
        fs::create_dir_all(&user_path)?;

        // Return Config
        Ok(Self {
            system_path: None,
            user_path: Some(user_path),
        })
    }

    /// Get state for the given application name and config version. State is meant to be used to
    /// store items that may need to be exposed to other programs but will change regularly without
    /// user action
    // Use folder at XDG config/name for config storage, return Config if successful
    //TODO: fallbacks for flatpak (HOST_XDG_CONFIG_HOME, xdg-desktop settings proxy)
    pub fn new_state(name: &str, version: u64) -> Result<Self, Error> {
        // Look for [name]/v[version]
        let path = sanitize_name(name)?.join(format!("v{}", version));

        // Get libcosmic user state directory
        let mut user_path = dirs::state_dir().ok_or(Error::NoConfigDirectory)?;
        user_path.push("cosmic");
        user_path.push(path);
        // Create new state directory if not found.
        fs::create_dir_all(&user_path)?;

        Ok(Self {
            system_path: None,
            user_path: Some(user_path),
        })
    }

    // Start a transaction (to set multiple configs at the same time)
    #[inline]
    pub fn transaction(&self) -> ConfigTransaction<'_> {
        ConfigTransaction {
            config: self,
            updates: Mutex::new(Vec::new()),
        }
    }

    // Watch keys for changes, will be triggered once per transaction
    // This may end up being an mpsc channel instead of a function
    // See EventHandler in the notify crate: https://docs.rs/notify/latest/notify/trait.EventHandler.html
    // Having a callback allows for any application abstraction to be used
    pub fn watch<F>(&self, f: F) -> Result<RecommendedWatcher, Error>
    // Argument is an array of all keys that changed in that specific transaction
    //TODO: simplify F requirements
    where
        F: Fn(&Self, &[String]) + Send + Sync + 'static,
    {
        let watch_config = self.clone();
        let Some(user_path) = self.user_path.as_ref() else {
            return Err(Error::NoConfigDirectory);
        };
        let user_path_clone = user_path.clone();
        let mut watcher =
            notify::recommended_watcher(move |event_res: Result<notify::Event, notify::Error>| {
                match event_res {
                    Ok(event) => {
                        match &event.kind {
                            EventKind::Access(_)
                            | EventKind::Modify(ModifyKind::Metadata(_))
                            | EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                                // Data not mutated
                                return;
                            }
                            _ => {}
                        }

                        let mut keys = Vec::new();
                        for path in &event.paths {
                            match path.strip_prefix(&user_path_clone) {
                                Ok(key_path) => {
                                    if let Some(key) = key_path.to_str() {
                                        // Skip any .atomicwrite temporary files
                                        if key.starts_with(".atomicwrite") {
                                            continue;
                                        }
                                        keys.push(key.to_string());
                                    }
                                }
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
        watcher.watch(user_path, notify::RecursiveMode::Recursive)?;
        Ok(watcher)
    }

    fn default_path(&self, key: &str) -> Result<PathBuf, Error> {
        let Some(system_path) = self.system_path.as_ref() else {
            return Err(Error::NoConfigDirectory);
        };

        Ok(system_path.join(sanitize_name(key)?))
    }

    /// Get the path of the key in the user's local config directory.
    fn key_path(&self, key: &str) -> Result<PathBuf, Error> {
        let Some(user_path) = self.user_path.as_ref() else {
            return Err(Error::NoConfigDirectory);
        };
        Ok(user_path.join(sanitize_name(key)?))
    }
}

// Getting any setting is available on a Config object
impl ConfigGet for Config {
    //TODO: check for transaction
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        match self.get_local(key) {
            Ok(value) => Ok(value),
            Err(Error::NotFound) => self.get_system_default(key),
            Err(why) => Err(why),
        }
    }

    fn get_local<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        // If key path exists
        match self.key_path(key) {
            Ok(key_path) if key_path.is_file() => {
                // Load user override
                let data = fs::read_to_string(key_path)
                    .map_err(|err| Error::GetKey(key.to_string(), err))?;

                Ok(ron::from_str(&data)?)
            }

            _ => Err(Error::NotFound),
        }
    }

    fn get_system_default<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        // Load system default
        let default_path = self.default_path(key)?;
        let data =
            fs::read_to_string(default_path).map_err(|err| Error::GetKey(key.to_string(), err))?;
        Ok(ron::from_str(&data)?)
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

impl ConfigTransaction<'_> {
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
impl ConfigSet for ConfigTransaction<'_> {
    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error> {
        //TODO: sanitize key (no slashes, cannot be . or ..)
        let key_path = self.config.key_path(key)?;
        let data = ron::ser::to_string_pretty(&value, ron::ser::PrettyConfig::new())?;
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

#[derive(Debug)]
pub struct Update<T> {
    pub errors: Vec<crate::Error>,
    pub keys: Vec<&'static str>,
    pub config: T,
}
