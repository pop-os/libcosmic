use iced_futures::futures::{SinkExt, Stream};
use iced_futures::{futures::channel::mpsc, stream};
use notify::RecommendedWatcher;
use std::{borrow::Cow, hash::Hash};

use crate::{Config, CosmicConfigEntry};

pub enum ConfigState<T> {
    Init(Cow<'static, str>, u64, bool),
    Waiting(T, RecommendedWatcher, mpsc::Receiver<Vec<String>>, Config),
    Failed,
}

pub enum ConfigUpdate<T> {
    Update(crate::Update<T>),
    Failed,
}

#[cold]
pub fn config_subscription<
    I: 'static + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<crate::Update<T>> {
    iced_futures::Subscription::run_with_id(id, watcher_stream(config_id, config_version, false))
}

#[cold]
pub fn config_state_subscription<
    I: 'static + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<crate::Update<T>> {
    iced_futures::Subscription::run_with_id(id, watcher_stream(config_id, config_version, true))
}

fn watcher_stream<T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry>(
    config_id: Cow<'static, str>,
    config_version: u64,
    is_state: bool,
) -> impl Stream<Item = crate::Update<T>> {
    stream::channel(100, move |mut output| {
        let config_id = config_id.clone();
        async move {
            let config_id = config_id.clone();
            let mut state = ConfigState::Init(config_id, config_version, is_state);

            loop {
                state = start_listening::<T>(state, &mut output).await;
            }
        }
    })
}

async fn start_listening<T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry>(
    state: ConfigState<T>,
    output: &mut mpsc::Sender<crate::Update<T>>,
) -> ConfigState<T> {
    use iced_futures::futures::{StreamExt, future::pending};

    match state {
        ConfigState::Init(config_id, version, is_state) => {
            let (tx, rx) = mpsc::channel(100);
            let Ok(config) = (if is_state {
                Config::new_state(&config_id, version)
            } else {
                Config::new(&config_id, version)
            }) else {
                return ConfigState::Failed;
            };
            let Ok(watcher) = config.watch(move |_helper, keys| {
                let mut tx = tx.clone();
                let _ = tx.try_send(keys.to_vec());
            }) else {
                return ConfigState::Failed;
            };

            match T::get_entry(&config) {
                Ok(t) => {
                    let update = crate::Update {
                        errors: Vec::new(),
                        keys: Vec::new(),
                        config: t.clone(),
                    };
                    _ = output.send(update).await;
                    ConfigState::Waiting(t, watcher, rx, config)
                }
                Err((errors, t)) => {
                    let update = crate::Update {
                        errors,
                        keys: Vec::new(),
                        config: t.clone(),
                    };
                    _ = output.send(update).await;
                    ConfigState::Waiting(t, watcher, rx, config)
                }
            }
        }
        ConfigState::Waiting(mut conf_data, watcher, mut rx, config) => match rx.next().await {
            Some(keys) => {
                let (errors, changed) = conf_data.update_keys(&config, &keys);

                if !changed.is_empty() {
                    _ = output
                        .send(crate::Update {
                            errors,
                            keys: changed,
                            config: conf_data.clone(),
                        })
                        .await;
                }
                ConfigState::Waiting(conf_data, watcher, rx, config)
            }
            None => ConfigState::Failed,
        },
        ConfigState::Failed => pending().await,
    }
}
