use iced_futures::futures::SinkExt;
use iced_futures::{futures::channel::mpsc, subscription};
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

pub fn config_subscription<
    I: 'static + Copy + Send + Sync + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<crate::Update<T>> {
    subscription::channel(id, 100, move |mut output| {
        let config_id = config_id.clone();
        async move {
            let config_id = config_id.clone();
            let mut state = ConfigState::Init(config_id, config_version, false);

            loop {
                state = start_listening(state, &mut output, id).await;
            }
        }
    })
}

pub fn config_state_subscription<
    I: 'static + Copy + Send + Sync + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<crate::Update<T>> {
    subscription::channel(id, 100, move |mut output| {
        let config_id = config_id.clone();
        async move {
            let config_id = config_id.clone();
            let mut state = ConfigState::Init(config_id, config_version, true);

            loop {
                state = start_listening(state, &mut output, id).await;
            }
        }
    })
}

async fn start_listening<
    I: Copy,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    state: ConfigState<T>,
    output: &mut mpsc::Sender<crate::Update<T>>,
    id: I,
) -> ConfigState<T> {
    use iced_futures::futures::{future::pending, StreamExt};

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
                        errors: errors,
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
