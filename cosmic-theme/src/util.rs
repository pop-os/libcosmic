use csscolorparser::Color;
use palette::Srgba;
use serde::{Deserialize, Serialize};

/// utility wrapper for serializing and deserializing colors with arbitrary CSS
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

/// straight alpha "A over B" operator on non-linear srgba
pub fn over<A: Into<Srgba>, B: Into<Srgba>>(a: A, b: B) -> Srgba {
    let a = a.into();
    let b = b.into();
    let o_a = (alpha_over(a.alpha, b.alpha)).max(0.0).min(1.0);
    let o_r = (c_over(a.red, b.red, a.alpha, b.alpha, o_a))
        .max(0.0)
        .min(1.0);
    let o_g = (c_over(a.green, b.green, a.alpha, b.alpha, o_a))
        .max(0.0)
        .min(1.0);
    let o_b = (c_over(a.blue, b.blue, a.alpha, b.alpha, o_a))
        .max(0.0)
        .min(1.0);

    Srgba::new(o_r, o_g, o_b, o_a)
}

fn alpha_over(a: f32, b: f32) -> f32 {
    a + b * (1.0 - a)
}

fn c_over(a: f32, b: f32, a_alpha: f32, b_alpha: f32, o_alpha: f32) -> f32 {
    a * a_alpha + b * b_alpha * (1.0 - a_alpha) / o_alpha
}
