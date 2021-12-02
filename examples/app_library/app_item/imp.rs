use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk4 as gtk;

use gtk::CompositeTemplate;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "app_item.ui")]
pub struct AppItem {
    #[template_child]
    pub name: TemplateChild<gtk::Label>,
    #[template_child]
    pub image: TemplateChild<gtk::Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppItem {
    const NAME: &'static str = "AppItem";
    type Type = super::AppItem;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for AppItem {}
impl WidgetImpl for AppItem {}
impl BoxImpl for AppItem {}
