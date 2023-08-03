use csscolorparser::Color;
use palette::Srgba;
use serde::{Deserialize, Serialize};

/// utility wrapper for serializing and deserializing colors with arbitrary CSS
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct CssColor {
    c: Color,
}

impl From<Srgba> for CssColor {
    fn from(c: Srgba) -> Self {
        Self {
            c: Color {
                r: c.red as f64,
                g: c.green as f64,
                b: c.blue as f64,
                a: c.alpha as f64,
            },
        }
    }
}

impl Into<Srgba> for CssColor {
    fn into(self) -> Srgba {
        Srgba::new(
            self.c.r as f32,
            self.c.g as f32,
            self.c.b as f32,
            self.c.a as f32,
        )
    }
}
