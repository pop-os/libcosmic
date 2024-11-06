#[cfg(feature = "desktop")]
use std::collections::BTreeMap;

#[cfg(feature = "desktop")]
#[derive(Debug, Default, Clone, derive_setters::Setters)]
#[setters(prefix = "set_", into, strip_option)]
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
