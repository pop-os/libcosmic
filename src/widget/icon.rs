use iced::widget::svg;

pub fn icon(name: &str, size: u16) -> svg::Svg {
    let handle = match freedesktop_icons::lookup(name)
        .with_size(size)
        .with_theme("Pop")
        .with_cache()
        .force_svg()
        .find()
    {
        Some(path) => svg::Handle::from_path(path),
        None => {
            eprintln!("icon '{}' size {} not found", name, size);
            svg::Handle::from_memory(Vec::new())
        },
    };
    svg::Svg::new(handle)
}
