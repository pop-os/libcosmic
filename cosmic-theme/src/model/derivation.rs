use palette::Srgba;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

use crate::composite::over;

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

    pub(crate) fn new(component: Component<C>, bg: C, on_bg: C) -> Self {
        let mut divider_c: Srgba = on_bg.clone().into();
        divider_c.alpha = 0.2;

        let divider = over(divider_c.clone(), bg.clone());
        Self {
            base: bg,
            component,
            divider: divider.into(),
            on: on_bg,
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
    /// the color of the border for the widget
    pub border: C,
    /// the color of the border for the widget when it is disabled
    pub disabled_border: C,
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
            border: self.border.into(),
            disabled_border: self.disabled_border.into(),
        }
    }

    /// helper for producing a component from a base color a neutral and an accent
    pub fn colored_component(base: C, neutral: C, accent: C, hovered: C, pressed: C) -> Self {
        let base: Srgba = base.into();
        let mut base_50 = base.clone();
        base_50.alpha *= 0.5;

        let on_20 = neutral.clone();
        let mut on_50: Srgba = on_20.clone().into();
        on_50.alpha = 0.5;

        Component {
            base: base.clone().into(),
            hover: over(hovered.clone(), base).into(),
            pressed: over(pressed, base).into(),
            selected: over(hovered, base).into(),
            selected_text: accent.clone(),
            divider: on_20,
            on: neutral,
            disabled: over(base_50, base).into(),
            on_disabled: over(on_50, base).into(),
            focus: accent,
            border: base.into(),
            disabled_border: base_50.into(),
        }
    }

    /// helper for producing a button component
    pub fn colored_button(
        base: C,
        overlay: C,
        on_button: C,
        accent: C,
        hovered: C,
        pressed: C,
    ) -> Self {
        let mut component = Component::colored_component(base, overlay, accent, hovered, pressed);
        component.on = on_button.clone();

        let mut on_disabled = on_button.into();
        on_disabled.alpha = 0.5;
        component.on_disabled = on_disabled.into();

        component
    }

    /// helper for producing a component color theme
    pub fn component(
        base: C,
        accent: C,
        on_component: C,
        hovered: C,
        pressed: C,
        is_high_contrast: bool,
        border: C,
    ) -> Self {
        let base = base.into();
        let mut base_50 = base.clone();
        base_50.alpha *= 0.5;

        let mut on_20 = on_component.clone().into();
        let mut on_50 = on_20.clone();

        on_20.alpha = 0.2;
        on_50.alpha = 0.5;

        let border = border.into();
        let mut disabled_border = border;
        disabled_border.alpha *= 0.5;

        Component {
            base: base.clone().into(),
            hover: if base.alpha < 0.001 {
                hovered.clone()
            } else {
                over(hovered.clone(), base).into()
            },
            pressed: if base.alpha < 0.001 {
                pressed.clone()
            } else {
                over(pressed.clone(), base).into()
            },
            selected: if base.alpha < 0.001 {
                hovered.clone()
            } else {
                over(hovered.clone(), base).into()
            },
            selected_text: accent.clone(),
            focus: accent.clone(),
            divider: if is_high_contrast {
                on_50.clone().into()
            } else {
                on_20.into()
            },
            on: on_component.clone(),
            disabled: over(base_50, base).into(),
            on_disabled: over(on_50, base).into(),
            border: border.into(),
            disabled_border: disabled_border.into(),
        }
    }
}
