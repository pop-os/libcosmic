// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Executable;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EntryLauncher {
    pub entry: Arc<freedesktop_desktop_entry::DesktopEntry>,
    pub locales: Arc<[String]>,
}

impl EntryLauncher {
    pub fn new(
        entry: Arc<freedesktop_desktop_entry::DesktopEntry>,
        locales: Arc<[String]>,
    ) -> Self {
        Self { entry, locales }
    }
}

impl Executable for EntryLauncher {
    fn name(&self) -> Cow<'_, str> {
        self.entry
            .desktop_entry_localized("Name", &self.locales)
            .unwrap_or(Cow::Borrowed(self.entry.appid.as_str()))
    }

    fn appid(&self) -> Option<&str> {
        Some(&self.entry.appid)
    }

    fn exec<'a>(&'a self, action: Option<&str>) -> &'a str {
        action
            .map_or_else(
                || self.entry.exec(),
                |action| self.entry.action_exec(action),
            )
            .unwrap_or_default()
    }

    fn path(&self) -> Option<&str> {
        self.entry.path.as_path().to_str()
    }

    fn working_directory(&self) -> Option<&str> {
        self.entry.path()
    }

    fn is_terminal(&self) -> bool {
        self.entry.terminal()
    }
}
