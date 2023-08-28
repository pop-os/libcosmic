//! Change the appearance of a text input.
use iced_core::{Background, BorderRadius, Color};

/// The appearance of a text input.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the text input.
    pub background: Background,
    /// The border radius of the text input.
    pub border_radius: BorderRadius,
    /// The border offset
    pub border_offset: Option<f32>,
    /// The border width of the text input.
    pub border_width: f32,
    /// The border [`Color`] of the text input.
    pub border_color: Color,
    /// The label [`Color`] of the text input.
    pub label_color: Color,
    /// The text [`Color`] of the text input.
    pub selected_text_color: Color,
    /// The text [`Color`] of the text input.
    pub text_color: Color,
    /// The selected fill [`Color`] of the text input.
    pub selected_fill: Color,
}

/// A set of rules that dictate the style of a text input.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the style of an active text input.
    fn active(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of an errored text input.
    fn error(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of a focused text input.
    fn focused(&self, style: &Self::Style) -> Appearance;

    /// Produces the [`Color`] of the placeholder of a text input.
    fn placeholder_color(&self, style: &Self::Style) -> Color;

    /// Produces the style of an hovered text input.
    fn hovered(&self, style: &Self::Style) -> Appearance {
        self.focused(style)
    }

    /// Produces the style of a disabled text input.
    fn disabled(&self, style: &Self::Style) -> Appearance;
}

#[derive(Default)]
pub enum TextInput {
    #[default]
    Default,
    ExpandableSearch,
    Search,
    Inline,
    Custom {
        active: Box<dyn Fn(&crate::Theme) -> Appearance>,
        error: Box<dyn Fn(&crate::Theme) -> Appearance>,
        hovered: Box<dyn Fn(&crate::Theme) -> Appearance>,
        focused: Box<dyn Fn(&crate::Theme) -> Appearance>,
        disabled: Box<dyn Fn(&crate::Theme) -> Appearance>,
        placeholder_color: Box<dyn Fn(&crate::Theme) -> Color>,
    },
}

impl StyleSheet for crate::Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;
        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;
        match style {
            TextInput::Default => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: self.current_container().component.divider.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::ExpandableSearch => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_xl.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_xl.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: self.current_container().component.divider.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { active, .. } => active(self),
        }
    }

    fn error(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;
        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: Color::from(palette.destructive_color()),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search | TextInput::ExpandableSearch => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_xl.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::TRANSPARENT.into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { error, .. } => error(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;
        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: None,
                border_color: palette.accent.base.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_xl.into(),
                border_offset: None,
                border_width: 1.0,
                border_color: palette.accent.base.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::ExpandableSearch => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_xl.into(),
                border_offset: None,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::from(self.current_container().component.hover).into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { hovered, .. } => hovered(self),
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        let palette = self.cosmic();
        let mut bg = palette.palette.neutral_7;
        bg.alpha = 0.25;
        let corner = palette.corner_radii;
        let label_color = palette.palette.neutral_9;

        match style {
            TextInput::Default => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_s.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: palette.accent.base.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Search | TextInput::ExpandableSearch => Appearance {
                background: Color::from(bg).into(),
                border_radius: corner.radius_xl.into(),
                border_width: 1.0,
                border_offset: Some(2.0),
                border_color: palette.accent.base.into(),
                text_color: self.current_container().on.into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Inline => Appearance {
                background: Color::from(palette.accent.base).into(),
                border_radius: corner.radius_0.into(),
                border_width: 0.0,
                border_offset: None,
                border_color: Color::TRANSPARENT,
                // TODO use regular text color here after text rendering handles multiple colors
                // in this case, for selected and unselected text
                text_color: palette.on_accent_color().into(),
                selected_text_color: palette.on_accent_color().into(),
                selected_fill: palette.accent_color().into(),
                label_color: label_color.into(),
            },
            TextInput::Custom { focused, .. } => focused(self),
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        if let TextInput::Custom {
            placeholder_color, ..
        } = style
        {
            return placeholder_color(self);
        }
        let palette = self.cosmic();
        let mut neutral_9 = palette.palette.neutral_9;
        neutral_9.alpha = 0.7;
        neutral_9.into()
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        if let TextInput::Custom { disabled, .. } = style {
            return disabled(self);
        }
        self.active(style)
    }
}
