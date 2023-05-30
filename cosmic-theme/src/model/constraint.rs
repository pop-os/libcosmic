/// Cosmic theme custom constraints which are used to pick colors
#[derive(Copy, Clone, Debug)]
pub struct ThemeConstraints {
    /// requested contrast ratio for elevated surfaces
    pub elevated_contrast_ratio: f32,
    /// requested contrast ratio for dividers
    pub divider_contrast_ratio: f32,
    /// requested contrast ratio for text
    pub text_contrast_ratio: f32,
    /// gray scale or color for dividers
    pub divider_gray_scale: bool,
    /// elevated surfaces are lightened or darkened
    pub lighten: bool,
}

impl Default for ThemeConstraints {
    fn default() -> Self {
        Self {
            elevated_contrast_ratio: 1.1,
            divider_contrast_ratio: 1.51,
            text_contrast_ratio: 7.0,
            divider_gray_scale: true,
            lighten: true,
        }
    }
}
