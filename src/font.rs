pub use iced::Font;

pub const FONT: Font = Font::External {
    name: "Fira Sans Regular",
    bytes: include_bytes!("../res/Fira/Sans/Regular.otf"),
};

pub const FONT_LIGHT: Font = Font::External {
    name: "Fira Sans Light",
    bytes: include_bytes!("../res/Fira/Sans/Light.otf"),
};

pub const FONT_SEMIBOLD: Font = Font::External {
    name: "Fira Sans SemiBold",
    bytes: include_bytes!("../res/Fira/Sans/SemiBold.otf"),
};
