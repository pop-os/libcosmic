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
