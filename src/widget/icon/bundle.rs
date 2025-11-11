// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Embedded icons for platforms which do not support icon themes yet.

/// Icon bundling is not enabled on unix platforms.
pub fn get(icon_name: &str) -> Option<super::Data> {
    None
}

#[cfg(not(unix))]
/// Get a bundled icon on non-unix platforms.
pub fn get(icon_name: &str) -> Option<super::Data> {
    ICONS
        .get(icon_name)
        .map(|bytes| super::Data::Svg(crate::iced::widget::svg::Handle::from_memory(*bytes)))
}

#[cfg(not(unix))]
#[crabtime::expression]
fn comptime_icon_bundler() -> String {
    let manifest_dir = std::path::Path::new(crabtime::WORKSPACE_PATH);
    let icon_paths = [
        "cosmic-icons/freedesktop/scalable",
        "cosmic-icons/extra/scalable",
    ];

    let key_value_assignments = icon_paths
        .into_iter()
        .map(|path| manifest_dir.join(path))
        .inspect(|icon_path| assert!(icon_path.exists(), "path = {icon_path:?}"))
        .map(|icon_path| std::fs::read_dir(icon_path).unwrap())
        .flat_map(|dir| {
            dir.flat_map(|entry| entry.unwrap().path().read_dir().unwrap())
                .map(|entry| {
                    let entry = entry.unwrap();
                    let path = entry.path().canonicalize().unwrap();
                    let file_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
                    let path = path.into_os_string().into_string().unwrap();
                    (file_name, path)
                })
        })
        .fold(
            std::collections::BTreeMap::new(),
            |mut set, (name, path)| {
                set.insert(name, path);
                set
            },
        )
        .into_iter()
        .fold(String::new(), |mut output, (name, path)| {
            output.push_str(&format!("    \"{name}\" => include_bytes!(\"{path}\"),\n"));
            output
        });

    ["phf::phf_map!(\n", &key_value_assignments, ")"].concat()
}

#[cfg(not(unix))]
static ICONS: phf::Map<&'static str, &'static [u8]> = {
    comptime_icon_bundler! {}
};
