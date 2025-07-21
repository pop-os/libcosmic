// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use {
    crate::ApplicationExt,
    iced::Subscription,
    iced_futures::futures::{
        SinkExt,
        channel::mpsc::{Receiver, Sender},
    },
    std::{any::TypeId, collections::HashMap},
    url::Url,
    zbus::{interface, proxy, zvariant::Value},
};

#[cold]
pub fn subscription<App: ApplicationExt>() -> Subscription<crate::Action<App::Message>> {
    use iced_futures::futures::StreamExt;
    iced_futures::Subscription::run_with_id(
        TypeId::of::<DbusActivation>(),
        iced::stream::channel(10, move |mut output| async move {
            let mut single_instance: DbusActivation = DbusActivation::new();
            let mut rx = single_instance.rx();
            if let Ok(builder) = zbus::connection::Builder::session() {
                let path: String = format!("/{}", App::APP_ID.replace('.', "/"));
                if let Ok(conn) = builder.build().await {
                    // XXX Setup done this way seems to be more reliable.
                    //
                    // the docs for serve_at seem to imply it will replace the
                    // existing interface at the requested path, but it doesn't
                    // seem to work that way all the time. The docs for
                    // object_server().at() imply it won't replace the existing
                    // interface.
                    //
                    // request_name is used either way, with the builder or
                    // with the connection, but it must be done after the
                    // object server is setup.
                    if conn.object_server().at(path, single_instance).await != Ok(true) {
                        tracing::error!("Failed to serve dbus");
                        std::process::exit(1);
                    }
                    if conn.request_name(App::APP_ID).await.is_err() {
                        tracing::error!("Failed to serve dbus");
                        std::process::exit(1);
                    }

                    output
                        .send(crate::Action::Cosmic(crate::app::Action::DbusConnection(
                            conn.clone(),
                        )))
                        .await;

                    #[cfg(feature = "smol")]
                    let handle = {
                        std::thread::spawn(move || {
                            let conn_clone = _conn.clone();

                            zbus::block_on(async move {
                                loop {
                                    conn_clone.executor().tick().await;
                                }
                            })
                        })
                    };
                    while let Some(mut msg) = rx.next().await {
                        if let Some(token) = msg.activation_token.take() {
                            if let Err(err) = output
                                .send(crate::Action::Cosmic(crate::app::Action::Activate(token)))
                                .await
                            {
                                tracing::error!(?err, "Failed to send message");
                            }
                        }
                        if let Err(err) = output.send(crate::Action::DbusActivation(msg)).await {
                            tracing::error!(?err, "Failed to send message");
                        }
                    }
                }
            } else {
                tracing::warn!("Failed to connect to dbus for single instance");
            }

            loop {
                iced::futures::pending!();
            }
        }),
    )
}

#[derive(Debug, Clone)]
pub struct Message<Action = String, Args = Vec<String>> {
    pub activation_token: Option<String>,
    pub desktop_startup_id: Option<String>,
    pub msg: Details<Action, Args>,
}

#[derive(Debug, Clone)]
pub enum Details<Action = String, Args = Vec<String>> {
    Activate,
    Open {
        url: Vec<Url>,
    },
    /// action can be deserialized as Flags
    ActivateAction {
        action: Action,
        args: Args,
    },
}

#[derive(Debug, Default)]
pub struct DbusActivation(Option<Sender<Message>>);

impl DbusActivation {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self(None)
    }

    #[inline]
    pub fn rx(&mut self) -> Receiver<Message> {
        let (tx, rx) = iced_futures::futures::channel::mpsc::channel(10);
        self.0 = Some(tx);
        rx
    }
}

#[proxy(interface = "org.freedesktop.DbusActivation", assume_defaults = true)]
pub trait DbusActivationInterface {
    /// Activate the application.
    fn activate(&mut self, platform_data: HashMap<&str, Value<'_>>) -> zbus::Result<()>;

    /// Open the given URIs.
    fn open(
        &mut self,
        uris: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;

    /// Activate the given action.
    fn activate_action(
        &mut self,
        action_name: &str,
        parameter: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;
}

#[interface(name = "org.freedesktop.DbusActivation")]
impl DbusActivation {
    #[cold]
    async fn activate(&mut self, platform_data: HashMap<&str, Value<'_>>) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(Message {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: Details::Activate,
                })
                .await;
        }
    }

    #[cold]
    async fn open(&mut self, uris: Vec<&str>, platform_data: HashMap<&str, Value<'_>>) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(Message {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: Details::Open {
                        url: uris.iter().filter_map(|u| Url::parse(u).ok()).collect(),
                    },
                })
                .await;
        }
    }

    #[cold]
    async fn activate_action(
        &mut self,
        action_name: &str,
        parameter: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(Message {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: Details::ActivateAction {
                        action: action_name.to_string(),
                        args: parameter
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect(),
                    },
                })
                .await;
        }
    }
}
