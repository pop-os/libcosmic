// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Stylesheet implements for [`crate::Theme`]

mod button;
pub use self::button::Button;

mod dropdown;

pub mod iced;
#[doc(inline)]
pub use self::iced::Checkbox;
#[doc(inline)]
pub use self::iced::Container;
#[doc(inline)]
pub use self::iced::ProgressBar;
#[doc(inline)]
pub use self::iced::Rule;
#[doc(inline)]
pub use self::iced::Svg;
#[doc(inline)]
pub use self::iced::Text;

pub mod menu_bar;

mod segmented_button;
#[doc(inline)]
pub use self::segmented_button::SegmentedButton;

mod text_input;
#[doc(inline)]
pub use self::text_input::TextInput;

#[cfg(all(feature = "wayland", feature = "winit"))]
pub mod tooltip;
#[cfg(all(feature = "wayland", feature = "winit"))]
pub use tooltip::Tooltip;
