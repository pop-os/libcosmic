// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

error_set::error_set! {
    Error := {
        #[display("failed to initiate dbus connection to systemd manager")]
        ProxyConnect(zbus::Error),
        #[display("failed to start transient unit with systemd manager")]
        StartTransientUnit(zbus::Error),
        #[display("executable path must be UTF-8 compatible")]
        Utf8Path,
        #[display("failed to find executable")]
        Which(which::Error),
    }
}

pub async fn start_transient_unit<E: super::Executable>(
    conn: &zbus::Connection,
    executable: &E,
    env_vars: &[String],
    args: &[String],
) -> Result<(), Error> {
    let path = which::which(&*args[0]).map_err(Error::Which)?;
    let path = path.to_str().ok_or(Error::Utf8Path)?;

    let systemd_manager = SystemdMangerProxy::new(conn)
        .await
        .map_err(Error::ProxyConnect)?;

    let unit_name = unit_name(executable.appid(), &args[0]);
    let unit = transient_properties(executable, path, args, env_vars);
    systemd_manager
        .start_transient_unit(&unit_name, "fail", unit, Vec::new())
        .await
        .map_err(Error::StartTransientUnit)
        .map(|_| ())
}

/// Create the properties for a transient unit.
fn transient_properties<'a, E: super::Executable>(
    executable: &'a E,
    path: &'a str,
    args: &'a [String],
    env_vars: &'a [String],
) -> Vec<(&'a str, zbus::zvariant::Value<'a>)> {
    use zbus::zvariant::Value as Variant;
    let mut properties = Vec::with_capacity(8);
    properties.push(("ExecStart", Variant::from(vec![(path, args, false)])));
    properties.push(("Type", Variant::Str("exec".into())));
    properties.push(("CollectMode", Variant::Str("inactive-or-failed".into())));

    if !env_vars.is_empty() {
        properties.push(("Environment", Variant::from(env_vars)));
    }

    if let Some(path) = executable.working_directory() {
        properties.push(("WorkingDirectory", Variant::Str(path.into())));
    } else if let Ok(current_dir) = std::env::current_dir()
        && let Some(path) = current_dir.to_str()
    {
        properties.push(("WorkingDirectory", Variant::Str(path.to_owned().into())));
    }

    properties
}

fn unit_name(appid: Option<&str>, cmd: &str) -> String {
    use rand::distr::SampleString;
    let random = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 8);

    let appid = appid
        .unwrap_or_else(|| cmd.rsplit('/').next().unwrap_or(cmd))
        .replace("-", "_");

    ["app-", &appid, "@", &random, ".service"].concat()
}

use proxy::SystemdMangerProxy;
mod proxy {
    #[zbus::proxy(
        interface = "org.freedesktop.systemd1.Manager",
        default_service = "org.freedesktop.systemd1",
        default_path = "/org/freedesktop/systemd1"
    )]
    pub trait SystemdManger {
        async fn start_transient_unit(
            &self,
            name: &str,
            mode: &str,
            properties: Vec<(&str, zbus::zvariant::Value<'_>)>,
            aux: Vec<(&str, Vec<(&str, zbus::zvariant::Value<'_>)>)>,
        ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    }
}
