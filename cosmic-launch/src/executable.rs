// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use crate::gpu;

pub trait Executable {
    fn name(&self) -> Cow<'_, str>;

    fn appid(&self) -> Option<&str> {
        None
    }

    fn exec<'a>(&'a self, action: Option<&str>) -> &'a str;

    fn try_exec(&self) -> Option<&str> {
        None
    }

    fn is_terminal(&self) -> bool {
        false
    }

    fn gpu_preference(&self) -> gpu::Preference {
        gpu::Preference::Default
    }

    fn path(&self) -> Option<&str> {
        None
    }

    fn working_directory(&self) -> Option<&str> {
        None
    }
}

pub struct CustomExecutable {
    appid: String,
    exec: String,
    path: Option<String>,
    working_dir: Option<String>,
    terminal: bool,
}

impl CustomExecutable {
    pub fn new(appid: String, exec: String) -> Self {
        Self {
            appid,
            exec,
            path: None,
            working_dir: None,
            terminal: false,
        }
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn terminal(mut self, enable: bool) -> Self {
        self.terminal = enable;
        self
    }

    pub fn working_dir(mut self, path: String) -> Self {
        self.working_dir = Some(path);
        self
    }
}

impl Executable for CustomExecutable {
    fn exec<'a>(&'a self, _action: Option<&str>) -> &'a str {
        self.exec.as_str()
    }

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.appid)
    }

    fn appid(&self) -> Option<&str> {
        Some(&self.appid)
    }

    fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    fn working_directory(&self) -> Option<&str> {
        self.working_dir.as_deref()
    }
}
