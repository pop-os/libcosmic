#[cfg(feature = "subscription")]
use iced_futures::futures::channel::mpsc;
#[cfg(feature = "subscription")]
use iced_futures::subscription;
use notify::{
    event::{EventKind, ModifyKind},
    RecommendedWatcher, Watcher,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    borrow::Cow,
    fs,
    hash::Hash,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

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
                                Err(err) => {
                                    //TODO: handle errors
                                }
                            }
                        }
                        if !keys.is_empty() {
                            f(&watch_config, &keys);
                        }
                    }
                    Err(err) => {
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

#[cfg(feature = "subscription")]
pub enum ConfigState<T> {
    Init(Cow<'static, str>, u64),
    Waiting(T, RecommendedWatcher, mpsc::Receiver<()>, Config),
    Failed,
}

#[cfg(feature = "subscription")]
pub enum ConfigUpdate<T> {
    Update(T),
    UpdateError(T, Vec<crate::Error>),
    Failed,
}

pub trait CosmicConfigEntry
where
    Self: Sized,
{
    fn write_entry(&self, config: &Config) -> Result<(), crate::Error>;
    fn get_entry(config: &Config) -> Result<Self, (Vec<crate::Error>, Self)>;
}

#[cfg(feature = "subscription")]
pub fn config_subscription<
    I: 'static + Copy + Send + Sync + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<(I, Result<T, (Vec<crate::Error>, T)>)> {
    subscription::unfold(
        id,
        ConfigState::Init(config_id, config_version),
        move |state| start_listening_loop(id, state),
    )
}

#[cfg(feature = "subscription")]
async fn start_listening<
    I: Copy,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    state: ConfigState<T>,
) -> (
    Option<(I, Result<T, (Vec<crate::Error>, T)>)>,
    ConfigState<T>,
) {
    use iced_futures::futures::{future::pending, StreamExt};

    match state {
        ConfigState::Init(config_id, version) => {
            let (tx, rx) = mpsc::channel(100);
            let config = match Config::new(&config_id, version) {
                Ok(c) => c,
                Err(_) => return (None, ConfigState::Failed),
            };
            let watcher = match config.watch(move |_helper, _keys| {
                let mut tx = tx.clone();
                let _ = tx.try_send(());
            }) {
                Ok(w) => w,
                Err(_) => return (None, ConfigState::Failed),
            };

            match T::get_entry(&config) {
                Ok(t) => (
                    Some((id, Ok(t.clone()))),
                    ConfigState::Waiting(t, watcher, rx, config),
                ),
                Err((errors, t)) => (
                    Some((id, Err((errors, t.clone())))),
                    ConfigState::Waiting(t, watcher, rx, config),
                ),
            }
        }
        ConfigState::Waiting(old, watcher, mut rx, config) => match rx.next().await {
            Some(_) => match T::get_entry(&config) {
                Ok(t) => (
                    if t != old {
                        Some((id, Ok(t.clone())))
                    } else {
                        None
                    },
                    ConfigState::Waiting(t, watcher, rx, config),
                ),
                Err((errors, t)) => (
                    if t != old {
                        Some((id, Err((errors, t.clone()))))
                    } else {
                        None
                    },
                    ConfigState::Waiting(t, watcher, rx, config),
                ),
            },

            None => (None, ConfigState::Failed),
        },
        ConfigState::Failed => pending().await,
    }
}

#[cfg(feature = "subscription")]
async fn start_listening_loop<
    I: Copy,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    mut state: ConfigState<T>,
) -> ((I, Result<T, (Vec<crate::Error>, T)>), ConfigState<T>) {
    loop {
        let (update, new_state) = start_listening(id, state).await;
        state = new_state;
        if let Some(update) = update {
            return (update, state);
        }
    }
}
