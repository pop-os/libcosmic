// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

/// Request a list of GPUs from from switcherooctl.
pub async fn enumerate(system_conn: &zbus::Connection) -> Option<impl Iterator<Item = Descriptor>> {
    let proxy = switcheroo_control::SwitcherooControlProxy::new(system_conn)
        .await
        .ok()?;

    if !proxy.has_dual_gpu().await.ok()? {
        return None;
    }

    let gpus = proxy.get_gpus().await.ok()?;
    if gpus.is_empty() {
        return None;
    }
    Some(gpus.into_iter().map(|gpu| Descriptor {
        name: gpu.name,
        environment: gpu.environment,
        default: gpu.default,
        discrete: gpu.discrete,
    }))
}

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
    Id(usize),
    NonDefault,
}

impl Preference {
    /// Find the preferred default GPU for this preference.
    pub fn preferred(self, gpus: &[Descriptor]) -> Option<usize> {
        if let Self::Id(id) = self {
            return Some(id);
        }

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
        if let Self::Id(id) = self {
            return Some(id);
        }

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
