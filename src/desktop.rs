pub use freedesktop_desktop_entry::DesktopEntry;
pub use mime::Mime;
use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSource {
    Name(String),
    Path(PathBuf),
}

impl IconSource {
    pub fn from_unknown(icon: &str) -> Self {
        let icon_path = Path::new(icon);
        if icon_path.is_absolute() && icon_path.exists() {
            Self::Path(icon_path.into())
        } else {
            Self::Name(icon.into())
        }
    }

    pub fn as_cosmic_icon(&self) -> crate::widget::icon::Icon {
        match self {
            Self::Name(name) => crate::widget::icon::from_name(name.as_str())
                .size(128)
                .fallback(Some(crate::widget::icon::IconFallback::Names(vec![
                    "application-default".into(),
                    "application-x-executable".into(),
                ])))
                .into(),
            Self::Path(path) => crate::widget::icon(crate::widget::icon::from_path(path.clone())),
        }
    }
}

impl Default for IconSource {
    fn default() -> Self {
        Self::Name("application-default".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopAction {
    pub name: String,
    pub exec: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesktopEntryData {
    pub id: String,
    pub name: String,
    pub wm_class: Option<String>,
    pub exec: Option<String>,
    pub icon: IconSource,
    pub path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub desktop_actions: Vec<DesktopAction>,
    pub mime_types: Vec<Mime>,
    pub prefers_dgpu: bool,
}

pub fn load_applications<'a>(
    locale: impl Into<Option<&'a str>>,
    include_no_display: bool,
) -> Vec<DesktopEntryData> {
    load_applications_filtered(locale, |de| include_no_display || !de.no_display())
}

pub fn app_id_or_fallback_matches(app_id: &str, entry: &DesktopEntryData) -> bool {
    let lowercase_wm_class = match entry.wm_class.as_ref() {
        Some(s) => Some(s.to_lowercase()),
        None => None,
    };

    app_id == entry.id
        || Some(app_id.to_lowercase()) == lowercase_wm_class
        || app_id.to_lowercase() == entry.name.to_lowercase()
}

pub fn load_applications_for_app_ids<'a, 'b>(
    locale: impl Into<Option<&'a str>>,
    app_ids: impl Iterator<Item = &'b str>,
    fill_missing_ones: bool,
    include_no_display: bool,
) -> Vec<DesktopEntryData> {
    let mut app_ids = app_ids.collect::<Vec<_>>();
    let mut applications = load_applications_filtered(locale, |de| {
        if !include_no_display && de.no_display() {
            return false;
        }
        // If appid matches, or startup_wm_class matches...
        if let Some(i) = app_ids.iter().position(|id| {
            id == &de.appid
                || id
                    .to_lowercase()
                    .eq(&de.startup_wm_class().unwrap_or_default().to_lowercase())
        }) {
            app_ids.remove(i);
            true
        // Fallback: If the name matches...
        } else if let Some(i) = app_ids.iter().position(|id| {
            de.name(None)
                .map(|n| n.to_lowercase() == id.to_lowercase())
                .unwrap_or_default()
        }) {
            app_ids.remove(i);
            true
        } else {
            false
        }
    });
    if fill_missing_ones {
        applications.extend(app_ids.into_iter().map(|app_id| DesktopEntryData {
            id: app_id.to_string(),
            name: app_id.to_string(),
            icon: IconSource::default(),
            ..Default::default()
        }));
    }
    applications
}

pub fn load_applications_filtered<'a, F: FnMut(&DesktopEntry) -> bool>(
    locale: impl Into<Option<&'a str>>,
    mut filter: F,
) -> Vec<DesktopEntryData> {
    let locale = locale.into();

    freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
        .filter_map(|path| {
            std::fs::read_to_string(&path).ok().and_then(|input| {
                DesktopEntry::decode(&path, &input).ok().and_then(|de| {
                    if !filter(&de) {
                        return None;
                    }

                    Some(DesktopEntryData::from_desktop_entry(
                        locale,
                        path.clone(),
                        de,
                    ))
                })
            })
        })
        .collect()
}

pub fn load_desktop_file<'a>(
    locale: impl Into<Option<&'a str>>,
    path: impl AsRef<Path>,
) -> Option<DesktopEntryData> {
    let path = path.as_ref();
    std::fs::read_to_string(path).ok().and_then(|input| {
        DesktopEntry::decode(path, &input)
            .ok()
            .map(|de| DesktopEntryData::from_desktop_entry(locale, PathBuf::from(path), de))
    })
}

impl DesktopEntryData {
    fn from_desktop_entry<'a>(
        locale: impl Into<Option<&'a str>>,
        path: impl Into<Option<PathBuf>>,
        de: DesktopEntry,
    ) -> DesktopEntryData {
        let locale = locale.into();

        let name = de
            .name(locale)
            .unwrap_or(Cow::Borrowed(de.appid))
            .to_string();

        // check if absolute path exists and otherwise treat it as a name
        let icon = de.icon().unwrap_or(de.appid);
        let icon_path = Path::new(icon);
        let icon = if icon_path.is_absolute() && icon_path.exists() {
            IconSource::Path(icon_path.into())
        } else {
            IconSource::Name(icon.into())
        };

        DesktopEntryData {
            id: de.appid.to_string(),
            wm_class: de.startup_wm_class().map(ToString::to_string),
            exec: de.exec().map(ToString::to_string),
            name,
            icon,
            path: path.into(),
            categories: de
                .categories()
                .unwrap_or_default()
                .split_terminator(';')
                .map(std::string::ToString::to_string)
                .collect(),
            desktop_actions: de
                .actions()
                .map(|actions| {
                    actions
                        .split(';')
                        .filter_map(|action| {
                            let name = de.action_entry_localized(action, "Name", locale);
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
                        .split_terminator(';')
                        .filter_map(|mime_type| mime_type.parse::<Mime>().ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            prefers_dgpu: de.prefers_non_default_gpu(),
        }
    }
}

pub fn spawn_desktop_exec<S, I, K, V>(exec: S, env_vars: I)
where
    S: AsRef<str>,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut exec = shlex::Shlex::new(exec.as_ref());
    let mut cmd = match exec.next() {
        Some(cmd) if !cmd.contains('=') => std::process::Command::new(cmd),
        _ => return,
    };

    for arg in exec {
        // TODO handle "%" args here if necessary?
        if !arg.starts_with('%') {
            cmd.arg(arg);
        }
    }

    cmd.envs(env_vars);

    crate::process::spawn(cmd);
}
