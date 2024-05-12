use std::ops::Deref;

use crate::{CosmicConfigEntry, Update};
use cosmic_settings_daemon::{Changed, ConfigProxy, CosmicSettingsDaemonProxy};
use futures_util::SinkExt;
use iced_futures::futures::{self, future::pending, StreamExt};

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
        ConfigProxy::builder(settings_daemon_proxy.connection())
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
        ConfigProxy::builder(settings_daemon_proxy.connection())
            .path(path)?
            .destination(name)?
            .build()
            .await
            .map(|proxy| Self { proxy })
    }
}

#[allow(clippy::too_many_lines)]
pub fn watcher_subscription<T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone>(
    settings_daemon: CosmicSettingsDaemonProxy<'static>,
    config_id: &'static str,
    is_state: bool,
) -> iced_futures::Subscription<Update<T>> {
    enum Change {
        Changes(Changed),
        OwnerChanged(bool),
    }

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

        let mut attempts = 0;

        loop {
            let watcher = if is_state {
                Watcher::new_state(&settings_daemon, config_id, version).await
            } else {
                Watcher::new_config(&settings_daemon, config_id, version).await
            };
            let Ok(watcher) = watcher else {
                tracing::error!("Failed to create watcher for {config_id}");

                #[cfg(feature = "tokio")]
                ::tokio::time::sleep(::tokio::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(feature = "async-std")]
                async_std::task::sleep(std::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(not(any(feature = "tokio", feature = "async-std")))]
                {
                    pending::<()>().await;
                    unreachable!();
                }
                attempts += 1;
                // The settings daemon has exited
                continue;
            };
            let Ok(changes) = watcher.receive_changed().await else {
                tracing::error!("Failed to listen for changes for {config_id}");

                #[cfg(feature = "tokio")]
                ::tokio::time::sleep(::tokio::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(feature = "async-std")]
                async_std::task::sleep(std::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(not(any(feature = "tokio", feature = "async-std")))]
                {
                    pending::<()>().await;
                    unreachable!();
                }
                attempts += 1;
                // The settings daemon has exited
                continue;
            };

            let mut changes = changes.map(Change::Changes).fuse();

            let Ok(owner_changed) = watcher.receive_owner_changed().await else {
                tracing::error!("Failed to listen for owner changes for {config_id}");
                #[cfg(feature = "tokio")]
                ::tokio::time::sleep(::tokio::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(feature = "async-std")]
                async_std::task::sleep(std::time::Duration::from_secs(2_u64.pow(attempts))).await;
                #[cfg(not(any(feature = "tokio", feature = "async-std")))]
                {
                    pending::<()>().await;
                    unreachable!();
                }
                attempts += 1;
                // The settings daemon has exited
                continue;
            };
            let mut owner_changed = owner_changed
                .map(|c| Change::OwnerChanged(c.is_some()))
                .fuse();
            loop {
                let change: Changed = futures::select! {
                    c = changes.next() => {
                        let Some(Change::Changes(c)) = c else {
                            break;
                        };
                        c
                    }
                    c = owner_changed.next() => {
                        let Some(Change::OwnerChanged(cont)) = c else {
                            break;
                        };
                        if cont {
                            continue;
                        } else {
                            // The settings daemon has exited
                            break;
                        }
                    },
                };

                // Reset the attempts counter if we received a change
                attempts = 0;
                let Ok(args) = change.args() else {
                    // The settings daemon has exited
                    break;
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
