// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub use iced::Font;

pub const FONT: Font = Font::External {
    name: "Fira Sans Regular",
    bytes: include_bytes!("../res/Fira/FiraSans-Regular.otf"),
};

pub const FONT_LIGHT: Font = Font::External {
    name: "Fira Sans Light",
    bytes: include_bytes!("../res/Fira/FiraSans-Light.otf"),
};

pub const FONT_SEMIBOLD: Font = Font::External {
    name: "Fira Sans SemiBold",
    bytes: include_bytes!("../res/Fira/FiraSans-SemiBold.otf"),
};
