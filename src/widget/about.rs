#[cfg(feature = "desktop")]
use {
    crate::{
        iced::{alignment::Vertical, Alignment, Length},
        widget::{self, horizontal_space},
        Element,
    },
    std::collections::BTreeMap,
};

#[cfg(feature = "desktop")]
#[derive(Debug, Default, Clone, derive_setters::Setters)]
#[setters(prefix = "set_", into, strip_option)]
/// Information about the application.
pub struct About {
    /// The application's name.
    pub application_name: Option<String>,
    /// The application's icon name.
    pub application_icon: Option<String>,
    /// Artists who contributed to the application.
    #[setters(skip)]
    pub artists: BTreeMap<String, String>,
    /// Comments about the application.
    pub comments: Option<String>,
    /// The application's copyright.
    pub copyright: Option<String>,
    /// Designers who contributed to the application.
    #[setters(skip)]
    pub designers: BTreeMap<String, String>,
    /// Name of the application's developer.
    pub developer_name: Option<String>,
    /// Developers who contributed to the application.
    #[setters(skip)]
    pub developers: BTreeMap<String, String>,
    /// Documenters who contributed to the application.
    #[setters(skip)]
    pub documenters: BTreeMap<String, String>,
    /// The license text.
    pub license: Option<String>,
    /// The license from a list of known licenses.
    pub license_type: Option<String>,
    /// The URL of the application’s support page.
    #[setters(skip)]
    pub support_url: Option<String>,
    /// The URL of the application’s repository.
    #[setters(skip)]
    pub repository_url: Option<String>,
    /// Translators who contributed to the application.
    #[setters(skip)]
    pub translators: BTreeMap<String, String>,
    /// Links associated with the application.
    #[setters(skip)]
    pub links: BTreeMap<String, String>,
    /// The application’s version.
    pub version: Option<String>,
    /// The application’s website.
    #[setters(skip)]
    pub website: Option<String>,
}

impl About {
    pub fn set_repository_url(mut self, repository_url: impl Into<String>) -> Self {
        let repository_url = repository_url.into();
        self.repository_url = Some(repository_url.clone());
        self.links.insert("Repository".into(), repository_url);
        self
    }

    pub fn set_support_url(mut self, support_url: impl Into<String>) -> Self {
        let support_url = support_url.into();
        self.support_url = Some(support_url.clone());
        self.links.insert("Support".into(), support_url);
        self
    }

    pub fn set_website(mut self, website: impl Into<String>) -> Self {
        let website = website.into();
        self.website = Some(website.clone());
        self.links.insert("Website".into(), website);
        self
    }

    pub fn set_artists(mut self, artists: impl Into<BTreeMap<String, String>>) -> Self {
        let artists: BTreeMap<String, String> = artists.into();
        self.artists = artists
            .into_iter()
            .map(|(k, v)| (k, format!("mailto:{v}")))
            .collect();
        self
    }

    pub fn set_designers(mut self, designers: impl Into<BTreeMap<String, String>>) -> Self {
        let designers: BTreeMap<String, String> = designers.into();
        self.designers = designers
            .into_iter()
            .map(|(k, v)| (k, format!("mailto:{v}")))
            .collect();
        self
    }

    pub fn set_developers(mut self, developers: impl Into<BTreeMap<String, String>>) -> Self {
        let developers: BTreeMap<String, String> = developers.into();
        self.developers = developers
            .into_iter()
            .map(|(k, v)| (k, format!("mailto:{v}")))
            .collect();
        self
    }

    pub fn set_documenters(mut self, documenters: impl Into<BTreeMap<String, String>>) -> Self {
        let documenters: BTreeMap<String, String> = documenters.into();
        self.documenters = documenters
            .into_iter()
            .map(|(k, v)| (k, format!("mailto:{v}")))
            .collect();
        self
    }

    pub fn set_translators(mut self, translators: impl Into<BTreeMap<String, String>>) -> Self {
        let translators: BTreeMap<String, String> = translators.into();
        self.translators = translators
            .into_iter()
            .map(|(k, v)| (k, format!("mailto:{v}")))
            .collect();
        self
    }
}

#[cfg(feature = "desktop")]
/// Constructs the widget for the about section.
pub fn about<'a, Message: Clone + 'static>(
    about: &'a About,
    on_url_press: impl Fn(String) -> Message,
) -> Element<'a, Message> {
    let spacing = crate::theme::active().cosmic().spacing;

    let section = |list: &'a BTreeMap<String, String>, title: &'a str| {
        if list.is_empty() {
            None
        } else {
            let developers: Vec<Element<Message>> = list
                .into_iter()
                .map(|(name, url)| {
                    widget::button::custom(
                        widget::row()
                            .push(widget::text(name))
                            .push(horizontal_space())
                            .push(crate::widget::icon::from_name("link-symbolic").icon())
                            .padding(spacing.space_xxs)
                            .align_y(Vertical::Center),
                    )
                    .class(crate::theme::Button::Text)
                    .on_press(on_url_press(url.clone()))
                    .width(Length::Fill)
                    .into()
                })
                .collect();
            Some(widget::settings::section().title(title).extend(developers))
        }
    };

    let application_name = about.application_name.as_ref().map(widget::text::title3);
    let application_icon = about
        .application_icon
        .as_ref()
        .map(|icon| crate::desktop::IconSource::Name(icon.clone()).as_cosmic_icon());

    let links_section = section(&about.links, "Links");
    let developers_section = section(&about.developers, "Developers");
    let designers_section = section(&about.designers, "Designers");
    let artists_section = section(&about.artists, "Artists");
    let translators_section = section(&about.translators, "Translators");
    let documenters_section = section(&about.documenters, "Documenters");

    let developer_name = about.developer_name.as_ref().map(widget::text);
    let version = about.version.as_ref().map(widget::button::standard);
    let license = about.license_type.as_ref().map(widget::button::standard);
    let copyright = about.copyright.as_ref().map(widget::text::body);
    let comments = about.comments.as_ref().map(widget::text::body);

    widget::scrollable(
        widget::column()
            .push_maybe(application_icon)
            .push_maybe(application_name)
            .push_maybe(developer_name)
            .push(
                widget::row()
                    .push_maybe(version)
                    .push_maybe(license)
                    .spacing(spacing.space_xs),
            )
            .push_maybe(links_section)
            .push_maybe(developers_section)
            .push_maybe(designers_section)
            .push_maybe(artists_section)
            .push_maybe(translators_section)
            .push_maybe(documenters_section)
            .push_maybe(comments)
            .push_maybe(copyright)
            .align_x(Alignment::Center)
            .spacing(spacing.space_xs)
            .width(Length::Fill),
    )
    .into()
}
