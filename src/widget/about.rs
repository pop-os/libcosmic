use crate::{
    Apply, Element, fl,
    iced::{Alignment, Length},
    widget::{self, horizontal_space},
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
    /// The license url.
    license_url: Option<String>,
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

macro_rules! set_contributors {
    ($field:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $field(mut self, contributors: impl Into<Vec<(&'a str, &'a str)>>) -> Self {
            self.$field = add_contributors(contributors.into());
            self
        }
    };
}

impl<'a> About {
    set_contributors!(artists, "Artists who contributed to the application.");
    set_contributors!(designers, "Designers who contributed to the application.");
    set_contributors!(developers, "Developers who contributed to the application.");
    set_contributors!(
        documenters,
        "Documenters who contributed to the application."
    );
    set_contributors!(
        translators,
        "Translators who contributed to the application."
    );

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
}

/// Constructs the widget for the about section.
pub fn about<'a, Message: Clone + 'static>(
    about: &'a About,
    on_url_press: impl Fn(&'a str) -> Message + 'a,
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs, space_m, ..
    } = crate::theme::spacing();

    let section_button = |name: &'a str, url: &'a str| -> Element<'a, Message> {
        widget::row()
            .push(widget::text(name))
            .push(horizontal_space())
            .push_maybe(
                (!url.is_empty()).then_some(crate::widget::icon::from_name("link-symbolic").icon()),
            )
            .align_y(Alignment::Center)
            .apply(widget::button::custom)
            .class(crate::theme::Button::Link)
            .on_press(on_url_press(url))
            .width(Length::Fill)
            .into()
    };

    let section = |list: &'a Vec<(String, String)>, title: String| {
        (!list.is_empty()).then_some({
            let items = list.iter().map(|(name, url)| section_button(name, url));
            widget::settings::section().title(title).extend(items)
        })
    };

    let header_children: Vec<Element<Message>> = [
        about.icon.as_ref().map(|i| {
            i.clone()
                .icon()
                .size(256)
                .width(Length::Fixed(128.))
                .height(Length::Fixed(128.))
                .content_fit(iced::ContentFit::Contain)
                .into()
        }),
        about.name.as_ref().map(|n| widget::text::title3(n).into()),
        about.author.as_ref().map(|a| widget::text::body(a).into()),
        about.version.as_ref().map(|v| {
            widget::button::standard(v)
                .apply(widget::container)
                .padding([space_xxs, 0, 0, 0])
                .into()
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    let header = (!header_children.is_empty())
        .then_some(widget::column::with_children(header_children).align_x(Alignment::Center));

    let links_section = section(&about.links, fl!("links"));
    let developers_section = section(&about.developers, fl!("developers"));
    let designers_section = section(&about.designers, fl!("designers"));
    let artists_section = section(&about.artists, fl!("artists"));
    let translators_section = section(&about.translators, fl!("translators"));
    let documenters_section = section(&about.documenters, fl!("documenters"));
    let license_section = about.license.as_ref().map(|license| {
        let url = about.license_url.as_deref().unwrap_or_default();
        widget::settings::section()
            .title(fl!("license"))
            .add(section_button(license, url))
    });
    let copyright = about.copyright.as_ref().map(widget::text::body);
    let comments = about.comments.as_ref().map(widget::text::body);

    widget::column()
        .push_maybe(header)
        .push_maybe(links_section)
        .push_maybe(developers_section)
        .push_maybe(designers_section)
        .push_maybe(artists_section)
        .push_maybe(translators_section)
        .push_maybe(documenters_section)
        .push_maybe(license_section)
        .push_maybe(comments)
        .push_maybe(copyright)
        .spacing(space_m)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}
