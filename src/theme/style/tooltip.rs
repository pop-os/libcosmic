use iced::Color;

use crate::widget::wayland::tooltip::Catalog;

#[derive(Default)]
pub enum Tooltip {
    #[default]
    Default,
}

impl Catalog for crate::Theme {
    type Class = Tooltip;

    fn style(&self, style: &Self::Class) -> crate::widget::wayland::tooltip::Style {
        let cosmic = self.cosmic();

        match style {
            Tooltip::Default => crate::widget::wayland::tooltip::Style {
                text_color: cosmic.on_bg_color().into(),
                background: None,
                border_width: 0.0,
                border_radius: cosmic.corner_radii.radius_0.into(),
                border_color: Color::TRANSPARENT,
                shadow_offset: iced::Vector::default(),
                outline_width: Default::default(),
                outline_color: Color::TRANSPARENT,
                icon_color: None,
            },
        }
    }
}
