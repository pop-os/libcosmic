use std::ops::Deref;

use crate::{CosmicConfigEntry, Update};
use cosmic_settings_daemon::{ConfigProxy, CosmicSettingsDaemonProxy};
use futures_util::SinkExt;
use iced_futures::futures::{future::pending, StreamExt};
pub async fn settings_daemon_proxy() -> zbus::Result<CosmicSettingsDaemonProxy<'static>> {
    let conn = zbus::Connection::session().await?;
    CosmicSettingsDaemonProxy::new(&conn).await
}

#[derive(Debug)]
pub struct Watcher {
    proxy: ConfigProxy<'static>,
}

impl Deref for Watcher {
    type Target = ConfigProxy<'static>;
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

impl Watcher {
    pub async fn new_config(
        settings_daemon_proxy: &CosmicSettingsDaemonProxy<'static>,
        id: &str,
        version: u64,
    ) -> zbus::Result<Self> {
        let (path, name) = settings_daemon_proxy.watch_config(id, version).await?;
        ConfigProxy::builder(settings_daemon_proxy.inner().connection())
            .path(path)?
            .destination(name)?
            .build()
            .await
            .map(|proxy| Self { proxy })
    }

    pub async fn new_state(
        settings_daemon_proxy: &CosmicSettingsDaemonProxy<'static>,
        id: &str,
        version: u64,
    ) -> zbus::Result<Self> {
        let (path, name) = settings_daemon_proxy.watch_state(id, version).await?;
        ConfigProxy::builder(settings_daemon_proxy.inner().connection())
            .path(path)?
            .destination(name)?
            .build()
            .await
            .map(|proxy| Self { proxy })
    }
}

pub fn watcher_subscription<T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone>(
    settings_daemon: CosmicSettingsDaemonProxy<'static>,
    config_id: &'static str,
    is_state: bool,
) -> iced_futures::Subscription<Update<T>> {
    let id = std::any::TypeId::of::<T>();
    iced_futures::subscription::channel((is_state, config_id, id), 5, move |mut tx| async move {
        let version = T::VERSION;

        let Ok(cosmic_config) = (if is_state {
            crate::Config::new_state(config_id, version)
        } else {
            crate::Config::new(config_id, version)
        }) else {
            pending::<()>().await;
            unreachable!();
        };
        let mut config = match T::get_entry(&cosmic_config) {
            Ok(config) => config,
            Err((errors, default)) => {
                if !errors.is_empty() {
                    eprintln!("Error getting config: {config_id} {errors:?}");
                }
                default
            }
        };
        if let Err(err) = tx
            .send(Update {
                errors: Vec::new(),
                keys: Vec::new(),
                config: config.clone(),
            })
            .await
        {
            eprintln!("Failed to send config: {err}");
        }

        let watcher = if is_state {
            Watcher::new_state(&settings_daemon, config_id, version).await
        } else {
            Watcher::new_config(&settings_daemon, config_id, version).await
        };
        let Ok(watcher) = watcher else {
            pending::<()>().await;
            unreachable!();
        };

        loop {
            let Ok(mut changes) = watcher.receive_changed().await else {
                pending::<()>().await;
                unreachable!();
            };
            while let Some(change) = changes.next().await {
                let Ok(args) = change.args() else {
                    continue;
                };
                let (errors, keys) = config.update_keys(&cosmic_config, &[args.key]);
                if !keys.is_empty() {
                    if let Err(err) = tx
                        .send(Update {
                            errors,
                            keys,
                            config: config.clone(),
                        })
                        .await
                    {
                        eprintln!("Failed to send config update: {err}");
                    }
                }
            }
        }
    })
}
