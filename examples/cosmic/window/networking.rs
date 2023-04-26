use super::{Page, SubPage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkingPage {
    Wired,
    OnlineAccounts,
}

impl SubPage for NetworkingPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "Wired",
            OnlineAccounts => "Online Accounts",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "Wired connection, connection profiles",
            OnlineAccounts => "Add accounts, IMAP and SMTP, enterprise logins",
        }
    }

    fn icon_name(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "network-workgroup-symbolic",
            OnlineAccounts => "goa-panel-symbolic", //TODO: new icon
        }
    }

    fn parent_page(&self) -> Page {
        Page::Networking(None)
    }

    fn into_page(self) -> Page {
        Page::Networking(Some(self))
    }
}
