// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub trait Executable {
    fn name(&self) -> &str;

    fn appid(&self) -> Option<&str> {
        None
    }

    fn exec(&self) -> &str;

    fn try_exec(&self) -> Option<&str> {
        None
    }

    fn is_terminal(&self) -> bool {
        false
    }

    fn path(&self) -> Option<&str> {
        None
    }

    fn working_directory(&self) -> Option<&str> {
        None
    }
}
