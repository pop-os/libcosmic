#[cfg(not(windows))]
pub use freedesktop_desktop_entry as fde;
#[cfg(not(windows))]
pub use mime::Mime;
use std::path::{Path, PathBuf};
#[cfg(not(windows))]
use std::{borrow::Cow, collections::HashSet, ffi::OsStr};

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
#[derive(Debug, Clone)]
pub struct DesktopEntryCache {
    locales: Vec<String>,
    entries: Vec<fde::DesktopEntry>,
}

#[cfg(not(windows))]
impl DesktopEntryCache {
    pub fn new(locales: Vec<String>) -> Self {
        Self {
            locales,
            entries: Vec::new(),
        }
    }

    pub fn from_entries(locales: Vec<String>, entries: Vec<fde::DesktopEntry>) -> Self {
        Self { locales, entries }
    }

    pub fn ensure_loaded(&mut self) {
        if self.entries.is_empty() {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.entries = fde::Iter::new(fde::default_paths())
            .filter_map(|p| fde::DesktopEntry::from_path(p, Some(&self.locales)).ok())
            .collect();
    }

    pub fn insert(&mut self, entry: fde::DesktopEntry) {
        if self
            .entries
            .iter()
            .any(|existing| existing.id() == entry.id())
        {
            return;
        }

        self.entries.push(entry);
    }

    pub fn locales(&self) -> &[String] {
        &self.locales
    }

    pub fn entries(&self) -> &[fde::DesktopEntry] {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut [fde::DesktopEntry] {
        &mut self.entries
    }
}

#[cfg(not(windows))]
impl Default for DesktopEntryCache {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(not(windows))]
#[derive(Debug, Clone)]
pub struct DesktopLookupContext<'a> {
    pub app_id: Cow<'a, str>,
    pub identifier: Option<Cow<'a, str>>,
    pub title: Option<Cow<'a, str>>,
}

#[cfg(not(windows))]
impl<'a> DesktopLookupContext<'a> {
    pub fn new(app_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            app_id: app_id.into(),
            identifier: None,
            title: None,
        }
    }

    pub fn with_identifier(mut self, identifier: impl Into<Cow<'a, str>>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[cfg(not(windows))]
#[derive(Debug, Clone)]
pub struct DesktopResolveOptions {
    pub include_no_display: bool,
    pub xdg_current_desktop: Option<String>,
}

#[cfg(not(windows))]
impl Default for DesktopResolveOptions {
    fn default() -> Self {
        Self {
            include_no_display: false,
            xdg_current_desktop: std::env::var("XDG_CURRENT_DESKTOP").ok(),
        }
    }
}

#[cfg(not(windows))]
/// Resolve a DesktopEntry for a running toplevel, applying heuristics over
/// app_id, identifier, and title. Includes Proton/Wine handling: Proton can
/// open games as `steam_app_X` (often `steam_app_default`), and Wine windows
/// may use an `.exe` app_id. In those cases we match the localized name
/// against the toplevel title and, for Proton default, restrict matches to
/// entries with `Game` in Categories.
pub fn resolve_desktop_entry(
    cache: &mut DesktopEntryCache,
    context: &DesktopLookupContext<'_>,
    options: &DesktopResolveOptions,
) -> fde::DesktopEntry {
    let app_id = fde::unicase::Ascii::new(context.app_id.as_ref());

    if let Some(entry) = fde::find_app_by_id(cache.entries(), app_id) {
        return entry.clone();
    }

    cache.refresh();
    if let Some(entry) = fde::find_app_by_id(cache.entries(), app_id) {
        return entry.clone();
    }

    let candidate_ids = candidate_desktop_ids(context);

    if let Some(entry) = try_match_cached(cache.entries(), &candidate_ids) {
        return entry;
    }

    if let Some(entry) = load_entry_via_app_ids(
        cache,
        &candidate_ids,
        options.include_no_display,
        options.xdg_current_desktop.as_deref(),
    ) {
        cache.insert(entry.clone());
        return entry;
    }

    if let Some(entry) = match_startup_wm_class(cache.entries(), context) {
        return entry;
    }

    // Chromium/CRX heuristic: scan exec/wmclass/icon for a CRX id match.
    if let Some(entry) = match_crx_id(cache.entries(), context) {
        return entry;
    }

    if let Some(entry) = match_exec_basename(cache.entries(), &candidate_ids) {
        return entry;
    }

    if let Some(entry) = proton_or_wine_fallback(cache, context) {
        cache.insert(entry.clone());
        entry
    } else {
        let fallback = fallback_entry(context);
        cache.insert(fallback.clone());
        fallback
    }
}

#[cfg(not(windows))]
fn try_match_cached(
    entries: &[fde::DesktopEntry],
    candidate_ids: &[String],
) -> Option<fde::DesktopEntry> {
    candidate_ids.iter().find_map(|candidate| {
        fde::find_app_by_id(entries, fde::unicase::Ascii::new(candidate.as_str())).cloned()
    })
}

#[cfg(not(windows))]
fn load_entry_via_app_ids(
    cache: &DesktopEntryCache,
    candidate_ids: &[String],
    include_no_display: bool,
    xdg_current_desktop: Option<&str>,
) -> Option<fde::DesktopEntry> {
    if candidate_ids.is_empty() {
        return None;
    }

    let candidate_refs: Vec<&str> = candidate_ids.iter().map(String::as_str).collect();
    let locales = cache.locales().to_vec();
    let iter_locales = locales.clone();

    let desktop_iter = fde::Iter::new(fde::default_paths())
        .filter_map(move |path| fde::DesktopEntry::from_path(path, Some(&iter_locales)).ok());

    let app_iter = load_applications_for_app_ids(
        desktop_iter,
        &locales,
        candidate_refs,
        false,
        include_no_display,
        xdg_current_desktop,
    );

    let locales_for_load = cache.locales().to_vec();
    for app in app_iter {
        if let Some(path) = app.path {
            if let Ok(entry) = fde::DesktopEntry::from_path(path, Some(&locales_for_load)) {
                return Some(entry);
            }
        }
    }

    None
}

#[cfg(not(windows))]
fn match_startup_wm_class(
    entries: &[fde::DesktopEntry],
    context: &DesktopLookupContext<'_>,
) -> Option<fde::DesktopEntry> {
    let mut candidates = Vec::new();
    candidates.push(context.app_id.as_ref());
    if let Some(identifier) = context.identifier.as_deref() {
        candidates.push(identifier);
    }
    if let Some(title) = context.title.as_deref() {
        candidates.push(title);
    }

    for entry in entries {
        let Some(wm_class) = entry.startup_wm_class() else {
            continue;
        };

        if candidates
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(wm_class))
        {
            return Some(entry.clone());
        }
    }

    None
}

#[cfg(not(windows))]
fn is_crx_id(candidate: &str) -> bool {
    is_crx_bytes(candidate.as_bytes())
}

#[cfg(not(windows))]
fn is_crx_bytes(bytes: &[u8]) -> bool {
    bytes.len() == 32 && bytes.iter().all(|b| matches!(b, b'a'..=b'p'))
}

#[cfg(not(windows))]
pub fn extract_crx_id(value: &str) -> Option<String> {
    if let Some(rest) = value.strip_prefix("chrome-") {
        if let Some(first) = rest.split(&['-', '_'][..]).next() {
            if is_crx_id(first) {
                return Some(first.to_string());
            }
        }
    }
    if let Some(rest) = value.strip_prefix("crx_") {
        let token = rest
            .split(|c: char| !c.is_ascii_lowercase())
            .next()
            .unwrap_or(rest);
        if is_crx_id(token) {
            return Some(token.to_string());
        }
    }
    if is_crx_id(value) {
        return Some(value.to_string());
    }

    for window in value.as_bytes().windows(32) {
        if is_crx_bytes(window) {
            // SAFETY: `is_crx_bytes` guarantees the window is ASCII.
            let slice = std::str::from_utf8(window).expect("ASCII window");
            return Some(slice.to_string());
        }
    }

    None
}

#[cfg(not(windows))]
fn match_crx_id(
    entries: &[fde::DesktopEntry],
    context: &DesktopLookupContext<'_>,
) -> Option<fde::DesktopEntry> {
    let crx = extract_crx_id(context.app_id.as_ref())
        .or_else(|| context.identifier.as_deref().and_then(extract_crx_id))?;

    for entry in entries {
        if let Some(exec) = entry.exec() {
            if exec.contains(&format!("--app-id={}", crx)) {
                return Some(entry.clone());
            }
        }
        if let Some(wm) = entry.startup_wm_class() {
            if wm.eq_ignore_ascii_case(&format!("crx_{}", crx)) {
                return Some(entry.clone());
            }
        }
        if let Some(icon) = entry.icon() {
            if icon.contains(&crx) {
                return Some(entry.clone());
            }
        }
    }

    None
}

#[cfg(not(windows))]
fn match_exec_basename(
    entries: &[fde::DesktopEntry],
    candidate_ids: &[String],
) -> Option<fde::DesktopEntry> {
    fn normalize_candidate(candidate: &str) -> String {
        candidate
            .trim_matches(|c: char| c == '"' || c == '\'')
            .to_ascii_lowercase()
    }

    let mut normalized: Vec<String> = candidate_ids
        .iter()
        .map(|c| normalize_candidate(c))
        .collect();
    normalized.retain(|c| !c.is_empty());

    for entry in entries {
        let Some(exec) = entry.exec() else {
            continue;
        };

        let command = exec
            .split_whitespace()
            .next()
            .map(|token| token.trim_matches(|c: char| c == '"' || c == '\''))
            .filter(|token| !token.is_empty());

        let Some(command) = command else {
            continue;
        };

        let command = Path::new(command);
        let basename = command
            .file_stem()
            .or_else(|| command.file_name())
            .and_then(|s| s.to_str());

        let Some(basename) = basename else {
            continue;
        };

        let basename_lower = basename.to_ascii_lowercase();

        if normalized
            .iter()
            .any(|candidate| candidate == &basename_lower)
        {
            return Some(entry.clone());
        }
    }

    None
}

#[cfg(not(windows))]
fn fallback_entry(context: &DesktopLookupContext<'_>) -> fde::DesktopEntry {
    let mut entry = fde::DesktopEntry {
        appid: context.app_id.to_string(),
        groups: Default::default(),
        path: Default::default(),
        ubuntu_gettext_domain: None,
    };

    let name = context
        .title
        .as_ref()
        .map(|title| title.to_string())
        .unwrap_or_else(|| context.app_id.to_string());
    entry.add_desktop_entry("Name".to_string(), name);
    entry
}

#[cfg(not(windows))]
// proton opens games as steam_app_X, where X is either the steam appid or
// "default". Games with a steam appid can have a desktop entry generated
// elsewhere; this specifically handles non-steam games opened under Proton.
// In addition, try to match WINE entries whose app_id is the full name of
// the executable (including `.exe`).
fn proton_or_wine_fallback(
    cache: &DesktopEntryCache,
    context: &DesktopLookupContext<'_>,
) -> Option<fde::DesktopEntry> {
    let app_id = context.app_id.as_ref();
    let is_proton_game = app_id == "steam_app_default";
    let is_wine_entry = app_id.ends_with(".exe");

    if !is_proton_game && !is_wine_entry {
        return None;
    }

    let title = context.title.as_deref()?;

    for entry in cache.entries() {
        let localized_name_matches = entry
            .name(cache.locales())
            .is_some_and(|name| name == title);

        if !localized_name_matches {
            continue;
        }

        if is_proton_game && !entry.categories().unwrap_or_default().contains(&"Game") {
            continue;
        }

        return Some(entry.clone());
    }

    None
}

#[cfg(not(windows))]
fn candidate_desktop_ids(context: &DesktopLookupContext<'_>) -> Vec<String> {
    const SUFFIXES: &[&str] = &[".desktop", ".Desktop", ".DESKTOP"];
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    fn push_candidate(seen: &mut HashSet<String>, ordered: &mut Vec<String>, candidate: &str) {
        let trimmed = candidate.trim();
        if trimmed.is_empty() {
            return;
        }

        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            ordered.push(trimmed.to_string());
        }
    }

    fn add_variants(
        seen: &mut HashSet<String>,
        ordered: &mut Vec<String>,
        value: Option<&str>,
        suffixes: &[&str],
    ) {
        let Some(value) = value else {
            return;
        };

        let stripped_quotes = value.trim_matches(|c: char| c == '"' || c == '\'');
        let trimmed = stripped_quotes.trim();
        if trimmed.is_empty() {
            return;
        }

        push_candidate(seen, ordered, trimmed);
        if stripped_quotes != trimmed {
            push_candidate(seen, ordered, stripped_quotes.trim());
        }

        for suffix in suffixes {
            if trimmed.ends_with(suffix) {
                let cut = &trimmed[..trimmed.len() - suffix.len()];
                push_candidate(seen, ordered, cut);
            }
        }

        if trimmed.contains('.') {
            if let Some(last) = trimmed.rsplit('.').next() {
                if last.len() >= 2 {
                    push_candidate(seen, ordered, last);
                }
            }
        }

        if trimmed.contains('-') {
            push_candidate(seen, ordered, &trimmed.replace('-', "_"));
        }
        if trimmed.contains('_') {
            push_candidate(seen, ordered, &trimmed.replace('_', "-"));
        }

        for token in trimmed.split(|c: char| matches!(c, '.' | '-' | '_' | '@' | ' ')) {
            if token.len() >= 2 && token != trimmed {
                push_candidate(seen, ordered, token);
            }
        }
    }

    add_variants(
        &mut seen,
        &mut ordered,
        Some(context.app_id.as_ref()),
        SUFFIXES,
    );
    add_variants(
        &mut seen,
        &mut ordered,
        context.identifier.as_deref(),
        SUFFIXES,
    );
    add_variants(&mut seen, &mut ordered, context.title.as_deref(), &[]);

    // Chromium/Chrome PWA heuristics: favorites may store a short id like
    // "chrome-<crx>-Default" while the actual desktop id is
    // "org.chromium.Chromium.flextop.chrome-<crx>-Default" (Flatpak Chromium)
    // or sometimes "org.chromium.Chromium.chrome-<crx>-Default". Expand those
    // candidates so we can match cached entries.
    if let Some(app_id) = Some(context.app_id.as_ref()) {
        if let Some(rest) = app_id.strip_prefix("chrome-") {
            if rest.ends_with("-Default") {
                let crx = rest.trim_end_matches("-Default");
                let variants = [
                    format!("org.chromium.Chromium.flextop.chrome-{}-Default", crx),
                    format!("org.chromium.Chromium.chrome-{}-Default", crx),
                ];
                for v in variants {
                    push_candidate(&mut seen, &mut ordered, &v);
                }
            }
        }
        if let Some(rest) = app_id.strip_prefix("crx_") {
            // Older identifiers may be crx_<id>; expand similarly
            let crx = rest;
            let variants = [
                format!("org.chromium.Chromium.flextop.chrome-{}-Default", crx),
                format!("org.chromium.Chromium.chrome-{}-Default", crx),
            ];
            for v in variants {
                push_candidate(&mut seen, &mut ordered, &v);
            }
        }
    }

    ordered
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
                && only_show_in.zip(de.only_show_in()).is_none_or(
                    |(xdg_current_desktop, only_show_in)| {
                        only_show_in.contains(&xdg_current_desktop)
                    },
                )
                && only_show_in.zip(de.not_show_in()).is_none_or(
                    |(xdg_current_desktop, not_show_in)| {
                        !not_show_in.contains(&xdg_current_desktop)
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
            if only_show_in.zip(de.not_show_in()).is_some_and(
                |(xdg_current_desktop, not_show_in)| not_show_in.contains(&xdg_current_desktop),
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

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;
    use std::{env, fs, path::Path, path::PathBuf};
    use tempfile::tempdir;

    struct EnvVarGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let original = env::var(key).ok();
            // std::env::{set_var, remove_var} are unsafe on newer toolchains;
            // we limit scope here to the test helper that toggles a single key.
            unsafe { std::env::set_var(key, value) };
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(ref original) = self.original {
                unsafe { std::env::set_var(self.key, original) };
            } else {
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }

    fn load_entry(file_name: &str, contents: &str, locales: &[String]) -> fde::DesktopEntry {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join(file_name);
        fs::write(&path, contents).expect("write desktop file");
        let entry = fde::DesktopEntry::from_path(path, Some(locales)).expect("load desktop file");
        // Ensure directory stays alive until after parsing
        temp.close().expect("close tempdir");
        entry
    }

    #[test]
    fn candidate_generation_covers_common_variants() {
        let ctx = DesktopLookupContext::new("com.example.App.desktop")
            .with_identifier("com-example-App")
            .with_title("Example App");
        let candidates = candidate_desktop_ids(&ctx);

        assert_eq!(candidates.first().unwrap(), "com.example.App.desktop");
        assert!(candidates.contains(&"com.example.App".to_string()));
        assert!(candidates.contains(&"com-example-App".to_string()));
        assert!(candidates.contains(&"com_example_App".to_string()));
        assert!(candidates.contains(&"Example App".to_string()));
        assert!(candidates.contains(&"Example".to_string()));
        assert!(candidates.contains(&"App".to_string()));
    }

    #[test]
    fn startup_wm_class_matching_detects_flatpak_chrome_apps() {
        let temp = tempdir().expect("tempdir");
        let apps_dir = temp.path().join("applications");
        fs::create_dir_all(&apps_dir).expect("create applications dir");

        let desktop_contents = "\
[Desktop Entry]
Version=1.0
Type=Application
Name=Proton Mail
Exec=chromium --app-id=jnpecgipniidlgicjocehkhajgdnjekh
Icon=chrome-jnpecgipniidlgicjocehkhajgdnjekh-Default
StartupWMClass=crx_jnpecgipniidlgicjocehkhajgdnjekh
";
        let desktop_path = apps_dir.join(
            "org.chromium.Chromium.flextop.chrome-jnpecgipniidlgicjocehkhajgdnjekh-Default.desktop",
        );
        fs::write(desktop_path, desktop_contents).expect("write desktop file");

        let _guard = EnvVarGuard::set("XDG_DATA_HOME", temp.path());

        let locales = vec!["en_US.UTF-8".to_string()];
        let mut cache = DesktopEntryCache::new(locales.clone());
        cache.refresh();

        let ctx = DesktopLookupContext::new("crx_jnpecgipniidlgicjocehkhajgdnjekh");
        let resolved = resolve_desktop_entry(&mut cache, &ctx, &DesktopResolveOptions::default());

        assert_eq!(
            resolved.id(),
            "org.chromium.Chromium.flextop.chrome-jnpecgipniidlgicjocehkhajgdnjekh-Default"
        );
    }

    #[test]
    fn exec_basename_matching_handles_vmware() {
        let temp = tempdir().expect("tempdir");
        let apps_dir = temp.path().join("applications");
        fs::create_dir_all(&apps_dir).expect("create applications dir");

        let desktop_contents = "\
[Desktop Entry]\n\
Version=1.0\n\
Type=Application\n\
Name=VMware Workstation\n\
Exec=/usr/bin/vmware %U\n\
Icon=vmware-workstation\n\
";
        let desktop_path = apps_dir.join("vmware-workstation.desktop");
        fs::write(desktop_path, desktop_contents).expect("write desktop file");

        let _guard = EnvVarGuard::set("XDG_DATA_HOME", temp.path());

        let locales = vec!["en_US.UTF-8".to_string()];
        let mut cache = DesktopEntryCache::new(locales.clone());
        cache.refresh();

        let ctx = DesktopLookupContext::new("vmware").with_title("Library — VMware Workstation");

        let resolved = resolve_desktop_entry(&mut cache, &ctx, &DesktopResolveOptions::default());

        assert_eq!(resolved.id(), "vmware-workstation.desktop");
    }

    #[test]
    fn proton_fallback_prefers_game_entries() {
        let locales = vec!["en_US.UTF-8".to_string()];
        let entry = load_entry(
            "proton.desktop",
            "[Desktop Entry]\nType=Application\nName=Proton Game\nCategories=Game;Utility;\nExec=proton-game\n",
            &locales,
        );
        let cache = DesktopEntryCache::from_entries(locales.clone(), vec![entry]);
        let ctx = DesktopLookupContext::new("steam_app_default").with_title("Proton Game");

        let resolved = proton_or_wine_fallback(&cache, &ctx).expect("expected proton match");
        let name = resolved
            .name(&locales)
            .expect("name available")
            .into_owned();

        assert_eq!(name, "Proton Game");
    }

    #[test]
    fn proton_fallback_skips_non_games() {
        let locales = vec!["en_US.UTF-8".to_string()];
        let entry = load_entry(
            "tool.desktop",
            "[Desktop Entry]\nType=Application\nName=Proton Tool\nCategories=Utility;\nExec=proton-tool\n",
            &locales,
        );
        let cache = DesktopEntryCache::from_entries(locales, vec![entry]);
        let ctx = DesktopLookupContext::new("steam_app_default").with_title("Proton Tool");

        assert!(proton_or_wine_fallback(&cache, &ctx).is_none());
    }

    #[test]
    fn wine_fallback_matches_executable_titles() {
        let locales = vec!["en_US.UTF-8".to_string()];
        let entry = load_entry(
            "wine.desktop",
            "[Desktop Entry]\nType=Application\nName=Wine Game\nExec=wine-game\n",
            &locales,
        );
        let cache = DesktopEntryCache::from_entries(locales.clone(), vec![entry]);
        let ctx = DesktopLookupContext::new("WINEGAME.EXE").with_title("Wine Game");

        let resolved = proton_or_wine_fallback(&cache, &ctx).expect("expected wine match");
        let name = resolved
            .name(&locales)
            .expect("name available")
            .into_owned();
        assert_eq!(name, "Wine Game");
    }

    #[test]
    fn fallback_entry_uses_title_when_available() {
        let ctx = DesktopLookupContext::new("unknown-app").with_title("Unknown App");
        let entry = fallback_entry(&ctx);

        assert_eq!(entry.id(), "unknown-app");
        assert_eq!(
            entry.name(&["en_US".to_string()]),
            Some(Cow::Owned("Unknown App".to_string()))
        );
    }

    #[test]
    fn desktop_entry_data_prefers_localized_name() {
        let locales = vec!["fr".to_string(), "en_US".to_string()];
        let entry = load_entry(
            "localized.desktop",
            "[Desktop Entry]\nType=Application\nName=Default\nName[fr]=Localisé\nExec=localized\n",
            &locales,
        );
        let data = DesktopEntryData::from_desktop_entry(&locales, entry);

        assert_eq!(data.name, "Localisé");
    }

    #[test]
    fn crx_id_extraction_variants() {
        let id = "cadlkienfkclaiaibeoongdcgmdikeeg"; // 32 chars a..p
        assert_eq!(
            super::extract_crx_id(&format!("chrome-{}-Default", id)),
            Some(id.to_string())
        );
        assert_eq!(
            super::extract_crx_id(&format!("crx_{}", id)),
            Some(id.to_string())
        );
        assert_eq!(super::extract_crx_id(id), Some(id.to_string()));
        // Embedded
        let embedded = format!("org.chromium.Chromium.flextop.chrome-{}-Default", id);
        assert_eq!(super::extract_crx_id(&embedded), Some(id.to_string()));
    }

    #[test]
    fn crx_matcher_by_exec_and_wmclass() {
        use std::fs;
        let id = "cadlkienfkclaiaibeoongdcgmdikeeg";
        let temp = tempdir().expect("tempdir");
        let apps_dir = temp.path().join("applications");
        fs::create_dir_all(&apps_dir).expect("create applications dir");
        let desktop_contents = format!(
            "[Desktop Entry]\nType=Application\nName=ChatGPT\nExec=chromium --app-id={} --profile-directory=Default\nStartupWMClass=crx_{}\nIcon=chrome-{}-Default\n",
            id, id, id
        );
        let desktop_path = apps_dir.join(
            "org.chromium.Chromium.flextop.chrome-cadlkienfkclaiaibeoongdcgmdikeeg-Default.desktop",
        );
        fs::write(&desktop_path, desktop_contents).expect("write desktop file");

        let _guard = EnvVarGuard::set("XDG_DATA_HOME", temp.path());
        let locales = vec!["en_US.UTF-8".to_string()];
        let mut cache = DesktopEntryCache::new(locales.clone());
        cache.refresh();

        let short_id = format!("chrome-{}-Default", id);
        let ctx = DesktopLookupContext::new(short_id);
        let resolved = resolve_desktop_entry(&mut cache, &ctx, &DesktopResolveOptions::default());
        assert!(resolved.icon().is_some());
        assert!(resolved.exec().is_some());
        let expected = format!("crx_{}", id);
        assert_eq!(resolved.startup_wm_class(), Some(expected.as_str()));
    }

    #[test]
    fn crx_extraction_handles_utf8_prefixes() {
        let id = "cadlkienfkclaiaibeoongdcgmdikeeg";
        let prefixed = format!("å{}", id);
        assert_eq!(super::extract_crx_id(&prefixed), Some(id.to_string()));
    }

    #[test]
    fn crx_extraction_ignores_non_ascii_sequences() {
        let id = "cadlkienfkclaiaibeoongdcgmdikeeg";
        let embedded = format!("{id}æøå");

        assert_eq!(super::extract_crx_id(&embedded), Some(id.to_string()));
        assert_eq!(super::extract_crx_id("æøå"), None);
    }
}
