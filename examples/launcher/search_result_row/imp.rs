use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::CompositeTemplate;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "application_row.ui")]
pub struct SearchResultRow {
    #[template_child]
    pub name: TemplateChild<gtk4::Label>,
    #[template_child]
    pub description: TemplateChild<gtk4::Label>,
    #[template_child]
    pub shortcut: TemplateChild<gtk4::Label>,
    #[template_child]
    pub image: TemplateChild<gtk4::Image>,
    #[template_child]
    pub categoryimage: TemplateChild<gtk4::Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchResultRow {
    const NAME: &'static str = "SearchResultRow";
    type Type = super::SearchResultRow;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for SearchResultRow {}

impl WidgetImpl for SearchResultRow {}

impl BoxImpl for SearchResultRow {}
