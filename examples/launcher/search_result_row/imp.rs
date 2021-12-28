use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk4 as gtk;

use gtk::CompositeTemplate;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "application_row.ui")]
pub struct SearchResultRow {
    #[template_child]
    pub name: TemplateChild<gtk::Label>,
    #[template_child]
    pub description: TemplateChild<gtk::Label>,
    #[template_child]
    pub shortcut: TemplateChild<gtk::Label>,
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
    #[template_child]
    pub categoryimage: TemplateChild<gtk::Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchResultRow {
    const NAME: &'static str = "SearchResultRow";
    type Type = super::SearchResultRow;
    type ParentType = gtk::Box;

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
