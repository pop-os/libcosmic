use almost::equal;
use palette::{convert::FromColorUnclamped, ClampAssign, Oklch, Srgb};

/// Get an array of 100 colors with a specific hue and chroma
/// over the full range of lightness.
/// Colors which are not valid Srgb will fallback to a color with the nearest valid chroma.
pub fn steps(mut c: Oklch) -> [Srgb; 100] {
    let mut steps = [Srgb::new(0.0, 0.0, 0.0); 100];

    for i in 0..steps.len() {
        let lightness = i as f32 / 100.0;
        c.l = lightness;
        steps[i] = oklch_to_srgba_nearest_chroma(c)
    }

    steps
}

/// find the nearest chroma which makes our color a valid color in Srgb
pub fn oklch_to_srgba_nearest_chroma(mut c: Oklch) -> Srgb {
    let mut r_chroma = c.chroma;
    let mut l_chroma = 0.0;
    // exit early if we found it right away
    let mut new_c = Srgb::from_color_unclamped(c);

    if is_valid_srgb(new_c) {
        new_c.clamp_assign();
        return new_c;
    }

    // is this an excessive depth to search?
    for _ in 0..64 {
        let new_c = Srgb::from_color_unclamped(c);
        if is_valid_srgb(new_c) {
            l_chroma = c.chroma;
            c.chroma = (c.chroma + r_chroma) / 2.0;
        } else {
            r_chroma = c.chroma;
            c.chroma = (c.chroma + l_chroma) / 2.0;
        }
    }
    Srgb::from_color_unclamped(c)
}

/// checks that the color is valid srgb
pub fn is_valid_srgb(c: Srgb) -> bool {
    (equal(c.red, Srgb::max_red()) || (c.red >= Srgb::min_red() && c.red <= Srgb::max_red()))
        && (equal(c.blue, Srgb::max_blue())
            || (c.blue >= Srgb::min_blue() && c.blue <= Srgb::max_blue()))
        && (equal(c.green, Srgb::max_green())
            || (c.green >= Srgb::min_green() && c.green <= Srgb::max_green()))
}

#[cfg(test)]
mod tests {
    use almost::equal;
    use palette::{OklabHue, Srgb};

    use super::{is_valid_srgb, oklch_to_srgba_nearest_chroma};

    #[test]
    fn test_valid_check() {
        assert!(is_valid_srgb(Srgb::new(1.0, 1.0, 1.0)));
        assert!(is_valid_srgb(Srgb::new(0.0, 0.0, 0.0)));
        assert!(is_valid_srgb(Srgb::new(0.5, 0.5, 0.5)));
        assert!(!is_valid_srgb(Srgb::new(-0.1, 0.0, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(0.0, -0.1, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(-0.0, 0.0, -0.1)));
        assert!(!is_valid_srgb(Srgb::new(-100.1, 0.0, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(0.0, -100.1, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(-0.0, 0.0, -100.1)));
        assert!(!is_valid_srgb(Srgb::new(1.1, 0.0, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(0.0, 1.1, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(-0.0, 0.0, 1.1)));
        assert!(!is_valid_srgb(Srgb::new(100.1, 0.0, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(0.0, 100.1, 0.0)));
        assert!(!is_valid_srgb(Srgb::new(-0.0, 0.0, 100.1)));
    }

    #[test]
    fn test_conversion_boundaries() {
        let c1 = palette::Oklch::new(0.0, 0.288, OklabHue::from_degrees(0.0));
        let srgb = oklch_to_srgba_nearest_chroma(c1);
        equal(srgb.red, 0.0);
        equal(srgb.blue, 0.0);
        equal(srgb.green, 0.0);

        let c1 = palette::Oklch::new(1.0, 0.288, OklabHue::from_degrees(0.0));
        let srgb = oklch_to_srgba_nearest_chroma(c1);

        equal(srgb.red, 1.0);
        equal(srgb.blue, 1.0);
        equal(srgb.green, 1.0);
    }

    #[test]
    fn test_conversion_colors() {
        let c1 = palette::Oklch::new(0.4608, 0.11111, OklabHue::new(57.31));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 133);
        assert!(srgb.green == 69);
        assert!(srgb.blue == 0);

        let c1 = palette::Oklch::new(0.30, 0.08, OklabHue::new(35.0));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 78);
        assert!(srgb.green == 27);
        assert!(srgb.blue == 15);

        let c1 = palette::Oklch::new(0.757, 0.146, OklabHue::new(301.2));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 192);
        assert!(srgb.green == 153);
        assert!(srgb.blue == 253);
    }

    #[test]
    fn test_conversion_fallback_colors() {
        let c1 = palette::Oklch::new(0.70, 0.284, OklabHue::new(35.0));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 255);
        assert!(srgb.green == 103);
        assert!(srgb.blue == 65);

        let c1 = palette::Oklch::new(0.757, 0.239, OklabHue::new(301.2));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 193);
        assert!(srgb.green == 152);
        assert!(srgb.blue == 255);

        let c1 = palette::Oklch::new(0.163, 0.333, OklabHue::new(141.0));
        let srgb = oklch_to_srgba_nearest_chroma(c1).into_format::<u8>();
        assert!(srgb.red == 1);
        assert!(srgb.green == 19);
        assert!(srgb.blue == 0);
    }
}
