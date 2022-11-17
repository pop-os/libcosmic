use iced::{widget::{svg, Image}, Length};

pub fn icon<Renderer>(name: &str, size: u16) -> svg::Svg<Renderer>
where
    Renderer: iced_native::svg::Renderer,
    Renderer::Theme: iced_native::svg::StyleSheet,
{
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
        }
    };
    svg::Svg::new(handle)
        .width(Length::Units(size))
        .height(Length::Units(size))
}

pub fn image_icon(name: &str, size: u16) -> Option<Image>
{
    freedesktop_icons::lookup(name)
        .with_size(size)
        .with_cache()
        .find().map(|path| Image::new(path).width(Length::Units(size))
        .height(Length::Units(size)))
}
