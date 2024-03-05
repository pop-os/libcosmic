use palette::Srgba;
use serde::{Deserialize, Serialize};

use crate::composite::over;

/// Theme Container colors of a theme, can be a theme background container, primary container, or secondary container
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[must_use]
pub struct Container {
    /// the color of the container
    pub base: Srgba,
    /// the color of components in the container
    pub component: Component,
    /// the color of dividers in the container
    pub divider: Srgba,
    /// the color of text in the container
    pub on: Srgba,
}

impl Container {
    pub(crate) fn new(component: Component, base: Srgba, on: Srgba) -> Self {
        let mut divider_c = base;
        divider_c.alpha = 0.2;

        Self {
            base,
            component,
            divider: over(divider_c, base),
            on,
        }
    }
}

/// The colors for a widget of the Cosmic theme
#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
#[must_use]
pub struct Component {
    /// The base color of the widget
    pub base: Srgba,
    /// The color of the widget when it is hovered
    pub hover: Srgba,
    /// the color of the widget when it is pressed
    pub pressed: Srgba,
    /// the color of the widget when it is selected
    pub selected: Srgba,
    /// the color of the widget when it is selected
    pub selected_text: Srgba,
    /// the color of the widget when it is focused
    pub focus: Srgba,
    /// the color of dividers for this widget
    pub divider: Srgba,
    /// the color of text for this widget
    pub on: Srgba,
    // the color of text with opacity 80 for this widget
    // pub text_opacity_80: Srgba,
    /// the color of the widget when it is disabled
    pub disabled: Srgba,
    /// the color of text in the widget when it is disabled
    pub on_disabled: Srgba,
    /// the color of the border for the widget
    pub border: Srgba,
    /// the color of the border for the widget when it is disabled
    pub disabled_border: Srgba,
}

#[allow(clippy::must_use_candidate)]
#[allow(clippy::doc_markdown)]
impl Component {
    /// get @hover_state_color
    pub fn hover_state_color(&self) -> Srgba {
        self.hover
    }

    /// get @pressed_state_color
    pub fn pressed_state_color(&self) -> Srgba {
        self.pressed
    }

    /// get @selected_state_color
    pub fn selected_state_color(&self) -> Srgba {
        self.selected
    }

    /// get @selected_state_text_color
    pub fn selected_state_text_color(&self) -> Srgba {
        self.selected_text
    }

    /// get @focus_color
    pub fn focus_color(&self) -> Srgba {
        self.focus
    }

    /// helper for producing a component from a base color a neutral and an accent
    pub fn colored_component(
        base: Srgba,
        neutral: Srgba,
        accent: Srgba,
        hovered: Srgba,
        pressed: Srgba,
    ) -> Self {
        let base: Srgba = base;
        let mut base_50 = base;
        base_50.alpha *= 0.5;

        let on_20 = neutral;
        let mut on_50: Srgba = on_20;
        on_50.alpha = 0.5;

        Component {
            base,
            hover: over(hovered, base),
            pressed: over(pressed, base),
            selected: over(hovered, base),
            selected_text: accent,
            divider: on_20,
            on: neutral,
            disabled: over(base_50, base),
            on_disabled: over(on_50, base),
            focus: accent,
            border: base,
            disabled_border: base_50,
        }
    }

    /// helper for producing a button component
    pub fn colored_button(
        base: Srgba,
        overlay: Srgba,
        on_button: Srgba,
        accent: Srgba,
        hovered: Srgba,
        pressed: Srgba,
    ) -> Self {
        let mut component = Component::colored_component(base, overlay, accent, hovered, pressed);
        component.on = on_button;

        let mut on_disabled = on_button;
        on_disabled.alpha = 0.5;
        component.on_disabled = on_disabled;

        component
    }

    /// helper for producing a component color theme
    #[allow(clippy::self_named_constructors)]
    pub fn component(
        base: Srgba,
        accent: Srgba,
        on_component: Srgba,
        hovered: Srgba,
        pressed: Srgba,
        is_high_contrast: bool,
        border: Srgba,
    ) -> Self {
        let mut base_50 = base;
        base_50.alpha *= 0.5;

        let mut on_20 = on_component;
        let mut on_50 = on_20;

        on_20.alpha = 0.2;
        on_50.alpha = 0.5;

        let mut disabled_border = border;
        disabled_border.alpha *= 0.5;

        Component {
            base,
            hover: if base.alpha < 0.001 {
                hovered
            } else {
                over(hovered, base)
            },
            pressed: if base.alpha < 0.001 {
                pressed
            } else {
                over(pressed, base)
            },
            selected: if base.alpha < 0.001 {
                hovered
            } else {
                over(hovered, base)
            },
            selected_text: accent,
            focus: accent,
            divider: if is_high_contrast { on_50 } else { on_20 },
            on: on_component,
            disabled: over(base_50, base),
            on_disabled: over(on_50, base),
            border,
            disabled_border,
        }
    }
}
