use derive_setters::Setters;

#[derive(Setters, Default, Debug, Clone)]
pub struct ListRow<'a> {
    pub(crate) title: &'a str,
    #[setters(strip_option)]
    pub subtitle: Option<&'a str>,
    #[setters(strip_option)]
    pub icon: Option<String>,
}

pub fn list_row<'a>() -> ListRow<'a> {
    ListRow {
        title: "",
        subtitle: None,
        icon: None,
    }
}
