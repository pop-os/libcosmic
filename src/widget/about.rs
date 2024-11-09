use iced_widget::tooltip::Position;
use license::License;
#[cfg(feature = "desktop")]
use {
    crate::{
        iced::{alignment::Vertical, Alignment, Length},
        widget::{self, horizontal_space},
        Element,
    },
    std::collections::BTreeMap,
};

#[derive(Debug, Default, Clone, derive_setters::Setters)]
#[setters(into, strip_option)]
/// Information about the application.
pub struct About {
    /// The application's name.
    name: Option<String>,
    /// The application's icon name.
    icon: Option<String>,
    /// The application’s version.
    version: Option<String>,
    /// Name of the application's author.
    author: Option<String>,
    /// Comments about the application.
    comments: Option<String>,
    /// The application's copyright.
    copyright: Option<String>,
    /// The license name.
    license: Option<String>,
    /// Artists who contributed to the application.
    #[setters(skip)]
    artists: BTreeMap<String, String>,
    /// Designers who contributed to the application.
    #[setters(skip)]
    designers: BTreeMap<String, String>,
    /// Developers who contributed to the application.
    #[setters(skip)]
    developers: BTreeMap<String, String>,
    /// Documenters who contributed to the application.
    #[setters(skip)]
    documenters: BTreeMap<String, String>,
    /// Translators who contributed to the application.
    #[setters(skip)]
    translators: BTreeMap<String, String>,
    /// Links associated with the application.
    #[setters(skip)]
    links: BTreeMap<String, String>,
}

impl<'a> About {
    /// Artists who contributed to the application.
    pub fn artists(mut self, artists: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let artists: BTreeMap<&'a str, &'a str> = artists.into();
        self.artists = artists
            .into_iter()
            .map(|(k, v)| (k.to_string(), format!("mailto:{v}")))
            .collect();
        self
    }

    /// Designers who contributed to the application.
    pub fn designers(mut self, designers: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let designers: BTreeMap<&'a str, &'a str> = designers.into();
        self.designers = designers
            .into_iter()
            .map(|(k, v)| (k.to_string(), format!("mailto:{v}")))
            .collect();
        self
    }

    /// Developers who contributed to the application.
    pub fn developers(mut self, developers: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let developers: BTreeMap<&'a str, &'a str> = developers.into();
        self.developers = developers
            .into_iter()
            .map(|(k, v)| (k.to_string(), format!("mailto:{v}")))
            .collect();
        self
    }

    /// Documenters who contributed to the application.
    pub fn documenters(mut self, documenters: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let documenters: BTreeMap<&'a str, &'a str> = documenters.into();
        self.documenters = documenters
            .into_iter()
            .map(|(k, v)| (k.to_string(), format!("mailto:{v}")))
            .collect();
        self
    }

    /// Translators who contributed to the application.
    pub fn translators(mut self, translators: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let translators: BTreeMap<&'a str, &'a str> = translators.into();
        self.translators = translators
            .into_iter()
            .map(|(k, v)| (k.to_string(), format!("mailto:{v}")))
            .collect();
        self
    }

    /// Links associated with the application.
    pub fn links(mut self, links: impl Into<BTreeMap<&'a str, &'a str>>) -> Self {
        let links: BTreeMap<&'a str, &'a str> = links.into();
        self.links = links
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    fn license_url(&self) -> Option<String> {
        let license: &dyn License = match self.license.as_ref() {
            Some(license) => license.parse().ok()?,
            None => return None,
        };

        self.license
            .as_ref()
            .map(|_| format!("https://spdx.org/licenses/{}.html", license.id()))
    }
}

/// Constructs the widget for the about section.
pub fn about<'a, Message: Clone + 'static>(
    about: &'a About,
    on_url_press: impl Fn(String) -> Message,
) -> Element<'a, Message> {
    let spacing = crate::theme::active().cosmic().spacing;

    let section = |list: &'a BTreeMap<String, String>, title: &'a str| {
        (!list.is_empty()).then_some({
            let developers: Vec<Element<Message>> =
                list.iter()
                    .map(|(name, url)| {
                        widget::button::custom(
                            widget::row()
                                .push(widget::text(name))
                                .push(horizontal_space())
                                .push_maybe((!url.is_empty()).then_some(
                                    crate::widget::icon::from_name("link-symbolic").icon(),
                                ))
                                .padding(spacing.space_xxs)
                                .align_y(Vertical::Center),
                        )
                        .class(crate::theme::Button::Text)
                        .on_press(on_url_press(url.clone()))
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();
            widget::settings::section().title(title).extend(developers)
        })
    };

    let application_name = about.name.as_ref().map(widget::text::title3);
    let application_icon = about
        .icon
        .as_ref()
        .map(|icon| crate::desktop::IconSource::Name(icon.clone()).as_cosmic_icon());

    let links_section = section(&about.links, "Links");
    let developers_section = section(&about.developers, "Developers");
    let designers_section = section(&about.designers, "Designers");
    let artists_section = section(&about.artists, "Artists");
    let translators_section = section(&about.translators, "Translators");
    let documenters_section = section(&about.documenters, "Documenters");
    let author = about.author.as_ref().map(widget::text);
    let version = about.version.as_ref().map(widget::button::standard);
    let license = about.license.as_ref().map(|license| {
        let url = about.license_url();
        widget::settings::section().title("License").add(
            widget::button::custom(
                widget::row()
                    .push(widget::text(license))
                    .push(horizontal_space())
                    .push_maybe(
                        url.is_some()
                            .then_some(crate::widget::icon::from_name("link-symbolic").icon()),
                    )
                    .padding(spacing.space_xxs)
                    .align_y(Vertical::Center),
            )
            .class(crate::theme::Button::Text)
            .on_press(on_url_press(url.unwrap_or(String::new())))
            .width(Length::Fill),
        )
    });
    let copyright = about.copyright.as_ref().map(widget::text::body);
    let comments = about.comments.as_ref().map(widget::text::body);

    widget::scrollable(
        widget::column()
            .push_maybe(application_icon)
            .push_maybe(application_name)
            .push_maybe(author)
            .push_maybe(version)
            .push_maybe(license)
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
