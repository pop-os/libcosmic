#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedSearchResult")]
pub struct BoxedSearchResult(pub Option<pop_launcher::SearchResult>);
