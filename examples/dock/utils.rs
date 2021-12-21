#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedLauncherActive")]
pub struct BoxedSearchResults(pub Vec<pop_launcher::SearchResult>);
