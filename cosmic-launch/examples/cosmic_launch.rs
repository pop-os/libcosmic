use std::borrow::Cow;

#[tokio::main]
async fn main() {
    let (session_conn, system_conn) =
        tokio::try_join!(zbus::Connection::session(), zbus::Connection::system()).unwrap();

    let gpus = match cosmic_launch::gpu::enumerate(&system_conn).await {
        Some(gpus) => gpus.collect(),
        _ => Vec::new(),
    };

    if cosmic_launch::spawn(
        &session_conn,
        &CosmicEdit {},
        None,
        &gpus,
        None,
        Vec::new(),
        &["src/lib.rs".to_string()],
    )
    .await
    .is_ok()
    {
        eprintln!("launched");
    }
}

struct CosmicEdit;
impl cosmic_launch::Executable for CosmicEdit {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("com.system76.CosmicEdit")
    }

    fn appid(&self) -> Option<&str> {
        Some("com.system76.CosmicEdit")
    }

    fn exec<'a>(&'a self, _action: Option<&str>) -> &'a str {
        "cosmic-edit %F"
    }

    fn path(&self) -> Option<&str> {
        Some("/usr/share/applications/com.system76.CosmicEdit.desktop")
    }
}
