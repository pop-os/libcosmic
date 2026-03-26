//! Color representation and serde helpers for the Cosmic theme

use hex_color::HexColor;
use palette::{Srgb, Srgba};
use serde::{Deserialize, Serialize};

/// A color in the Cosmic theme for serialization and deserialization
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorRepr {
    /// A color represented as a hex string
    #[serde(with = "hex_color::rgba")]
    Hex(HexColor),
    /// A color represented as an RGBA value
    Rgba(Srgba),
    /// A color represented as an RGB value
    Rgb(Srgb),
}

/// An optional color in the Cosmic theme for serialization and deserialization
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ColorReprOption(Option<ColorRepr>);

impl From<Srgb> for ColorRepr {
    fn from(color: Srgb) -> Self {
        let rgb_u8: Srgb<u8> = color.into_format();
        ColorRepr::Hex(HexColor {
            r: rgb_u8.red,
            g: rgb_u8.green,
            b: rgb_u8.blue,
            a: 255,
        })
    }
}

impl From<Srgba> for ColorRepr {
    fn from(color: Srgba) -> Self {
        let rgba_u8: Srgba<u8> = color.into_format();
        ColorRepr::Hex(HexColor {
            r: rgba_u8.red,
            g: rgba_u8.green,
            b: rgba_u8.blue,
            a: rgba_u8.alpha,
        })
    }
}

impl From<ColorRepr> for Srgb {
    fn from(value: ColorRepr) -> Self {
        match value {
            ColorRepr::Hex(hex) => Srgb::<u8>::new(hex.r, hex.g, hex.b).into_format(),
            ColorRepr::Rgb(rgb) => rgb,
            ColorRepr::Rgba(rgba) => Srgb::new(rgba.red, rgba.green, rgba.blue),
        }
    }
}

impl From<ColorRepr> for Srgba {
    fn from(value: ColorRepr) -> Self {
        match value {
            ColorRepr::Hex(hex) => Srgba::<u8>::new(hex.r, hex.g, hex.b, hex.a).into_format(),
            ColorRepr::Rgb(rgb) => Srgba::new(rgb.red, rgb.green, rgb.blue, 1.0),
            ColorRepr::Rgba(rgba) => rgba,
        }
    }
}

impl From<ColorReprOption> for Option<Srgb> {
    fn from(value: ColorReprOption) -> Self {
        value.0.map(std::convert::Into::into)
    }
}

impl From<ColorReprOption> for Option<Srgba> {
    fn from(value: ColorReprOption) -> Self {
        value.0.map(std::convert::Into::into)
    }
}

impl From<Option<Srgb>> for ColorReprOption {
    fn from(value: Option<Srgb>) -> Self {
        ColorReprOption(value.map(std::convert::Into::into))
    }
}

impl From<Option<Srgba>> for ColorReprOption {
    fn from(value: Option<Srgba>) -> Self {
        ColorReprOption(value.map(std::convert::Into::into))
    }
}

/// A trait for converting between a color type and its representation for serialization and deserialization
pub trait ConvColorRepr: Sized {
    /// Convert from a color representation to the color type
    fn from_repr(repr: ColorRepr) -> Self;
    /// Convert from the color type to its representation for serialization
    fn to_repr(&self) -> ColorRepr;
}

impl ConvColorRepr for Srgba {
    fn from_repr(repr: ColorRepr) -> Self {
        repr.into()
    }

    fn to_repr(&self) -> ColorRepr {
        (*self).into()
    }
}

impl ConvColorRepr for Srgb {
    fn from_repr(repr: ColorRepr) -> Self {
        repr.into()
    }

    fn to_repr(&self) -> ColorRepr {
        (*self).into()
    }
}

/// Serde helpers for serializing and deserializing colors in the Cosmic theme
pub mod color_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serialize a color to a hex string
    pub fn serialize<T, S>(color: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ConvColorRepr,
        S: Serializer,
    {
        let repr = color.to_repr();
        repr.serialize(serializer)
    }

    /// Deserialize a color from a hex string or RGB/RGBA
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: ConvColorRepr,
        D: Deserializer<'de>,
    {
        let repr = ColorRepr::deserialize(deserializer)?;
        Ok(T::from_repr(repr))
    }

    /// Serde helpers for serializing and deserializing optional colors in the Cosmic theme
    pub mod option {
        use super::*;

        /// Serialize an optional color
        pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: ConvColorRepr,
            S: Serializer,
        {
            match value {
                Some(v) => super::serialize(v, serializer),
                None => serializer.serialize_none(),
            }
        }

        /// Deserialize an optional color
        pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
        where
            T: ConvColorRepr,
            D: Deserializer<'de>,
        {
            let opt = Option::<ColorRepr>::deserialize(deserializer)?;
            Ok(opt.map(T::from_repr))
        }
    }
}
