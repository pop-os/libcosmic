fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    generate_bundled_icons();
}

fn generate_bundled_icons() {
    println!("cargo::rerun-if-changed=cosmic-icons");

    let manifest_dir = std::path::Path::new(std::env!("CARGO_MANIFEST_DIR"));
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

    let code = [
        "static ICONS: phf::Map<&'static str, &'static [u8]> = phf::phf_map!(\n",
        &key_value_assignments,
        ");",
    ]
    .concat();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_file = std::path::Path::new(&out_dir).join("bundled_icons.rs");
    std::fs::write(&out_file, &code).unwrap();
}
