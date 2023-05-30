use super::ColorPicker;
use crate::{Selection, ThemeConstraints};
use anyhow::{anyhow, bail, Result};
use float_cmp::approx_eq;
use palette::{Clamp, IntoColor, Lch, RelativeContrast, Srgba};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

/// Implementation of a Cosmic color chooser which exactly meets constraints
#[derive(Debug, Default, Clone)]
pub struct Exact<C> {
    selection: Selection<C>,
    constraints: ThemeConstraints,
}

impl<C> Exact<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// create a new Exact color picker
    pub fn new(selection: Selection<C>, constraints: ThemeConstraints) -> Self {
        Self {
            selection,
            constraints,
        }
    }
}

impl<C> ColorPicker<C> for Exact<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn get_constraints(&self) -> ThemeConstraints {
        self.constraints
    }

    fn get_selection(&self) -> Selection<C> {
        self.selection.clone()
    }

    fn pick_color_graphic(
        &self,
        color: C,
        contrast: f32,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> (C, Option<anyhow::Error>) {
        let mut err = None;

        let res = self.pick_color(color.clone(), Some(contrast), grayscale, lighten);
        if let Ok(c) = res {
            return (c, err);
        } else if let Err(e) = res {
            err = Some(anyhow!("Graphic contrast {} failed: {}", contrast, e));
        }

        let res = self.pick_color(color.clone(), None, grayscale, lighten);
        if let Ok(c) = res {
            return (c, err);
        } else if let Err(e) = res {
            err = Some(e);
        }

        // return same color if no other color possible
        (color, err)
    }

    fn pick_color_text(
        &self,
        color: C,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> (C, Option<anyhow::Error>) {
        let mut err = None;

        // AAA
        let res = self.pick_color(color.clone(), Some(7.0), grayscale, lighten);
        if let Ok(c) = res {
            return (c, err);
        } else if let Err(e) = res {
            err = Some(anyhow!("AAA text contrast failed: {}", e));
        }

        // AA
        let res = self.pick_color(color.clone(), Some(4.5), grayscale, lighten);
        if let Ok(c) = res {
            return (c, err);
        } else if let Err(e) = res {
            err = Some(anyhow!("AA text contrast failed: {}", e));
        }

        let res = self.pick_color(color.clone(), None, grayscale, lighten);
        if let Ok(c) = res {
            return (c, err);
        } else if let Err(e) = res {
            err = Some(e);
        }

        (color, err)
    }

    fn pick_color(
        &self,
        color: C,
        contrast: Option<f32>,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> Result<C> {
        let srgba: Srgba = color.clone().into();
        let mut lch_color: Lch = srgba.into_color();

        // set to grayscale
        if grayscale {
            lch_color.chroma = 0.0;
        }

        // lighten or darken
        // TODO closed form solution using Lch color space contrast formula?
        // for now do binary search...

        if let Some(contrast) = contrast {
            let (min, max) = match lighten {
                Some(b) if b => (lch_color.l, 100.0),
                Some(_) => (0.0, lch_color.l),
                None => (0.0, 100.0),
            };
            let (mut l, mut r) = (min, max);

            for _ in 0..100 {
                let cur_guess_lightness = (l + r) / 2.0;
                let mut cur_guess = lch_color;
                cur_guess.l = cur_guess_lightness;
                let cur_contrast = srgba.get_contrast_ratio(&cur_guess.into_color());
                let contrast_dir = contrast > cur_contrast;
                let lightness_dir = lch_color.l < cur_guess.l;
                if approx_eq!(f32, contrast, cur_contrast, ulps = 4) {
                    lch_color = cur_guess;
                    break;
                    // TODO fix
                } else if lightness_dir && contrast_dir || !lightness_dir && !contrast_dir {
                    l = cur_guess_lightness;
                } else {
                    r = cur_guess_lightness;
                }
            }

            // clamp to valid value in range
            lch_color.clamp_self();

            // verify contrast
            let actual_contrast = srgba.get_contrast_ratio(&lch_color.into_color());
            if !approx_eq!(f32, contrast, actual_contrast, ulps = 4) {
                bail!(
                    "Failed to derive color with contrast {} from {:?}",
                    contrast,
                    color
                );
            }

            Ok(C::from(lch_color.into_color()))
        } else {
            // maximize contrast if no constraint is given
            if lch_color.l > 50.0 {
                Ok(C::from(palette::named::BLACK.into_format().into_color()))
            } else {
                Ok(C::from(palette::named::WHITE.into_format().into_color()))
            }
        }
    }
}
