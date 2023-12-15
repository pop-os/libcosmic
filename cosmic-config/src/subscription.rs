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
    Update(T),
    UpdateError(T, Vec<crate::Error>),
    Failed,
}

pub fn config_subscription<
    I: 'static + Copy + Send + Sync + Hash,
    T: 'static + Send + Sync + PartialEq + Clone + CosmicConfigEntry,
>(
    id: I,
    config_id: Cow<'static, str>,
    config_version: u64,
) -> iced_futures::Subscription<(I, Result<T, (Vec<crate::Error>, T)>)> {
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
) -> iced_futures::Subscription<(I, Result<T, (Vec<crate::Error>, T)>)> {
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
    output: &mut mpsc::Sender<(I, Result<T, (Vec<crate::Error>, T)>)>,
    id: I,
) -> ConfigState<T> {
    use iced_futures::futures::{future::pending, StreamExt};

    match state {
        ConfigState::Init(config_id, version, is_state) => {
            let (tx, rx) = mpsc::channel(100);
            let config = match if is_state {
                Config::new_state(&config_id, version)
            } else {
                Config::new(&config_id, version)
            } {
                Ok(c) => c,
                Err(_) => return ConfigState::Failed,
            };
            let watcher = match config.watch(move |_helper, keys| {
                let mut tx = tx.clone();
                let _ = tx.try_send(keys.to_vec());
            }) {
                Ok(w) => w,
                Err(_) => return ConfigState::Failed,
            };

            match T::get_entry(&config) {
                Ok(t) => {
                    _ = output.send((id, Ok(t.clone()))).await;
                    ConfigState::Waiting(t, watcher, rx, config)
                }
                Err((errors, t)) => {
                    _ = output.send((id, Err((errors, t.clone())))).await;
                    ConfigState::Waiting(t, watcher, rx, config)
                }
            }
        }
        ConfigState::Waiting(mut conf_data, watcher, mut rx, config) => match rx.next().await {
            Some(keys) => {
                let (errors, changed) = conf_data.update_keys(&config, &keys);

                if !changed.is_empty() {
                    if errors.is_empty() {
                        _ = output.send((id, Ok(conf_data.clone()))).await;
                    } else {
                        _ = output.send((id, Err((errors, conf_data.clone())))).await;
                    }
                }
                ConfigState::Waiting(conf_data, watcher, rx, config)
            }
            None => ConfigState::Failed,
        },
        ConfigState::Failed => pending().await,
    }
}
