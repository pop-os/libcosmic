use super::{Page, SubPage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeAndLanguagePage {
    DateAndTime,
    RegionAndLanguage,
}

impl SubPage for TimeAndLanguagePage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "Date & Time",
            RegionAndLanguage => "Region & Language",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "Time zone, automatic clock settings, and some time formatting.",
            RegionAndLanguage => "Format dates, times, and numbers based on your region",
        }
    }

    fn icon_name(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "preferences-system-time-symbolic",
            RegionAndLanguage => "preferences-desktop-locale-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::TimeAndLanguage(None)
    }

    fn into_page(self) -> Page {
        Page::TimeAndLanguage(Some(self))
    }
}
