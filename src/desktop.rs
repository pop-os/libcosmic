pub use freedesktop_desktop_entry::DesktopEntry;
use iced_widget::canvas::path::lyon_path::geom::euclid::approxord::min;
pub use mime::Mime;
use std::{
    borrow::Cow,
    cmp::max,
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

/// From 0 to 1.
/// 1 is a perfect match.
fn match_entry(id: &str, de: &DesktopEntry) -> f32 {
    let cmp = |id, de| {
        let lcsstr = textdistance::str::lcsstr(id, de);
        lcsstr as f32 / (max(id.len(), de.len())) as f32
    };

    fn max_f32(a: f32, b: f32) -> f32 {
        if a > b {
            return a;
        } else {
            b
        }
    }

    let id = id.to_lowercase();
    let de_id = de.appid.to_lowercase();
    let de_wm_class = de.startup_wm_class().unwrap_or_default().to_lowercase();
    let de_name = de.name(None).unwrap_or_default().to_lowercase();

    return max_f32(
        cmp(&id, &de_id),
        max_f32(cmp(&id, &de_wm_class), cmp(&id, &de_name)),
    );
}

/// The result will be in the same order of `app_ids`
pub fn load_applications_for_app_ids<'a, I: AsRef<str>>(
    locale: impl Into<Option<&'a str>>,
    app_ids: impl Iterator<Item = I>,
    fill_missing_ones: bool,
    include_no_display: bool,
) -> Vec<DesktopEntryData> {
    // need to be owned
    let all_desktop_entries_string =
        freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
            .filter_map(|path| {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            })
            .collect::<Vec<_>>();

    let all_desktop_entries = all_desktop_entries_string
        .iter()
        .filter_map(|(path, content)| DesktopEntry::decode(&path, &content).ok())
        .collect::<Vec<_>>();

    let mut applications = Vec::new();

    let locale = locale.into();

    for id in app_ids {
        let mut max_score = None;
        let mut second_max_score = 0.;

        for de in &all_desktop_entries {
            if !include_no_display && de.no_display() {
                continue;
            }

            let score = match_entry(id.as_ref(), de);

            match max_score {
                Some((prev_max_score, _)) => {
                    if prev_max_score < score {
                        second_max_score = prev_max_score;
                        max_score = Some((score, de));
                    }
                }
                None => {
                    max_score = Some((score, de));
                }
            }

            if score > 0.99 {
                break;
            }
        }

        let mut has_matched = false;
        if let Some((max_score, de)) = max_score {
            let entropy = max_score - second_max_score;

            // println!("{id}: {max_score}:{entropy}: {}", de.appid);

            if max_score > 0.7 || entropy > 0.15 && max_score > 0.2 {
                applications.push(DesktopEntryData::from_desktop_entry(
                    locale,
                    Some(de.path.to_path_buf()),
                    de,
                ));
                has_matched = true;
            }
        }

        if fill_missing_ones && !has_matched {
            applications.push(DesktopEntryData {
                id: id.as_ref().to_string(),
                name: id.as_ref().to_string(),
                icon: IconSource::default(),
                ..Default::default()
            });
        }
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
                        &de,
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
            .map(|de| DesktopEntryData::from_desktop_entry(locale, PathBuf::from(path), &de))
    })
}

impl DesktopEntryData {
    fn from_desktop_entry<'a>(
        locale: impl Into<Option<&'a str>>,
        path: impl Into<Option<PathBuf>>,
        de: &DesktopEntry,
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
