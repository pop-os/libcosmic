use palette::Srgba;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

use crate::{util::over, CosmicPalette};

/// Theme Container colors of a theme, can be a theme background container, primary container, or secondary container
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Container<C> {
    /// the color of the container
    pub base: C,
    /// the color of components in the container
    pub component: Component<C>,
    /// the color of dividers in the container
    pub divider: C,
    /// the color of text in the container
    pub on: C,
}

impl<C> Container<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// convert to srgba
    pub fn into_srgba(self) -> Container<Srgba> {
        Container {
            base: self.base.into(),
            component: self.component.into_srgba(),
            divider: self.divider.into(),
            on: self.on.into(),
        }
    }

    pub(crate) fn new(
        palette: CosmicPalette<C>,
        container_type: ComponentType,
        bg: C,
        on_bg: C,
    ) -> Self {
        let mut divider_c: Srgba = on_bg.clone().into();
        divider_c.alpha = 0.2;

        let divider = over(divider_c.clone(), bg.clone());
        Self {
            base: bg,
            component: (palette, container_type).into(),
            divider: divider.into(),
            on: on_bg,
        }
    }
}

impl<C> From<(CosmicPalette<C>, ContainerType)> for Container<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn from((p, t): (CosmicPalette<C>, ContainerType)) -> Self {
        match (p, t) {
            (CosmicPalette::Dark(p), ContainerType::Background) => Self::new(
                CosmicPalette::Dark(p.clone()),
                ComponentType::Background,
                p.gray_1.clone(),
                p.neutral_7.clone(),
            ),
            (CosmicPalette::Dark(p), ContainerType::Primary) => Self::new(
                CosmicPalette::Dark(p.clone()),
                ComponentType::Primary,
                p.gray_2.clone(),
                p.neutral_8.clone(),
            ),
            (CosmicPalette::Dark(p), ContainerType::Secondary) => Self::new(
                CosmicPalette::Dark(p.clone()),
                ComponentType::Secondary,
                p.gray_3.clone(),
                p.neutral_8.clone(),
            ),
            (CosmicPalette::HighContrastDark(p), ContainerType::Background) => Self::new(
                CosmicPalette::HighContrastDark(p.clone()),
                ComponentType::Background,
                p.gray_1.clone(),
                p.neutral_8.clone(),
            ),
            (CosmicPalette::HighContrastDark(p), ContainerType::Primary) => Self::new(
                CosmicPalette::HighContrastDark(p.clone()),
                ComponentType::Primary,
                p.gray_2.clone(),
                p.neutral_9.clone(),
            ),
            (CosmicPalette::HighContrastDark(p), ContainerType::Secondary) => Self::new(
                CosmicPalette::HighContrastDark(p.clone()),
                ComponentType::Secondary,
                p.gray_3.clone(),
                p.neutral_9.clone(),
            ),
            (CosmicPalette::Light(p), ContainerType::Background) => Self::new(
                CosmicPalette::Light(p.clone()),
                ComponentType::Background,
                p.gray_1.clone(),
                p.neutral_9.clone(),
            ),
            (CosmicPalette::Light(p), ContainerType::Primary) => Self::new(
                CosmicPalette::Light(p.clone()),
                ComponentType::Primary,
                p.gray_2.clone(),
                p.neutral_8.clone(),
            ),
            (CosmicPalette::Light(p), ContainerType::Secondary) => Self::new(
                CosmicPalette::Light(p.clone()),
                ComponentType::Secondary,
                p.gray_3.clone(),
                p.neutral_8.clone(),
            ),
            (CosmicPalette::HighContrastLight(p), ContainerType::Background) => Self::new(
                CosmicPalette::HighContrastLight(p.clone()),
                ComponentType::Background,
                p.gray_1.clone(),
                p.neutral_10.clone(),
            ),
            (CosmicPalette::HighContrastLight(p), ContainerType::Primary) => Self::new(
                CosmicPalette::HighContrastLight(p.clone()),
                ComponentType::Primary,
                p.gray_2.clone(),
                p.neutral_9.clone(),
            ),
            (CosmicPalette::HighContrastLight(p), ContainerType::Secondary) => Self::new(
                CosmicPalette::HighContrastLight(p.clone()),
                ComponentType::Secondary,
                p.gray_3.clone(),
                p.neutral_9.clone(),
            ),
        }
    }
}

/// The type of the container
#[derive(Copy, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum ContainerType {
    /// Background type
    Background,
    /// Primary type
    Primary,
    /// Secondary type
    Secondary,
}

impl Default for ContainerType {
    fn default() -> Self {
        Self::Background
    }
}

impl fmt::Display for ContainerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ContainerType::Background => write!(f, "Background"),
            ContainerType::Primary => write!(f, "Primary Container"),
            ContainerType::Secondary => write!(f, "Secondary Container"),
        }
    }
}

/// The colors for a widget of the Cosmic theme
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize, Eq)]
pub struct Component<C> {
    /// The base color of the widget
    pub base: C,
    /// The color of the widget when it is hovered
    pub hover: C,
    /// the color of the widget when it is pressed
    pub pressed: C,
    /// the color of the widget when it is selected
    pub selected: C,
    /// the color of the widget when it is selected
    pub selected_text: C,
    /// the color of the widget when it is focused
    pub focus: C,
    /// the color of dividers for this widget
    pub divider: C,
    /// the color of text for this widget
    pub on: C,
    // the color of text with opacity 80 for this widget
    // pub text_opacity_80: C,
    /// the color of the widget when it is disabled
    pub disabled: C,
    /// the color of text in the widget when it is disabled
    pub on_disabled: C,
}

impl<C> Component<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// get @hover_state_color
    pub fn hover_state_color(&self) -> Srgba {
        self.hover.clone().into()
    }
    /// get @pressed_state_color
    pub fn pressed_state_color(&self) -> Srgba {
        self.pressed.clone().into()
    }
    /// get @selected_state_color
    pub fn selected_state_color(&self) -> Srgba {
        self.selected.clone().into()
    }
    /// get @selected_state_text_color
    pub fn selected_state_text_color(&self) -> Srgba {
        self.selected_text.clone().into()
    }
    /// get @focus_color
    pub fn focus_color(&self) -> Srgba {
        self.focus.clone().into()
    }
    /// convert to srgba
    pub fn into_srgba(self) -> Component<Srgba> {
        Component {
            base: self.base.into(),
            hover: self.hover.into(),
            pressed: self.pressed.into(),
            selected: self.selected.into(),
            selected_text: self.selected_text.into(),
            focus: self.focus.into(),
            divider: self.divider.into(),
            on: self.on.into(),
            disabled: self.disabled.into(),
            on_disabled: self.on_disabled.into(),
        }
    }

    /// helper for producing a component from a base color a neutral and an accent
    pub fn colored_component(base: C, neutral: C, accent: C) -> Self {
        let neutral = neutral.clone().into();
        let mut neutral_05 = neutral.clone();
        let mut neutral_10 = neutral.clone();
        let mut neutral_20 = neutral.clone();
        neutral_05.alpha = 0.05;
        neutral_10.alpha = 0.1;
        neutral_20.alpha = 0.2;

        let base: Srgba = base.into();
        let mut base_50 = base.clone();
        base_50.alpha = 0.5;

        let on_20 = neutral.clone();
        let mut on_50 = on_20.clone();

        on_50.alpha = 0.5;

        Component {
            base: base.clone().into(),
            hover: over(neutral_10, base).into(),
            pressed: over(neutral_20, base).into(),
            selected: over(neutral_10, base).into(),
            selected_text: accent.clone(),
            divider: on_20.into(),
            on: neutral.into(),
            disabled: base_50.into(),
            on_disabled: on_50.into(),
            focus: accent,
        }
    }

    /// helper for producing a component color theme
    pub fn component(
        base: C,
        component_state_overlay: C,
        base_overlay: C,
        base_overlay_alpha: f32,
        accent: C,
        on_component: C,
        is_high_contrast: bool,
    ) -> Self {
        let component_state_overlay = component_state_overlay.clone().into();
        let mut component_state_overlay_10 = component_state_overlay.clone();
        let mut component_state_overlay_20 = component_state_overlay.clone();
        component_state_overlay_10.alpha = 0.1;
        component_state_overlay_20.alpha = 0.2;

        let base = base.into();
        let mut base_overlay = base_overlay.into();
        base_overlay.alpha = base_overlay_alpha;
        let base = over(base_overlay, base);
        let mut base_50 = base.clone();
        base_50.alpha = 0.5;

        let mut on_20 = on_component.clone().into();
        let mut on_50 = on_20.clone();

        on_20.alpha = 0.2;
        on_50.alpha = 0.5;

        Component {
            base: base.clone().into(),
            hover: over(component_state_overlay_10, base).into(),
            pressed: over(component_state_overlay_20, base).into(),
            selected: over(component_state_overlay_10, base).into(),
            selected_text: accent.clone(),
            focus: accent.clone(),
            divider: if is_high_contrast {
                on_50.clone().into()
            } else {
                on_20.into()
            },
            on: on_component.clone(),
            disabled: base_50.into(),
            on_disabled: on_50.into(),
        }
    }
}

/// Derived theme element from a palette and constraints
#[derive(Debug)]
pub struct Derivation<E> {
    /// Derived  theme element
    pub derived: E,
    /// Derivation errors (Failed constraints)
    pub errors: Vec<anyhow::Error>,
}

pub(crate) enum ComponentType {
    Background,
    Primary,
    Secondary,
    Destructive,
    Warning,
    Success,
    Accent,
}

impl<C> From<(CosmicPalette<C>, ComponentType)> for Component<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn from((p, t): (CosmicPalette<C>, ComponentType)) -> Self {
        match (p, t) {
            (CosmicPalette::Dark(p), ComponentType::Background) => Self::component(
                p.gray_1,
                p.neutral_1,
                p.neutral_10,
                0.08,
                p.blue,
                p.neutral_8,
                false,
            ),

            (CosmicPalette::Dark(p), ComponentType::Primary) => Self::component(
                p.gray_2,
                p.neutral_1,
                p.neutral_10,
                0.08,
                p.blue,
                p.neutral_8,
                false,
            ),

            (CosmicPalette::Dark(p), ComponentType::Secondary) => Self::component(
                p.gray_3,
                p.neutral_1,
                p.neutral_10,
                0.08,
                p.blue,
                p.neutral_9,
                false,
            ),
            (CosmicPalette::HighContrastDark(p), ComponentType::Background) => Self::component(
                p.gray_1,
                p.neutral_1,
                p.neutral_10,
                0.08,
                p.blue,
                p.neutral_9,
                true,
            ),
            (CosmicPalette::HighContrastDark(p), ComponentType::Primary) => Self::component(
                p.gray_2,
                p.neutral_1,
                p.neutral_10,
                0.08,
                p.blue,
                p.neutral_9,
                true,
            ),
            (CosmicPalette::HighContrastDark(p), ComponentType::Secondary) => Self::component(
                p.gray_3,
                p.neutral_1,
                p.neutral_10.clone(),
                0.08,
                p.blue,
                p.neutral_10,
                true,
            ),

            (CosmicPalette::Light(p), ComponentType::Background) => Component::component(
                p.gray_1.clone(),
                p.neutral_1.clone(),
                p.neutral_1,
                0.75,
                p.blue.clone(),
                p.neutral_8,
                false,
            ),
            (CosmicPalette::Light(p), ComponentType::Primary) => Component::component(
                p.gray_2.clone(),
                p.neutral_1.clone(),
                p.neutral_1,
                0.9,
                p.blue.clone(),
                p.neutral_8,
                false,
            ),
            (CosmicPalette::Light(p), ComponentType::Secondary) => Component::component(
                p.gray_3.clone(),
                p.neutral_1.clone(),
                p.neutral_1,
                1.0,
                p.blue.clone(),
                p.neutral_8,
                false,
            ),
            (CosmicPalette::HighContrastLight(p), ComponentType::Background) => {
                Component::component(
                    p.gray_1.clone(),
                    p.neutral_1.clone(),
                    p.neutral_1,
                    0.75,
                    p.blue.clone(),
                    p.neutral_9,
                    true,
                )
            }
            (CosmicPalette::HighContrastLight(p), ComponentType::Primary) => Component::component(
                p.gray_2.clone(),
                p.neutral_1.clone(),
                p.neutral_1,
                0.9,
                p.blue.clone(),
                p.neutral_9,
                true,
            ),
            (CosmicPalette::HighContrastLight(p), ComponentType::Secondary) => {
                Component::component(
                    p.gray_3.clone(),
                    p.neutral_1.clone(),
                    p.neutral_1,
                    1.0,
                    p.blue.clone(),
                    p.neutral_9,
                    true,
                )
            }

            (CosmicPalette::Dark(p), ComponentType::Destructive)
            | (CosmicPalette::Light(p), ComponentType::Destructive)
            | (CosmicPalette::HighContrastLight(p), ComponentType::Destructive)
            | (CosmicPalette::HighContrastDark(p), ComponentType::Destructive) => {
                Component::colored_component(p.red.clone(), p.neutral_1.clone(), p.blue.clone())
            }

            (CosmicPalette::Dark(p), ComponentType::Warning)
            | (CosmicPalette::Light(p), ComponentType::Warning)
            | (CosmicPalette::HighContrastLight(p), ComponentType::Warning)
            | (CosmicPalette::HighContrastDark(p), ComponentType::Warning) => {
                Component::colored_component(p.yellow.clone(), p.neutral_1, p.blue.clone())
            }

            (CosmicPalette::Dark(p), ComponentType::Success)
            | (CosmicPalette::Light(p), ComponentType::Success)
            | (CosmicPalette::HighContrastLight(p), ComponentType::Success)
            | (CosmicPalette::HighContrastDark(p), ComponentType::Success) => {
                Component::colored_component(p.green.clone(), p.neutral_1, p.blue.clone())
            }

            (CosmicPalette::Dark(p), ComponentType::Accent)
            | (CosmicPalette::Light(p), ComponentType::Accent)
            | (CosmicPalette::HighContrastDark(p), ComponentType::Accent)
            | (CosmicPalette::HighContrastLight(p), ComponentType::Accent) => {
                Component::colored_component(p.blue.clone(), p.neutral_1, p.blue.clone())
            }
        }
    }
}
