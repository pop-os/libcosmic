use std::error::Error;
use std::time::Duration;
use std::{collections::HashMap, convert::Infallible};

use cosmic::{app::Message, Task};
use zbus::{export::futures_util::StreamExt, proxy, zvariant::Value, Connection};

#[proxy(
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    /// Call the org.freedesktop.Notifications.Notify D-Bus method
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: HashMap<&str, &Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;

    // We need to define the signal we want to listen for
    #[zbus(signal)]
    fn action_invoked(&self, id: u32, action_key: String) -> zbus::Result<()>;

    // We need to define the signal we want to listen for
    #[zbus(signal)]
    fn activation_token(&self, id: u32, activation_token: String) -> zbus::Result<()>;
}

pub fn notif_task<M: std::marker::Send + 'static>() -> Task<cosmic::app::Message<M>> {
    Task::future(async {
        match notif().await {
            Ok(t) => t,
            Err(err) => {
                tracing::error!("Ooops: {err:?}");
                panic!("oops");
            }
        }
    })
}

pub async fn notif<M: std::marker::Send + 'static>(
) -> Result<cosmic::app::Message<M>, Box<dyn Error>> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    let connection = Connection::session().await?;

    let proxy = NotificationsProxy::new(&connection).await?;
    let mut actions = proxy.receive_action_invoked().await?;
    let mut tokens = proxy.receive_activation_token().await?;

    let _reply = proxy
        .notify(
            "my-app",
            0,
            "dialog-information",
            "A summary",
            "Some body",
            &["default", "asdf"],
            HashMap::new(),
            5000,
        )
        .await?;

    let token = if let Some(msg) = tokens.next().await {
        // struct `JobNewArgs` is generated from `job_new` signal function arguments
        let args: ActivationTokenArgs = msg.args().expect("Error parsing message");

        println!("action: id={} action={}", args.id, args.activation_token,);
        args.activation_token
    } else {
        eprintln!("no token");
        panic!("oops");
    };
    if let Some(msg) = actions.next().await {
        // struct `JobNewArgs` is generated from `job_new` signal function arguments
        let args: ActionInvokedArgs = msg.args().expect("Error parsing message");

        println!("action: id={} action={}", args.id, args.action_key,);
    }

    Ok(Message::Cosmic(cosmic::app::cosmic::Message::Activate(
        token,
    )))
}
