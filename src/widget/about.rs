use {
    crate::{
        Element,
        iced::{Alignment, Length},
        widget::{self, horizontal_space},
    },
    license::License,
};

#[derive(Debug, Default, Clone, derive_setters::Setters)]
#[setters(into, strip_option)]
/// Information about the application.
pub struct About {
    /// The application's name.
    name: Option<String>,
    /// The application's icon name.
    icon: Option<widget::icon::Handle>,
    /// The application's version.
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
    artists: Vec<(String, String)>,
    /// Designers who contributed to the application.
    #[setters(skip)]
    designers: Vec<(String, String)>,
    /// Developers who contributed to the application.
    #[setters(skip)]
    developers: Vec<(String, String)>,
    /// Documenters who contributed to the application.
    #[setters(skip)]
    documenters: Vec<(String, String)>,
    /// Translators who contributed to the application.
    #[setters(skip)]
    translators: Vec<(String, String)>,
    /// Links associated with the application.
    #[setters(skip)]
    links: Vec<(String, String)>,
}

fn add_contributors(contributors: Vec<(&str, &str)>) -> Vec<(String, String)> {
    contributors
        .into_iter()
        .map(|(name, email)| (name.to_string(), format!("mailto:{email}")))
        .collect()
}

impl<'a> About {
    /// Artists who contributed to the application.
    pub fn artists(mut self, artists: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.artists = add_contributors(artists.into());
        self
    }

    /// Designers who contributed to the application.
    pub fn designers(mut self, designers: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.designers = add_contributors(designers.into());
        self
    }

    /// Developers who contributed to the application.
    pub fn developers(mut self, developers: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.developers = add_contributors(developers.into());
        self
    }

    /// Documenters who contributed to the application.
    pub fn documenters(mut self, documenters: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.documenters = add_contributors(documenters.into());
        self
    }

    /// Translators who contributed to the application.
    pub fn translators(mut self, translators: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
        self.translators = add_contributors(translators.into());
        self
    }

    /// Links associated with the application.
    pub fn links<K: Into<String>, V: Into<String>>(
        mut self,
        links: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        self.links = links
            .into_iter()
            .map(|(name, url)| (name.into(), url.into()))
            .collect();
        self
    }

    fn license_url(&self) -> Option<String> {
        self.license.as_ref().and_then(|license_str| {
            let license: &dyn License = license_str.parse().ok()?;
            Some(format!("https://spdx.org/licenses/{}.html", license.id()))
        })
    }
}

/// Constructs the widget for the about section.
pub fn about<'a, Message: Clone + 'static>(
    about: &'a About,
    on_url_press: impl Fn(String) -> Message,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs, space_m, ..
    } = crate::theme::spacing();

    let section = |list: &'a Vec<(String, String)>, title: &'a str| {
        (!list.is_empty()).then_some({
            let items: Vec<Element<Message>> =
                list.iter()
                    .map(|(name, url)| {
                        widget::button::custom(
                            widget::row()
                                .push(widget::text(name))
                                .push(horizontal_space())
                                .push_maybe((!url.is_empty()).then_some(
                                    crate::widget::icon::from_name("link-symbolic").icon(),
                                ))
                                .align_y(Alignment::Center),
                        )
                        .class(crate::theme::Button::Link)
                        .on_press(on_url_press(url.clone()))
                        .width(Length::Fill)
                        .into()
                    })
                    .collect();
            widget::settings::section().title(title).extend(items)
        })
    };

    let application_name = about.name.as_ref().map(widget::text::title3);
    let application_icon = about.icon.as_ref().map(|i| i.clone().icon());
    let author = about.author.as_ref().map(widget::text::body);
    let version = about.version.as_ref().map(widget::button::standard);
    let links_section = section(&about.links, "Links");
    let developers_section = section(&about.developers, "Developers");
    let designers_section = section(&about.designers, "Designers");
    let artists_section = section(&about.artists, "Artists");
    let translators_section = section(&about.translators, "Translators");
    let documenters_section = section(&about.documenters, "Documenters");
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
                    .align_y(Alignment::Center),
            )
            .class(crate::theme::Button::Link)
            .on_press(on_url_press(url.unwrap_or_default()))
            .width(Length::Fill),
        )
    });
    let copyright = about.copyright.as_ref().map(widget::text::body);
    let comments = about.comments.as_ref().map(widget::text::body);

    widget::column()
        .push(
            widget::column()
                .push_maybe(application_icon)
                .push_maybe(application_name)
                .push_maybe(author)
                .push_maybe(version)
                .align_x(Alignment::Center)
                .spacing(space_xxs),
        )
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
        .spacing(space_m)
        .width(Length::Fill)
        .into()
}
