#[tokio::main]
async fn main() {
    let conn = zbus::Connection::session().await.unwrap();
    let env_vars: Vec<(String, String)> = std::env::vars().collect();
    if cosmic_launch::spawn(&conn, &CosmicEdit {}, env_vars, &["src/lib.rs".to_string()])
        .await
        .is_ok()
    {
        eprintln!("launched");
    }
}

struct CosmicEdit;
impl cosmic_launch::Executable for CosmicEdit {
    fn name(&self) -> &str {
        "com.system76.CosmicEdit"
    }

    fn appid(&self) -> Option<&str> {
        Some("com.system76.CosmicEdit")
    }

    fn exec(&self) -> &str {
        "cosmic-edit %F"
    }

    fn path(&self) -> Option<&str> {
        Some("/usr/share/applications/com.system76.CosmicEdit.desktop")
    }
}
