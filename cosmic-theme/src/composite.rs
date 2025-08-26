use palette::Srgba;

/// straight alpha "A over B" operator on non-linear srgba
pub fn over<A: Into<Srgba>, B: Into<Srgba>>(a: A, b: B) -> Srgba {
    let a = a.into();
    let b = b.into();
    let o_a = (alpha_over(a.alpha, b.alpha)).clamp(0.0, 1.0);
    let o_r = (c_over(a.red, b.red, a.alpha, b.alpha, o_a)).clamp(0.0, 1.0);
    let o_g = (c_over(a.green, b.green, a.alpha, b.alpha, o_a)).clamp(0.0, 1.0);
    let o_b = (c_over(a.blue, b.blue, a.alpha, b.alpha, o_a)).clamp(0.0, 1.0);

    Srgba::new(o_r, o_g, o_b, o_a)
}

fn alpha_over(a: f32, b: f32) -> f32 {
    a + b * (1.0 - a)
}

fn c_over(a: f32, b: f32, a_alpha: f32, b_alpha: f32, o_alpha: f32) -> f32 {
    a * a_alpha + b * b_alpha * (1.0 - a_alpha) / o_alpha
}
