use palette::{named, IntoColor, Lch, Srgba};
use std::convert::TryFrom;

/// A Selection is a group of colors from which a cosmic palette can be derived
#[derive(Copy, Clone, Debug, Default)]
pub struct Selection<C> {
    /// base background container color
    pub background: C,
    /// base primary container color
    pub primary_container: C,
    /// base secondary container color
    pub secondary_container: C,
    /// base accent color
    pub accent: C,
    /// custom accent color (overrides base)
    pub accent_fg: Option<C>,
    /// custom accent nav handle text color (overrides base)
    pub accent_nav_handle_fg: Option<C>,
    /// base destructive element color
    pub destructive: C,
    /// base destructive element color
    pub warning: C,
    /// base destructive element color
    pub success: C,
}

// vector should be in order of most common
impl<C> TryFrom<Vec<Srgba>> for Selection<C>
where
    C: Clone + From<Srgba>,
{
    type Error = anyhow::Error;

    fn try_from(mut colors: Vec<Srgba>) -> Result<Self, Self::Error> {
        if colors.len() < 8 {
            anyhow::bail!("length of inputted vector must be at least 8.")
        } else {
            let lch_colors: Vec<Lch> = colors
                .iter()
                .map(|x| {
                    let srgba: Srgba = x.clone().into();
                    srgba.color.into_format().into_color()
                })
                .collect();

            let red_lch: Lch = named::CRIMSON.into_format().into_color();
            let mut reddest_i = 1;
            for (i, c) in lch_colors[1..].iter().enumerate() {
                let d_cur = (c.hue.to_degrees() - red_lch.hue.to_degrees()).abs();
                let reddest_d = (lch_colors[reddest_i].hue.to_degrees().abs()
                    - red_lch.hue.to_degrees().abs())
                .abs();
                if d_cur < reddest_d {
                    reddest_i = i;
                }
            }

            let yellow_lch: Lch = named::YELLOW.into_format().into_color();
            let mut yellow_i = 1;
            for (i, c) in lch_colors[1..].iter().enumerate() {
                let d_cur = (c.hue.to_degrees() - yellow_lch.hue.to_degrees()).abs();
                let reddest_d = (lch_colors[yellow_i].hue.to_degrees().abs()
                    - yellow_lch.hue.to_degrees().abs())
                .abs();
                if d_cur < reddest_d {
                    yellow_i = i;
                }
            }

            let green_lch: Lch = named::GREEN.into_format().into_color();
            let mut green_i = 1;
            for (i, c) in lch_colors[1..].iter().enumerate() {
                let d_cur = (c.hue.to_degrees() - green_lch.hue.to_degrees()).abs();
                let reddest_d = (lch_colors[green_i].hue.to_degrees().abs()
                    - green_lch.hue.to_degrees().abs())
                .abs();
                if d_cur < reddest_d {
                    green_i = i;
                }
            }

            let red = colors.remove(reddest_i);
            let green = colors.remove(green_i);
            let yellow = colors.remove(yellow_i);

            Ok(Self {
                background: colors[0].into(),
                primary_container: colors[1].into(),
                secondary_container: colors[3].into(),
                accent: colors[2].into(),
                accent_fg: Some(colors[2].into()),
                accent_nav_handle_fg: Some(colors[2].into()),
                destructive: red.into(),
                warning: yellow.into(),
                success: green.into(),
            })
        }
    }
}
