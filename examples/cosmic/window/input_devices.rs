use super::{Page, SubPage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputDevicesPage {
    Keyboard,
    Touchpad,
    Mouse,
}

impl SubPage for InputDevicesPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "Keyboard",
            Touchpad => "Touchpad",
            Mouse => "Mouse",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "Input sources, switching, special character entry, shortcuts.",
            Touchpad => "Touchpad speed, click options, gestures.",
            Mouse => "Mouse speed, acceleration, natural scrolling.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "input-keyboard-symbolic",
            Touchpad => "input-touchpad-symbolic",
            Mouse => "input-mouse-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::InputDevices(None)
    }

    fn into_page(self) -> Page {
        Page::InputDevices(Some(self))
    }
}
