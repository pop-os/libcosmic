#[cfg(not(windows))]
pub use freedesktop_desktop_entry as fde;
#[cfg(not(windows))]
pub use mime::Mime;
use std::path::PathBuf;
#[cfg(not(windows))]
use std::{borrow::Cow, ffi::OsStr};

pub trait IconSourceExt {
    fn as_cosmic_icon(&self) -> crate::widget::icon::Icon;
}

#[cfg(not(windows))]
impl IconSourceExt for fde::IconSource {
    fn as_cosmic_icon(&self) -> crate::widget::icon::Icon {
        match self {
            fde::IconSource::Name(name) => crate::widget::icon::from_name(name.as_str())
                .size(128)
                .fallback(Some(crate::widget::icon::IconFallback::Names(vec![
                    "application-default".into(),
                    "application-x-executable".into(),
                ])))
                .into(),
            fde::IconSource::Path(path) => {
                crate::widget::icon(crate::widget::icon::from_path(path.clone()))
            }
        }
    }
}

#[cfg(not(windows))]
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopAction {
    pub name: String,
    pub exec: String,
}

#[cfg(not(windows))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopEntryData {
    pub id: String,
    pub name: String,
    pub wm_class: Option<String>,
    pub exec: Option<String>,
    pub icon: fde::IconSource,
    pub path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub desktop_actions: Vec<DesktopAction>,
    pub mime_types: Vec<Mime>,
    pub prefers_dgpu: bool,
    pub terminal: bool,
}

#[cfg(not(windows))]
pub fn load_applications<'a>(
    locales: &'a [String],
    include_no_display: bool,
    only_show_in: Option<&'a str>,
) -> impl Iterator<Item = DesktopEntryData> + 'a {
    fde::Iter::new(fde::default_paths())
        .filter_map(move |p| fde::DesktopEntry::from_path(p, Some(locales)).ok())
        .filter(move |de| {
            (include_no_display || !de.no_display())
                && !only_show_in.zip(de.only_show_in()).is_some_and(
                    |(xdg_current_desktop, only_show_in)| {
                        !only_show_in.contains(&xdg_current_desktop)
                    },
                )
        })
        .map(move |de| DesktopEntryData::from_desktop_entry(locales, de))
}

// Create an iterator which filters desktop entries by app IDs.
#[cfg(not(windows))]
#[auto_enums::auto_enum(Iterator)]
pub fn load_applications_for_app_ids<'a>(
    iter: impl Iterator<Item = fde::DesktopEntry> + 'a,
    locales: &'a [String],
    app_ids: Vec<&'a str>,
    fill_missing_ones: bool,
    include_no_display: bool,
    only_show_in: Option<&'a str>,
) -> impl Iterator<Item = DesktopEntryData> + 'a {
    let app_ids = std::rc::Rc::new(std::cell::RefCell::new(app_ids));
    let app_ids_ = app_ids.clone();

    let applications = iter
        .filter(move |de| {
            if !include_no_display && de.no_display() {
                return false;
            }
            if only_show_in.zip(de.only_show_in()).is_some_and(
                |(xdg_current_desktop, only_show_in)| !only_show_in.contains(&xdg_current_desktop),
            ) {
                return false;
            }

            // Search by ID first
            app_ids
                .borrow()
                .iter()
                .position(|id| de.matches_id(fde::unicase::Ascii::new(*id)))
                // Then fall back to search by name
                .or_else(|| {
                    app_ids
                        .borrow()
                        .iter()
                        .position(|id| de.matches_name(fde::unicase::Ascii::new(*id)))
                })
                // Remove the app ID if found
                .map(|i| {
                    app_ids.borrow_mut().remove(i);
                    true
                })
                .unwrap_or_default()
        })
        .map(move |de| DesktopEntryData::from_desktop_entry(locales, de));

    if fill_missing_ones {
        applications.chain(
            std::iter::once_with(move || {
                std::mem::take(&mut *app_ids_.borrow_mut())
                    .into_iter()
                    .map(|app_id| DesktopEntryData {
                        id: app_id.to_string(),
                        name: app_id.to_string(),
                        icon: fde::IconSource::default(),
                        ..Default::default()
                    })
            })
            .flatten(),
        )
    } else {
        applications
    }
}

#[cfg(not(windows))]
pub fn load_desktop_file(locales: &[String], path: PathBuf) -> Option<DesktopEntryData> {
    fde::DesktopEntry::from_path(path, Some(locales))
        .ok()
        .map(|de| DesktopEntryData::from_desktop_entry(locales, de))
}

#[cfg(not(windows))]
impl DesktopEntryData {
    pub fn from_desktop_entry(locales: &[String], de: fde::DesktopEntry) -> DesktopEntryData {
        let name = de
            .name(locales)
            .unwrap_or(Cow::Borrowed(&de.appid))
            .to_string();

        // check if absolute path exists and otherwise treat it as a name
        let icon = fde::IconSource::from_unknown(de.icon().unwrap_or(&de.appid));

        DesktopEntryData {
            id: de.appid.to_string(),
            wm_class: de.startup_wm_class().map(ToString::to_string),
            exec: de.exec().map(ToString::to_string),
            name,
            icon,
            categories: de
                .categories()
                .unwrap_or_default()
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
            desktop_actions: de
                .actions()
                .map(|actions| {
                    actions
                        .into_iter()
                        .filter_map(|action| {
                            let name = de.action_entry_localized(action, "Name", locales);
                            let exec = de.action_entry(action, "Exec");
                            if let (Some(name), Some(exec)) = (name, exec) {
                                Some(DesktopAction {
                                    name: name.to_string(),
                                    exec: exec.to_string(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            mime_types: de
                .mime_type()
                .map(|mime_types| {
                    mime_types
                        .into_iter()
                        .filter_map(|mime_type| mime_type.parse::<Mime>().ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            prefers_dgpu: de.prefers_non_default_gpu(),
            terminal: de.terminal(),
            path: Some(de.path),
        }
    }
}

#[cfg(not(windows))]
#[cold]
pub async fn spawn_desktop_exec<S, I, K, V>(
    exec: S,
    env_vars: I,
    app_id: Option<&str>,
    terminal: bool,
) where
    S: AsRef<str>,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let term_exec;

    let exec_str = if terminal {
        let term = cosmic_settings_config::shortcuts::context()
            .ok()
            .and_then(|config| {
                cosmic_settings_config::shortcuts::system_actions(&config)
                    .get(&cosmic_settings_config::shortcuts::action::System::Terminal)
                    .cloned()
            })
            .unwrap_or_else(|| String::from("cosmic-term"));

        term_exec = format!("{term} -- {}", exec.as_ref());
        &term_exec
    } else {
        exec.as_ref()
    };

    let mut exec = shlex::Shlex::new(exec_str);

    let executable = match exec.next() {
        Some(executable) if !executable.contains('=') => executable,
        _ => return,
    };

    let mut cmd = std::process::Command::new(&executable);

    for arg in exec {
        // TODO handle "%" args here if necessary?
        if !arg.starts_with('%') {
            cmd.arg(arg);
        }
    }

    cmd.envs(env_vars);

    // https://systemd.io/DESKTOP_ENVIRONMENTS
    //
    // Similar to what Gnome sets, for now.
    if let Some(pid) = crate::process::spawn(cmd).await {
        #[cfg(feature = "desktop-systemd-scope")]
        if let Ok(session) = zbus::Connection::session().await {
            if let Ok(systemd_manager) = SystemdMangerProxy::new(&session).await {
                let _ = systemd_manager
                    .start_transient_unit(
                        &format!("app-cosmic-{}-{}.scope", app_id.unwrap_or(&executable), pid),
                        "fail",
                        &[
                            (
                                "Description".to_string(),
                                zbus::zvariant::Value::from("Application launched by COSMIC")
                                    .try_to_owned()
                                    .unwrap(),
                            ),
                            (
                                "PIDs".to_string(),
                                zbus::zvariant::Value::from(vec![pid])
                                    .try_to_owned()
                                    .unwrap(),
                            ),
                            (
                                "CollectMode".to_string(),
                                zbus::zvariant::Value::from("inactive-or-failed")
                                    .try_to_owned()
                                    .unwrap(),
                            ),
                        ],
                        &[],
                    )
                    .await;
            }
        }
    }
}

#[cfg(not(windows))]
#[cfg(feature = "desktop-systemd-scope")]
#[zbus::proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManger {
    async fn start_transient_unit(
        &self,
        name: &str,
        mode: &str,
        properties: &[(String, zbus::zvariant::OwnedValue)],
        aux: &[(String, Vec<(String, zbus::zvariant::OwnedValue)>)],
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}
