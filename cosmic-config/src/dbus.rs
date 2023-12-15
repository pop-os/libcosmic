use std::ops::Deref;

use crate::CosmicConfigEntry;
use cosmic_settings_daemon::{Changed, ConfigProxy, CosmicSettingsDaemonProxy, Ping};
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
        ConfigProxy::builder(settings_daemon_proxy.connection())
            .path(path)?
            .destination(name)?
            .build()
            .await
            .map(|proxy| Self { proxy })
    }
}

#[derive(Debug)]
pub struct ConfigUpdate<T> {
    pub errors: Vec<crate::Error>,
    pub keys: Vec<&'static str>,
    pub config: T,
}

pub fn watcher_subscription<T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone>(
    settings_daemon: CosmicSettingsDaemonProxy<'static>,
    config_id: &'static str,
) -> iced_futures::Subscription<ConfigUpdate<T>> {
    let id = std::any::TypeId::of::<T>();
    iced_futures::subscription::channel((config_id, id), 5, move |mut tx| async move {
        let version = T::VERSION;
        let Ok(cosmic_config) = crate::Config::new(config_id, version) else {
            pending::<()>().await;
            unreachable!();
        };
        let mut config = match T::get_entry(&cosmic_config) {
            Ok(config) => config,
            Err((errors, default)) => {
                if !errors.is_empty() {
                    eprintln!("Failed to get config: {errors:?}");
                }
                default
            }
        };
        if let Err(err) = tx
            .send(ConfigUpdate {
                errors: Vec::new(),
                keys: Vec::new(),
                config: config.clone(),
            })
            .await
        {
            eprintln!("Failed to send config: {err}");
        }

        let Ok(watcher) = Watcher::new_config(&settings_daemon, config_id, version).await else {
            pending::<()>().await;
            unreachable!();
        };

        loop {
            let Ok(changes) = watcher.receive_changed().await else {
                pending::<()>().await;
                unreachable!();
            };
            let Ok(pings) = watcher.receive_ping().await else {
                pending::<()>().await;
                unreachable!();
            };
            let mut streams = futures_util::stream_select!(
                changes.map(Message::ConfigChanged),
                pings.map(Message::ConfigPing)
            );
            while let Some(v) = streams.next().await {
                match v {
                    Message::ConfigChanged(change) => {
                        let Ok(args) = change.args() else {
                            continue;
                        };
                        let (errors, keys) = config.update_keys(&cosmic_config, &[args.key]);
                        if !keys.is_empty() {
                            if let Err(err) = tx
                                .send(ConfigUpdate {
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
                    Message::ConfigPing(_) => {
                        // send pong
                        if let Err(err) = watcher.pong().await {
                            eprintln!("Failed to send pong: {err}");
                        }
                    }
                }
            }
        }
    })
}

pub enum Message {
    ConfigChanged(Changed),
    ConfigPing(Ping),
}
