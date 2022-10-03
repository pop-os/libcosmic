pub use iced::Font;

pub const FONT: Font = Font::External {
    name: "Fira Sans Regular",
    bytes: include_bytes!("../res/FiraSans/Regular.otf"),
};

pub const FONT_LIGHT: Font = Font::External {
    name: "Fira Sans Light",
    bytes: include_bytes!("../res/FiraSans/Light.otf"),
};

pub const FONT_SEMIBOLD: Font = Font::External {
    name: "Fira Sans SemiBold",
    bytes: include_bytes!("../res/FiraSans/SemiBold.otf"),
};
