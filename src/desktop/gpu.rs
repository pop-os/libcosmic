// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

/// An available GPU that can be selected by a launcher.
#[derive(Debug, Clone)]
pub struct Descriptor {
    pub name: String,
    pub environment: HashMap<String, String>,
    pub default: bool,
    pub discrete: bool,
}

/// A preference to use when choosing a GPU to launch with.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Preference {
    #[default]
    Default,
    NonDefault,
}

impl Preference {
    /// Find the preferred default GPU for this preference.
    pub fn preferred(self, gpus: &[Descriptor]) -> Option<usize> {
        let default_idx = gpus.iter().position(|gpu| gpu.default);
        if matches!(self, Preference::Default) {
            return default_idx;
        }

        let default_discrete = gpus.iter().position(|gpu| gpu.default && gpu.discrete);
        if default_discrete.is_some() {
            return default_discrete;
        }

        let non_default_discrete = gpus.iter().position(|gpu| gpu.discrete);
        let non_default = || gpus.iter().position(|gpu| !gpu.default);
        non_default_discrete.or_else(non_default).or(default_idx)
    }

    /// Select the GPU override to launch with.
    pub fn select(self, gpus: &[Descriptor]) -> Option<usize> {
        if matches!(self, Preference::Default) {
            return None;
        }

        let default_discrete = gpus.iter().position(|gpu| gpu.default && gpu.discrete);
        if default_discrete.is_some() {
            return None;
        }

        let non_default_discrete = gpus.iter().position(|gpu| gpu.discrete);
        let non_default = || gpus.iter().position(|gpu| !gpu.default);
        non_default_discrete.or_else(non_default)
    }
}
